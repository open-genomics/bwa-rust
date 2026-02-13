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
