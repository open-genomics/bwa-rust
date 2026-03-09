use anyhow::Result;
use std::io::Write;

/// Write SAM header (@HD, @SQ, @PG) to output
pub fn write_header<W: Write, S: AsRef<str>>(out: &mut W, contigs: &[(S, u32)]) -> Result<()> {
    writeln!(out, "@HD\tVN:1.6\tSO:unsorted")?;
    for (name, len) in contigs {
        writeln!(out, "@SQ\tSN:{}\tLN:{}", name.as_ref(), len)?;
    }
    writeln!(out, "@PG\tID:bwa-rust\tPN:bwa-rust\tVN:{}", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

/// Format an unmapped SAM record (FLAG=4)
pub fn format_unmapped(qname: &str, seq: &str, qual: &str) -> String {
    format!("{}\t4\t*\t0\t0\t*\t*\t0\t0\t{}\t{}", qname, seq, qual,)
}

/// Format a mapped SAM record with optional tags
pub fn format_record(
    qname: &str,
    flag: u16,
    rname: &str,
    pos: u32,
    mapq: u8,
    cigar: &str,
    seq: &str,
    qual: &str,
    score: i32,
    sub_score: i32,
    nm: u32,
) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t*\t0\t0\t{}\t{}\tAS:i:{}\tXS:i:{}\tNM:i:{}",
        qname, flag, rname, pos, mapq, cigar, seq, qual, score, sub_score, nm,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_format() {
        let mut buf = Vec::new();
        let contigs = vec![("chr1", 1000u32), ("chr2", 2000u32)];
        write_header(&mut buf, &contigs).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("@HD\tVN:1.6\tSO:unsorted"));
        assert!(s.contains("@SQ\tSN:chr1\tLN:1000"));
        assert!(s.contains("@SQ\tSN:chr2\tLN:2000"));
        assert!(s.contains("@PG\tID:bwa-rust"));
    }

    #[test]
    fn unmapped_format() {
        let line = format_unmapped("read1", "ACGT", "IIII");
        assert!(line.contains("\t4\t"));
        assert!(line.starts_with("read1\t"));
    }

    #[test]
    fn record_format() {
        let line = format_record("read1", 0, "chr1", 100, 60, "50M", "ACGT", "IIII", 100, 0, 2);
        assert!(line.starts_with("read1\t0\tchr1\t100\t60\t50M\t"));
        assert!(line.contains("AS:i:100"));
        assert!(line.contains("NM:i:2"));
    }

    #[test]
    fn header_empty_contigs() {
        let mut buf = Vec::new();
        let contigs: Vec<(&str, u32)> = vec![];
        write_header(&mut buf, &contigs).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("@HD"));
        assert!(s.contains("@PG"));
        assert!(!s.contains("@SQ"));
    }

    #[test]
    fn unmapped_has_correct_tab_count() {
        let line = format_unmapped("r1", "ACGT", "IIII");
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 11);
        assert_eq!(fields[0], "r1");
        assert_eq!(fields[1], "4");
        assert_eq!(fields[2], "*");
        assert_eq!(fields[3], "0");
        assert_eq!(fields[9], "ACGT");
        assert_eq!(fields[10], "IIII");
    }

    #[test]
    fn record_format_reverse_complement() {
        let line = format_record("read1", 16, "chr1", 50, 30, "20M", "ACGT", "IIII", 40, 10, 1);
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields[1], "16");
        assert!(line.contains("XS:i:10"));
    }

    #[test]
    fn record_format_secondary_alignment() {
        let line = format_record("read1", 256, "chr2", 200, 0, "10M1I10M", "ACGT", "IIII", 30, 50, 3);
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields[1], "256");
        assert_eq!(fields[2], "chr2");
        assert_eq!(fields[3], "200");
        assert_eq!(fields[4], "0");
        assert_eq!(fields[5], "10M1I10M");
    }

    #[test]
    fn header_with_string_contigs() {
        let mut buf = Vec::new();
        let contigs = vec![("chrX".to_string(), 155_270_560u32)];
        write_header(&mut buf, &contigs).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("@SQ\tSN:chrX\tLN:155270560"));
    }
}
