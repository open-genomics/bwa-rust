use std::collections::HashMap;

use crate::index::fm::Contig;
use crate::index::fm::FMIndex;
use crate::util::dna;

use super::extend::chain_to_alignment_buf_with_zdrop;
use super::extend::ChainAlignResult;
use super::seed::find_smem_seeds_with_max_occ;
use super::sw;
use super::{build_chains_with_limit, filter_chains};
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
    /// Reference sequence segment for MD:Z tag generation (oriented to match query strand)
    pub ref_seq: Vec<u8>,
    /// Query sequence segment (oriented to match the alignment strand)
    pub query_seq: Vec<u8>,
    /// Start position on the original query (0-based, forward strand)
    pub query_start: usize,
    /// End position on the original query (0-based, exclusive, forward strand)
    pub query_end: usize,
}

/// 从 FM 索引查找种子、构建链并执行 SW 对齐，将所有候选结果追加到 `candidates`。
///
/// - `query_norm`：归一化（大写 ACGTN）的 query 字节序列
/// - `query_alpha`：对应的字母表编码序列（`dna::to_alphabet`）
/// - `is_rev`：该 query 是否为反向互补链
/// - `original_query_len`：原始 query 长度（用于坐标转换）
/// - `opt`：比对参数（含 `min_seed_len`、`clip_penalty`、`max_occ` 等）
pub fn collect_candidates(
    fm: &FMIndex,
    query_norm: &[u8],
    query_alpha: &[u8],
    sw_params: SwParams,
    is_rev: bool,
    original_query_len: usize,
    opt: &AlignOpt,
    candidates: &mut Vec<AlignCandidate>,
) {
    let len = query_alpha.len();
    if len == 0 {
        return;
    }

    // BWA 风格：min_seed_len 默认 19，但不超过 read 长度的一半
    let min_mem_len = opt.min_seed_len.min(len / 2 + 1).max(1);
    let seeds = find_smem_seeds_with_max_occ(fm, query_alpha, min_mem_len, opt.max_occ);
    if seeds.is_empty() {
        return;
    }

    // 构建多条链
    let mut chains = build_chains_with_limit(&seeds, len, opt.max_chains_per_contig);
    // 过滤弱链：保留得分 >= 最佳得分 * 0.3 的链
    // 0.3 阈值来自 BWA 经验值，平衡保留多比对和过滤噪声
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

        let approx =
            chain_to_alignment_buf_with_zdrop(ch, query_norm, ref_seq.as_slice(), sw_params, opt.zdrop, &mut sw_buf);
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
            ref_seq.as_slice(),
            query_norm,
            original_query_len,
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
    ref_seq: &[u8],
    query_norm: &[u8],
    original_query_len: usize,
) -> AlignCandidate {
    // Extract the aligned reference segment for MD:Z tag generation
    // ref_offset is the window start, res.ref_start is the offset within the window
    let abs_ref_start = ref_offset + res.ref_start;
    // Calculate reference length consumed by CIGAR
    let ref_len = cigar_ref_length(&res.cigar);
    let ref_segment = if abs_ref_start + ref_len <= ref_seq.len() {
        ref_seq[abs_ref_start..abs_ref_start + ref_len].to_vec()
    } else {
        Vec::new()
    };

    let query_len = cigar_query_length(&res.cigar);
    let query_segment = if query_len <= query_norm.len() {
        query_norm[..query_len].to_vec()
    } else {
        Vec::new()
    };

    // Convert coordinates to original query (forward strand) coordinates
    // If is_rev is true, query_norm is the reverse complement, so we need to map coordinates
    let (query_start, query_end) = if is_rev {
        // For reverse complement alignment:
        // res.query_start/end are on the revcomp sequence
        // Map back to original: orig_start = len - revcomp_end, orig_end = len - revcomp_start
        let orig_start = original_query_len.saturating_sub(res.query_end);
        let orig_end = original_query_len.saturating_sub(res.query_start);
        (orig_start, orig_end)
    } else {
        (res.query_start, res.query_end)
    };

    AlignCandidate {
        score: res.score,
        sort_score: effective_score(res.score, &res.cigar, clip_penalty),
        is_rev,
        rname: contig.name.clone(),
        pos1: (ref_offset + res.ref_start) as u32 + 1,
        cigar: res.cigar.clone(),
        nm: res.nm,
        contig_idx,
        ref_seq: ref_segment,
        query_seq: query_segment,
        query_start,
        query_end,
    }
}

/// Calculate the reference length consumed by a CIGAR string.
fn cigar_ref_length(cigar: &str) -> usize {
    sw::parse_cigar(cigar)
        .into_iter()
        .filter_map(|(op, len)| match op {
            'M' | '=' | 'X' | 'D' | 'N' => Some(len),
            _ => None,
        })
        .sum()
}

fn cigar_query_length(cigar: &str) -> usize {
    sw::parse_cigar(cigar)
        .into_iter()
        .filter_map(|(op, len)| match op {
            'M' | '=' | 'X' | 'I' | 'S' => Some(len),
            _ => None,
        })
        .sum()
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
        collect_candidates(&fm, &norm, &alpha, sw, false, norm.len(), &opt, &mut candidates);
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
        collect_candidates(&fm, &norm, &alpha, sw, false, norm.len(), &opt, &mut candidates);
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
        collect_candidates(&fm, &[], &[], sw, false, 0, &opt, &mut candidates);
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
                ref_seq: Vec::new(),
                query_seq: Vec::new(),
                query_start: 0,
                query_end: 20,
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
                ref_seq: Vec::new(),
                query_seq: Vec::new(),
                query_start: 0,
                query_end: 20,
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
                ref_seq: Vec::new(),
                query_seq: Vec::new(),
                query_start: 0,
                query_end: 20,
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
                ref_seq: Vec::new(),
                query_seq: Vec::new(),
                query_start: 0,
                query_end: 20,
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
                ref_seq: Vec::new(),
                query_seq: Vec::new(),
                query_start: 20,
                query_end: 40,
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
                ref_seq: Vec::new(),
                query_seq: Vec::new(),
                query_start: 0,
                query_end: 20,
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

    #[test]
    fn build_candidate_keeps_full_query_for_soft_clipped_md_tag() {
        let contig = Contig {
            name: "chr1".to_string(),
            len: 4,
            offset: 0,
        };
        let res = ChainAlignResult {
            score: 8,
            cigar: "2S4M2S".to_string(),
            nm: 0,
            query_start: 2,
            query_end: 6,
            ref_start: 0,
            ref_end: 4,
        };

        let cand = build_candidate(&contig, 0, false, &res, 0, 1, b"ACGT", b"NNACGTNN", 8);

        assert_eq!(cand.query_seq, b"NNACGTNN");
        assert_eq!(
            crate::io::sam::generate_md_tag(&cand.ref_seq, &cand.query_seq, &cand.cigar),
            "4"
        );
    }
}
