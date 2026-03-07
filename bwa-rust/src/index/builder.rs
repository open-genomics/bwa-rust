use anyhow::Result;
use std::io::BufRead;

use super::{sa, bwt, fm};
use crate::io::fasta::FastaReader;
use crate::util::dna;

/// Result of building an FM index from FASTA
pub struct IndexBuildResult {
    pub fm: fm::FMIndex,
    pub n_seqs: usize,
    pub total_len: usize,
}

/// Build an FM index from a buffered FASTA reader
pub fn build_fm_index<R: BufRead>(reader: R, block_size: usize) -> Result<IndexBuildResult> {
    let mut fasta = FastaReader::new(reader);

    let mut n_seqs = 0usize;
    let mut total_len = 0usize;
    let mut text: Vec<u8> = Vec::new();
    let mut contigs: Vec<fm::Contig> = Vec::new();

    while let Some(rec) = fasta.next_record()? {
        n_seqs += 1;
        total_len += rec.seq.len();
        let norm = dna::normalize_seq(&rec.seq);
        let start = text.len() as u32;
        for b in norm {
            text.push(dna::to_alphabet(b));
        }
        let len_u32 = (text.len() as u32).saturating_sub(start);
        contigs.push(fm::Contig { name: rec.id, len: len_u32, offset: start });
        // sentinel between contigs
        text.push(0);
    }

    if n_seqs == 0 {
        anyhow::bail!("FASTA contains no sequences");
    }
    if total_len == 0 {
        anyhow::bail!("FASTA contains only empty sequences");
    }

    let sa_arr = sa::build_sa(&text);
    let bwt_arr = bwt::build_bwt(&text, &sa_arr);
    let fm = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, dna::SIGMA as u8, block_size);

    Ok(IndexBuildResult { fm, n_seqs, total_len })
}

/// Convenience: build FM index from a FASTA file path
pub fn build_fm_from_fasta(path: &str, block_size: usize) -> Result<IndexBuildResult> {
    let fh = std::fs::File::open(path)
        .map_err(|e| anyhow::anyhow!("cannot open FASTA '{}': {}", path, e))?;
    let buf = std::io::BufReader::new(fh);
    build_fm_index(buf, block_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn build_from_fasta_reader() {
        let data = b">chr1\nACGTACGT\n>chr2\nGGCC\n";
        let cursor = Cursor::new(&data[..]);
        let result = build_fm_index(cursor, 4).unwrap();
        assert_eq!(result.n_seqs, 2);
        assert_eq!(result.total_len, 12);
        assert_eq!(result.fm.contigs.len(), 2);
        assert_eq!(result.fm.contigs[0].name, "chr1");
        assert_eq!(result.fm.contigs[1].name, "chr2");
    }

    #[test]
    fn build_empty_fasta_fails() {
        let data = b"";
        let cursor = Cursor::new(&data[..]);
        assert!(build_fm_index(cursor, 4).is_err());
    }
}
