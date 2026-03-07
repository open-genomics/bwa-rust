use crate::index::fm::FMIndex;
use crate::util::dna;

use super::extend::chain_to_alignment_buf;
use super::sw;
use super::{build_chains, filter_chains, find_smem_seeds};
use super::{AlignOpt, SwParams};

#[derive(Debug, Clone)]
pub struct AlignCandidate {
    pub score: i32,
    pub is_rev: bool,
    pub rname: String,
    pub pos1: u32,
    pub cigar: String,
    pub nm: u32,
    pub contig_idx: usize,
}

pub fn collect_candidates(
    fm: &FMIndex,
    query_norm: &[u8],
    query_alpha: &[u8],
    sw_params: SwParams,
    is_rev: bool,
    opt: &AlignOpt,
    candidates: &mut Vec<AlignCandidate>,
) {
    let len = query_alpha.len();
    if len == 0 {
        return;
    }

    // BWA 风格：min_seed_len 默认 19，但不超过 read 长度的一半
    let min_mem_len = opt.min_seed_len.min(len / 2 + 1).max(1);
    let seeds = find_smem_seeds(fm, query_alpha, min_mem_len);
    if seeds.is_empty() {
        return;
    }

    // 构建多条链
    let mut chains = build_chains(&seeds, len);
    filter_chains(&mut chains, 0.3);

    let mut sw_buf = sw::SwBuffer::new();

    for ch in &chains {
        let ci = ch.contig;
        let contig = &fm.contigs[ci];
        let offset = contig.offset as usize;
        let contig_len = contig.len as usize;
        if contig_len == 0 {
            continue;
        }

        let mut ref_seq: Vec<u8> = Vec::with_capacity(contig_len);
        for &code in &fm.text[offset..offset + contig_len] {
            ref_seq.push(dna::from_alphabet(code));
        }
        if ref_seq.is_empty() {
            continue;
        }

        let res = chain_to_alignment_buf(ch, query_norm, &ref_seq, sw_params, &mut sw_buf);
        if res.score <= 0 || res.cigar.is_empty() {
            continue;
        }

        candidates.push(AlignCandidate {
            score: res.score,
            is_rev,
            rname: contig.name.clone(),
            pos1: (res.ref_start as u32) + 1,
            cigar: res.cigar,
            nm: res.nm,
            contig_idx: ci,
        });
    }
}

pub fn dedup_candidates(candidates: &mut Vec<AlignCandidate>) {
    let mut keep = vec![true; candidates.len()];
    for i in 0..candidates.len() {
        if !keep[i] {
            continue;
        }
        for j in (i + 1)..candidates.len() {
            if !keep[j] {
                continue;
            }
            if candidates[i].contig_idx == candidates[j].contig_idx
                && candidates[i].pos1 == candidates[j].pos1
                && candidates[i].is_rev == candidates[j].is_rev
            {
                keep[j] = false;
            }
        }
    }
    let mut idx = 0;
    candidates.retain(|_| {
        let k = keep[idx];
        idx += 1;
        k
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::fm::{Contig, FMIndex};
    use crate::index::{bwt, sa};

    fn default_opt() -> AlignOpt {
        AlignOpt::default()
    }

    fn build_test_fm(seq: &[u8]) -> FMIndex {
        let norm = dna::normalize_seq(seq);
        let mut text: Vec<u8> = Vec::new();
        for &b in &norm {
            text.push(dna::to_alphabet(b));
        }
        let len = text.len() as u32;
        let contigs = vec![Contig {
            name: "chr1".to_string(),
            len,
            offset: 0,
        }];
        text.push(0);
        let sa_arr = sa::build_sa(&text);
        let bwt_arr = bwt::build_bwt(&text, &sa_arr);
        FMIndex::build(text, bwt_arr, sa_arr, contigs, dna::SIGMA as u8, 4)
    }

    #[test]
    fn collect_candidates_exact_match() {
        let reference = b"ACGTACGTACGTACGTACGTACGT";
        let fm = build_test_fm(reference);
        let read = b"ACGTACGTACGT";
        let norm = dna::normalize_seq(read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let sw = SwParams {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 16,
        };
        let mut candidates = Vec::new();
        let opt = default_opt();
        collect_candidates(&fm, &norm, &alpha, sw, false, &opt, &mut candidates);
        assert!(!candidates.is_empty());
        assert!(candidates[0].score > 0);
        assert!(candidates[0].cigar.contains('M'));
    }

    #[test]
    fn collect_candidates_with_mismatch() {
        let reference = b"ACGTACGTAGCTGATCGTAGCTAGCTAGCTGATCGTAGCTAGCTAGCTGAT";
        let fm = build_test_fm(reference);
        let mut read = reference[..40].to_vec();
        read[20] = if read[20] == b'A' { b'T' } else { b'A' };
        let norm = dna::normalize_seq(&read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let sw = SwParams {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 16,
        };
        let mut candidates = Vec::new();
        let opt = default_opt();
        collect_candidates(&fm, &norm, &alpha, sw, false, &opt, &mut candidates);
        assert!(!candidates.is_empty());
        assert!(candidates[0].score > 0);
    }

    #[test]
    fn collect_candidates_empty_query() {
        let fm = build_test_fm(b"ACGTACGTACGTACGTACGTACGT");
        let sw = SwParams {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 16,
        };
        let mut candidates = Vec::new();
        let opt = default_opt();
        collect_candidates(&fm, &[], &[], sw, false, &opt, &mut candidates);
        assert!(candidates.is_empty());
    }

    #[test]
    fn dedup_candidates_removes_duplicates() {
        let mut cands = vec![
            AlignCandidate {
                score: 50,
                is_rev: false,
                rname: "chr1".into(),
                pos1: 10,
                cigar: "20M".into(),
                nm: 0,
                contig_idx: 0,
            },
            AlignCandidate {
                score: 40,
                is_rev: false,
                rname: "chr1".into(),
                pos1: 10,
                cigar: "20M".into(),
                nm: 1,
                contig_idx: 0,
            },
            AlignCandidate {
                score: 45,
                is_rev: true,
                rname: "chr1".into(),
                pos1: 10,
                cigar: "20M".into(),
                nm: 0,
                contig_idx: 0,
            },
        ];
        dedup_candidates(&mut cands);
        assert_eq!(cands.len(), 2); // same pos+dir removed, different dir kept
    }

    #[test]
    fn dedup_candidates_keeps_all_unique() {
        let mut cands = vec![
            AlignCandidate {
                score: 50,
                is_rev: false,
                rname: "chr1".into(),
                pos1: 10,
                cigar: "20M".into(),
                nm: 0,
                contig_idx: 0,
            },
            AlignCandidate {
                score: 45,
                is_rev: false,
                rname: "chr1".into(),
                pos1: 20,
                cigar: "20M".into(),
                nm: 0,
                contig_idx: 0,
            },
            AlignCandidate {
                score: 40,
                is_rev: true,
                rname: "chr1".into(),
                pos1: 10,
                cigar: "20M".into(),
                nm: 0,
                contig_idx: 0,
            },
        ];
        dedup_candidates(&mut cands);
        assert_eq!(cands.len(), 3);
    }

    #[test]
    fn dedup_candidates_empty() {
        let mut cands: Vec<AlignCandidate> = vec![];
        dedup_candidates(&mut cands);
        assert!(cands.is_empty());
    }
}
