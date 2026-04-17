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

/// A pair of reads from paired-end sequencing.
#[derive(Debug, Clone)]
pub struct ReadPair {
    /// Read name (same for both reads, without /1 or /2 suffix)
    pub name: String,
    /// First read sequence
    pub seq1: Vec<u8>,
    /// First read quality
    pub qual1: Vec<u8>,
    /// Second read sequence
    pub seq2: Vec<u8>,
    /// Second read quality
    pub qual2: Vec<u8>,
}

pub struct FastqReader<R: BufRead> {
    reader: R,
    buf: String,
    done: bool,
}

impl<R: BufRead> FastqReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: String::new(),
            done: false,
        }
    }

    pub fn next_record(&mut self) -> Result<Option<FastqRecord>> {
        if self.done {
            return Ok(None);
        }

        // header line starting with '@'
        self.buf.clear();
        let mut n = self.reader.read_line(&mut self.buf)?;
        if n == 0 {
            self.done = true;
            return Ok(None);
        }
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
        if n == 0 {
            return Err(anyhow!("unexpected EOF after header"));
        }
        let seq = self.buf.trim_end().as_bytes().to_vec();

        // plus line
        self.buf.clear();
        n = self.reader.read_line(&mut self.buf)?;
        if n == 0 || !self.buf.starts_with('+') {
            return Err(anyhow!("missing '+' line"));
        }

        // quality line
        self.buf.clear();
        n = self.reader.read_line(&mut self.buf)?;
        if n == 0 {
            return Err(anyhow!("missing quality line"));
        }
        let qual = self.buf.trim_end().as_bytes().to_vec();

        // If quality length is shorter than seq (line-wrapped seq not supported here), error
        if qual.len() != seq.len() {
            return Err(anyhow!("seq/qual length mismatch"));
        }

        Ok(Some(FastqRecord { id, desc, seq, qual }))
    }
}

/// Reader for paired-end FASTQ files.
/// Supports both separate files and interleaved format.
pub struct PairedFastqReader<R1: BufRead, R2: BufRead> {
    reader1: FastqReader<R1>,
    reader2: Option<FastqReader<R2>>,
    #[allow(dead_code)]
    buf: String,
    done: bool,
}

impl<R1: BufRead, R2: BufRead> PairedFastqReader<R1, R2> {
    /// Create a reader for two separate FASTQ files.
    pub fn new_separate(reader1: R1, reader2: R2) -> Self {
        Self {
            reader1: FastqReader::new(reader1),
            reader2: Some(FastqReader::new(reader2)),
            buf: String::new(),
            done: false,
        }
    }

    /// Create a reader for interleaved FASTQ (reads alternate R1, R2).
    pub fn new_interleaved(reader: R1) -> Self {
        Self {
            reader1: FastqReader::new(reader),
            reader2: None,
            buf: String::new(),
            done: false,
        }
    }

    /// Read the next pair of reads.
    pub fn next_pair(&mut self) -> Result<Option<ReadPair>> {
        if self.done {
            return Ok(None);
        }

        if let Some(ref mut reader2) = self.reader2 {
            // Separate files mode
            let rec1 = self.reader1.next_record()?;
            let rec2 = reader2.next_record()?;

            match (rec1, rec2) {
                (Some(r1), Some(r2)) => {
                    // Remove /1 /2 suffixes if present and ensure names match
                    let name1 = strip_read_suffix(&r1.id);
                    let name2 = strip_read_suffix(&r2.id);

                    if name1 != name2 {
                        return Err(anyhow!("read name mismatch: '{}' vs '{}'", name1, name2));
                    }

                    Ok(Some(ReadPair {
                        name: name1,
                        seq1: r1.seq,
                        qual1: r1.qual,
                        seq2: r2.seq,
                        qual2: r2.qual,
                    }))
                }
                (None, None) => {
                    self.done = true;
                    Ok(None)
                }
                (Some(_), None) => Err(anyhow!("R1 file has more reads than R2")),
                (None, Some(_)) => Err(anyhow!("R2 file has more reads than R1")),
            }
        } else {
            // Interleaved mode
            let rec1 = self.reader1.next_record()?;
            if rec1.is_none() {
                self.done = true;
                return Ok(None);
            }
            let r1 = rec1.unwrap();

            let rec2 = self.reader1.next_record()?;
            if rec2.is_none() {
                return Err(anyhow!("interleaved FASTQ has odd number of reads"));
            }
            let r2 = rec2.unwrap();

            let name1 = strip_read_suffix(&r1.id);
            let name2 = strip_read_suffix(&r2.id);

            if name1 != name2 {
                return Err(anyhow!("interleaved read name mismatch: '{}' vs '{}'", name1, name2));
            }

            Ok(Some(ReadPair {
                name: name1,
                seq1: r1.seq,
                qual1: r1.qual,
                seq2: r2.seq,
                qual2: r2.qual,
            }))
        }
    }
}

/// Strip /1 or /2 suffix from read name.
fn strip_read_suffix(name: &str) -> String {
    if name.ends_with("/1") || name.ends_with("/2") {
        name[..name.len() - 2].to_string()
    } else {
        name.to_string()
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
