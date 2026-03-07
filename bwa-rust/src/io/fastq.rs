use anyhow::{anyhow, Result};
use std::io::BufRead;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FastqRecord {
    pub id: String,
    pub desc: Option<String>,
    pub seq: Vec<u8>,
    pub qual: Vec<u8>,
}

pub struct FastqReader<R: BufRead> {
    reader: R,
    buf: String,
    done: bool,
}

impl<R: BufRead> FastqReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader, buf: String::new(), done: false }
    }

    pub fn next_record(&mut self) -> Result<Option<FastqRecord>> {
        if self.done { return Ok(None); }

        // header line starting with '@'
        self.buf.clear();
        let mut n = self.reader.read_line(&mut self.buf)?;
        if n == 0 { self.done = true; return Ok(None); }
        if !self.buf.starts_with('@') {
            return Err(anyhow!("FASTQ header not starting with '@'"));
        }
        let header = self.buf[1..].trim_end().to_string();
        let mut parts = header.splitn(2, char::is_whitespace);
        let id = parts.next().unwrap_or("").to_string();
        let desc = parts.next().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

        // sequence line
        self.buf.clear();
        n = self.reader.read_line(&mut self.buf)?;
        if n == 0 { return Err(anyhow!("unexpected EOF after header")); }
        let seq = self.buf.trim_end().as_bytes().to_vec();

        // plus line
        self.buf.clear();
        n = self.reader.read_line(&mut self.buf)?;
        if n == 0 || !self.buf.starts_with('+') { return Err(anyhow!("missing '+' line")); }

        // quality line
        self.buf.clear();
        n = self.reader.read_line(&mut self.buf)?;
        if n == 0 { return Err(anyhow!("missing quality line")); }
        let qual = self.buf.trim_end().as_bytes().to_vec();

        // If quality length is shorter than seq (line-wrapped seq not supported here), error
        if qual.len() != seq.len() { return Err(anyhow!("seq/qual length mismatch")); }

        Ok(Some(FastqRecord { id, desc, seq, qual }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_simple_fastq() {
        let data = b"@read1 desc1\nACGT\n+\nIIII\n@read2\nTTAA\n+\nHHHH\n";
        let mut r = FastqReader::new(Cursor::new(&data[..]));

        let r1 = r.next_record().unwrap().unwrap();
        assert_eq!(r1.id, "read1");
        assert_eq!(r1.desc.as_deref(), Some("desc1"));
        assert_eq!(r1.seq, b"ACGT");
        assert_eq!(r1.qual, b"IIII");

        let r2 = r.next_record().unwrap().unwrap();
        assert_eq!(r2.id, "read2");
        assert_eq!(r2.desc, None);
        assert_eq!(r2.seq, b"TTAA");
        assert_eq!(r2.qual, b"HHHH");

        assert!(r.next_record().unwrap().is_none());
    }

    #[test]
    fn parse_fastq_with_crlf() {
        let data = b"@read1\r\nACGT\r\n+\r\nIIII\r\n";
        let mut r = FastqReader::new(Cursor::new(&data[..]));
        let r1 = r.next_record().unwrap().unwrap();
        assert_eq!(r1.id, "read1");
        assert_eq!(r1.seq, b"ACGT");
        assert_eq!(r1.qual, b"IIII");
        assert!(r.next_record().unwrap().is_none());
    }

    #[test]
    fn parse_fastq_empty_input() {
        let data = b"";
        let mut r = FastqReader::new(Cursor::new(&data[..]));
        assert!(r.next_record().unwrap().is_none());
    }

    #[test]
    fn parse_fastq_bad_header() {
        let data = b"ACGT\n+\nIIII\n";
        let mut r = FastqReader::new(Cursor::new(&data[..]));
        assert!(r.next_record().is_err());
    }

    #[test]
    fn parse_fastq_missing_plus() {
        let data = b"@read1\nACGT\nIIII\n";
        let mut r = FastqReader::new(Cursor::new(&data[..]));
        assert!(r.next_record().is_err());
    }

    #[test]
    fn parse_fastq_seq_qual_length_mismatch() {
        let data = b"@read1\nACGT\n+\nIII\n";
        let mut r = FastqReader::new(Cursor::new(&data[..]));
        assert!(r.next_record().is_err());
    }

    #[test]
    fn parse_fastq_truncated_after_header() {
        let data = b"@read1\n";
        let mut r = FastqReader::new(Cursor::new(&data[..]));
        assert!(r.next_record().is_err());
    }

    #[test]
    fn parse_fastq_multiple_reads() {
        let data = b"@r1\nA\n+\nI\n@r2\nCC\n+\nHH\n@r3\nGGG\n+\nJJJ\n";
        let mut r = FastqReader::new(Cursor::new(&data[..]));
        let r1 = r.next_record().unwrap().unwrap();
        assert_eq!(r1.id, "r1");
        assert_eq!(r1.seq, b"A");
        let r2 = r.next_record().unwrap().unwrap();
        assert_eq!(r2.id, "r2");
        assert_eq!(r2.seq, b"CC");
        let r3 = r.next_record().unwrap().unwrap();
        assert_eq!(r3.id, "r3");
        assert_eq!(r3.seq, b"GGG");
        assert!(r.next_record().unwrap().is_none());
    }

    #[test]
    fn parse_fastq_description_with_spaces() {
        let data = b"@read1 some long description here\nACGT\n+\nIIII\n";
        let mut r = FastqReader::new(Cursor::new(&data[..]));
        let r1 = r.next_record().unwrap().unwrap();
        assert_eq!(r1.id, "read1");
        assert_eq!(r1.desc.as_deref(), Some("some long description here"));
    }

    #[test]
    fn parse_fastq_lowercase_seq() {
        let data = b"@read1\nacgt\n+\nIIII\n";
        let mut r = FastqReader::new(Cursor::new(&data[..]));
        let r1 = r.next_record().unwrap().unwrap();
        assert_eq!(r1.seq, b"acgt");
    }
}
