use anyhow::Result;
use std::io::BufRead;

#[derive(Debug, Clone)]
pub struct FastaRecord {
    pub id: String,
    pub desc: Option<String>,
    pub seq: Vec<u8>,
}

pub struct FastaReader<R: BufRead> {
    reader: R,
    buf: String,
    done: bool,
    peek_header: Option<String>,
}

impl<R: BufRead> FastaReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: String::new(),
            done: false,
            peek_header: None,
        }
    }

    pub fn next_record(&mut self) -> Result<Option<FastaRecord>> {
        if self.done {
            return Ok(None);
        }

        // Find header line
        let header = if let Some(h) = self.peek_header.take() {
            h
        } else {
            loop {
                self.buf.clear();
                let n = self.reader.read_line(&mut self.buf)?;
                if n == 0 {
                    self.done = true;
                    return Ok(None);
                }
                if self.buf.starts_with('>') {
                    let h = self.buf[1..].trim().to_string();
                    break h;
                }
            }
        };

        // Parse id and description
        let mut parts = header.splitn(2, char::is_whitespace);
        let id = parts.next().unwrap_or("").to_string();
        let desc = parts
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // Read sequence lines
        let mut seq: Vec<u8> = Vec::new();
        loop {
            self.buf.clear();
            let n = self.reader.read_line(&mut self.buf)?;
            if n == 0 {
                self.done = true;
                break;
            }
            if self.buf.starts_with('>') {
                let h = self.buf[1..].trim().to_string();
                self.peek_header = Some(h);
                break;
            }
            for &b in self.buf.as_bytes() {
                match b {
                    b'\n' | b'\r' | b' ' | b'\t' => {}
                    _ => seq.push(b.to_ascii_uppercase()),
                }
            }
        }

        Ok(Some(FastaRecord { id, desc, seq }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_simple_fasta() {
        let data = b">chr1 first\nACgTNN\n>chr2\nAAA\n";
        let cursor = Cursor::new(&data[..]);
        let mut r = FastaReader::new(cursor);

        let r1 = r.next_record().unwrap().unwrap();
        assert_eq!(r1.id, "chr1");
        assert_eq!(r1.desc.as_deref(), Some("first"));
        assert_eq!(r1.seq, b"ACGTNN");

        let r2 = r.next_record().unwrap().unwrap();
        assert_eq!(r2.id, "chr2");
        assert_eq!(r2.desc, None);
        assert_eq!(r2.seq, b"AAA");

        assert!(r.next_record().unwrap().is_none());
    }

    #[test]
    fn parse_fasta_with_crlf_and_whitespace() {
        let data = b">chr1 desc\r\nAC g t n\r\n acgt\r\n>chr2 \r\n N N N \r\n";
        let cursor = Cursor::new(&data[..]);
        let mut r = FastaReader::new(cursor);

        let r1 = r.next_record().unwrap().unwrap();
        assert_eq!(r1.id, "chr1");
        assert_eq!(r1.desc.as_deref(), Some("desc"));
        assert_eq!(r1.seq, b"ACGTNACGT");

        let r2 = r.next_record().unwrap().unwrap();
        assert_eq!(r2.id, "chr2");
        assert_eq!(r2.desc, None);
        assert_eq!(r2.seq, b"NNN");

        assert!(r.next_record().unwrap().is_none());
    }

    #[test]
    fn parse_fasta_with_leading_empty_lines() {
        let data = b"\n\n>chr1\nACGT\n";
        let cursor = Cursor::new(&data[..]);
        let mut r = FastaReader::new(cursor);

        let r1 = r.next_record().unwrap().unwrap();
        assert_eq!(r1.id, "chr1");
        assert_eq!(r1.desc, None);
        assert_eq!(r1.seq, b"ACGT");

        assert!(r.next_record().unwrap().is_none());
    }
}
