use std::io::BufRead;
use std::path::Path;

use anyhow::Result;

use super::{bwt, fm, sa};
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
        contigs.push(fm::Contig {
            name: rec.id,
            len: len_u32,
            offset: start,
        });
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
pub fn build_fm_from_fasta(path: impl AsRef<Path>, block_size: usize) -> Result<IndexBuildResult> {
    let path = path.as_ref();
    let fh = std::fs::File::open(path).map_err(|e| anyhow::anyhow!("cannot open FASTA '{}': {}", path.display(), e))?;
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

    #[test]
    fn build_single_seq_fasta() {
        let data = b">seq1\nACGTACGT\n";
        let cursor = Cursor::new(&data[..]);
        let result = build_fm_index(cursor, 4).unwrap();
        assert_eq!(result.n_seqs, 1);
        assert_eq!(result.total_len, 8);
        assert_eq!(result.fm.contigs.len(), 1);
        assert_eq!(result.fm.contigs[0].name, "seq1");
        assert_eq!(result.fm.contigs[0].len, 8);
        assert_eq!(result.fm.contigs[0].offset, 0);
    }

    #[test]
    fn build_multi_contig_offsets() {
        let data = b">c1\nACGT\n>c2\nGGCC\n>c3\nTT\n";
        let cursor = Cursor::new(&data[..]);
        let result = build_fm_index(cursor, 4).unwrap();
        assert_eq!(result.n_seqs, 3);
        assert_eq!(result.total_len, 10);
        assert_eq!(result.fm.contigs[0].offset, 0);
        assert_eq!(result.fm.contigs[0].len, 4);
        // c2 starts after c1 + sentinel
        assert_eq!(result.fm.contigs[1].offset, 5);
        assert_eq!(result.fm.contigs[1].len, 4);
        // c3 starts after c2 + sentinel
        assert_eq!(result.fm.contigs[2].offset, 10);
        assert_eq!(result.fm.contigs[2].len, 2);
    }

    #[test]
    fn build_fasta_preserves_sequence_content() {
        let data = b">chr1\nACGTN\n";
        let cursor = Cursor::new(&data[..]);
        let result = build_fm_index(cursor, 4).unwrap();
        let fm = &result.fm;
        let offset = fm.contigs[0].offset as usize;
        let len = fm.contigs[0].len as usize;
        let recovered: Vec<u8> = fm.text[offset..offset + len].iter().map(|&c| dna::from_alphabet(c)).collect();
        assert_eq!(recovered, b"ACGTN");
    }

    #[test]
    fn build_fasta_search_works() {
        let data = b">chr1\nACGTACGTACGT\n";
        let cursor = Cursor::new(&data[..]);
        let result = build_fm_index(cursor, 4).unwrap();
        let pat: Vec<u8> = b"CGTA".iter().map(|&b| dna::to_alphabet(b)).collect();
        let res = result.fm.backward_search(&pat);
        assert!(res.is_some());
        let (l, r) = res.unwrap();
        assert_eq!(r - l, 2); // "CGTA" appears twice in ACGTACGTACGT
    }

    #[test]
    fn build_fasta_with_lowercase() {
        let data = b">chr1\nacgtacgt\n";
        let cursor = Cursor::new(&data[..]);
        let result = build_fm_index(cursor, 4).unwrap();
        assert_eq!(result.total_len, 8);
        let pat: Vec<u8> = b"ACGT".iter().map(|&b| dna::to_alphabet(b)).collect();
        assert!(result.fm.backward_search(&pat).is_some());
    }
}
