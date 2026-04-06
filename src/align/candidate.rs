use std::collections::HashMap;

use crate::index::fm::Contig;
use crate::index::fm::FMIndex;
use crate::util::dna;

use super::extend::chain_to_alignment_buf;
use super::extend::ChainAlignResult;
use super::sw;
use super::{build_chains, filter_chains, find_smem_seeds};
use super::{AlignOpt, SwParams};

#[derive(Debug, Clone)]
pub struct AlignCandidate {
    pub score: i32,
    pub sort_score: i32,
    pub is_rev: bool,
    pub rname: String,
    pub pos1: u32,
    pub cigar: String,
    pub nm: u32,
    pub contig_idx: usize,
}

/// 从 FM 索引查找种子、构建链并执行 SW 对齐，将所有候选结果追加到 `candidates`。
///
/// - `query_norm`：归一化（大写 ACGTN）的 query 字节序列
/// - `query_alpha`：对应的字母表编码序列（`dna::to_alphabet`）
/// - `is_rev`：该 query 是否为反向互补链
/// - `opt`：比对参数（含 `min_seed_len`、`clip_penalty` 等）
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
    let mut refine_buf = sw::SwBuffer::new();
    let mut ref_cache: HashMap<usize, Vec<u8>> = HashMap::new();

    for ch in &chains {
        let ci = ch.contig;
        let contig = &fm.contigs[ci];
        let ref_seq = ref_cache.entry(ci).or_insert_with(|| {
            let offset = contig.offset as usize;
            let contig_len = contig.len as usize;
            fm.text[offset..offset + contig_len]
                .iter()
                .map(|&code| dna::from_alphabet(code))
                .collect()
        });
        if ref_seq.is_empty() {
            continue;
        }

        let approx = chain_to_alignment_buf(ch, query_norm, ref_seq.as_slice(), sw_params, &mut sw_buf);
        let refined = refine_candidate_alignment(ch, query_norm, ref_seq.as_slice(), sw_params, &mut refine_buf);
        let (ref_offset, selected) = choose_alignment(approx, refined, opt.clip_penalty);

        if selected.score <= 0 || selected.cigar.is_empty() {
            continue;
        }

        candidates.push(build_candidate(
            contig,
            ci,
            is_rev,
            &selected,
            ref_offset,
            opt.clip_penalty,
        ));
    }
}

fn refine_candidate_alignment(
    chain: &super::chain::Chain,
    query_norm: &[u8],
    reference: &[u8],
    sw_params: SwParams,
    sw_buf: &mut sw::SwBuffer,
) -> Option<(usize, ChainAlignResult)> {
    if chain.seeds.is_empty() || query_norm.is_empty() || reference.is_empty() {
        return None;
    }

    let seed_start = chain.seeds.iter().map(|s| s.rb as usize).min()?;
    let seed_end = chain.seeds.iter().map(|s| s.re as usize).max()?;
    let pad = query_norm.len() + sw_params.band_width + 16;
    let window_start = seed_start.saturating_sub(pad);
    let window_end = (seed_end + pad).min(reference.len());
    if window_start >= window_end {
        return None;
    }

    let res = sw::semiglobal_align_with_buf(query_norm, &reference[window_start..window_end], sw_params, sw_buf);
    if res.score <= 0 || res.cigar.is_empty() {
        return None;
    }

    Some((
        window_start,
        ChainAlignResult {
            score: res.score,
            cigar: res.cigar,
            nm: res.nm,
            query_start: res.query_start,
            query_end: res.query_end,
            ref_start: res.ref_start,
            ref_end: res.ref_end,
        },
    ))
}

fn choose_alignment(
    approx: ChainAlignResult,
    refined: Option<(usize, ChainAlignResult)>,
    clip_penalty: i32,
) -> (usize, ChainAlignResult) {
    let approx_rank = effective_score(approx.score, &approx.cigar, clip_penalty);
    let Some((window_offset, refined)) = refined else {
        return (0, approx);
    };
    let refined_rank = effective_score(refined.score, &refined.cigar, clip_penalty);

    if refined_rank > approx_rank
        || (refined_rank == approx_rank && refined.score > approx.score)
        || (refined_rank == approx_rank && refined.score == approx.score && refined.nm < approx.nm)
    {
        (window_offset, refined)
    } else {
        (0, approx)
    }
}

fn build_candidate(
    contig: &Contig,
    contig_idx: usize,
    is_rev: bool,
    res: &ChainAlignResult,
    ref_offset: usize,
    clip_penalty: i32,
) -> AlignCandidate {
    AlignCandidate {
        score: res.score,
        sort_score: effective_score(res.score, &res.cigar, clip_penalty),
        is_rev,
        rname: contig.name.clone(),
        pos1: (ref_offset + res.ref_start) as u32 + 1,
        cigar: res.cigar.clone(),
        nm: res.nm,
        contig_idx,
    }
}

fn effective_score(score: i32, cigar: &str, clip_penalty: i32) -> i32 {
    score - soft_clipped_bases(cigar) as i32 * clip_penalty
}

fn soft_clipped_bases(cigar: &str) -> usize {
    sw::parse_cigar(cigar)
        .into_iter()
        .filter_map(|(op, len)| if op == 'S' { Some(len) } else { None })
        .sum()
}

/// 对已按得分排序的候选列表进行原地去重：
/// 相同 contig、相同位置（`pos1`）、相同方向（`is_rev`）的候选只保留得分最高的一条（即第一条）。
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
    use crate::testutil::build_test_fm;

    fn default_opt() -> AlignOpt {
        AlignOpt::default()
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
                sort_score: 50,
                is_rev: false,
                rname: "chr1".into(),
                pos1: 10,
                cigar: "20M".into(),
                nm: 0,
                contig_idx: 0,
            },
            AlignCandidate {
                score: 40,
                sort_score: 40,
                is_rev: false,
                rname: "chr1".into(),
                pos1: 10,
                cigar: "20M".into(),
                nm: 1,
                contig_idx: 0,
            },
            AlignCandidate {
                score: 45,
                sort_score: 45,
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
                sort_score: 50,
                is_rev: false,
                rname: "chr1".into(),
                pos1: 10,
                cigar: "20M".into(),
                nm: 0,
                contig_idx: 0,
            },
            AlignCandidate {
                score: 45,
                sort_score: 45,
                is_rev: false,
                rname: "chr1".into(),
                pos1: 20,
                cigar: "20M".into(),
                nm: 0,
                contig_idx: 0,
            },
            AlignCandidate {
                score: 40,
                sort_score: 40,
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

    #[test]
    fn effective_score_penalizes_soft_clipping() {
        assert_eq!(effective_score(16, "5S16M", 1), 11);
        assert_eq!(effective_score(13, "4M1I16M", 1), 13);
    }
}
