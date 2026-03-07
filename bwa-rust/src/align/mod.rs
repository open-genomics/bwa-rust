pub mod seed;
pub mod chain;
pub mod sw;
pub mod pipeline;

pub use sw::{SwParams, SwResult, banded_sw};
pub use seed::{MemSeed, AlnReg, find_smem_seeds, find_mem_seeds};
pub use chain::{Chain, best_chain, build_chains, filter_chains};
pub use pipeline::{align_fastq_with_opt, align_fastq_with_fm_opt};

#[derive(Clone, Copy, Debug)]
pub struct AlignOpt {
    pub match_score: i32,
    pub mismatch_penalty: i32,
    pub gap_open: i32,
    pub gap_extend: i32,
    pub band_width: usize,
    pub score_threshold: i32,
    pub min_seed_len: usize,
    pub threads: usize,
}

impl Default for AlignOpt {
    fn default() -> Self {
        Self {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 16,
            score_threshold: 20,
            min_seed_len: 19,
            threads: 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChainAlignResult {
    pub score: i32,
    pub cigar: String,
    pub nm: u32,
    pub query_start: usize,
    pub query_end: usize,
    pub ref_start: usize,
    pub ref_end: usize,
}

pub fn chain_to_alignment(
    chain: &Chain,
    query: &[u8],
    reference: &[u8],
    p: SwParams,
) -> ChainAlignResult {
    chain_to_alignment_buf(chain, query, reference, p, &mut sw::SwBuffer::new())
}

pub fn chain_to_alignment_buf(
    chain: &Chain,
    query: &[u8],
    reference: &[u8],
    p: SwParams,
    buf: &mut sw::SwBuffer,
) -> ChainAlignResult {
    if chain.seeds.is_empty() {
        return ChainAlignResult {
            score: 0,
            cigar: String::new(),
            nm: 0,
            query_start: 0,
            query_end: 0,
            ref_start: 0,
            ref_end: 0,
        };
    }

    let mut seeds = chain.seeds.clone();
    seeds.sort_by_key(|s| (s.qb, s.rb));

    let mut ops: Vec<(char, usize)> = Vec::new();
    let mut total_score: i32 = 0;
    let mut total_nm: u32 = 0;
    let zdrop = 100;

    let first_seed = &seeds[0];
    let last_seed = &seeds[seeds.len() - 1];

    let mut query_start = first_seed.qb;
    let mut ref_start = first_seed.rb as usize;
    let mut query_end = last_seed.qe;
    let mut ref_end = last_seed.re as usize;

    if first_seed.qb > 0 && first_seed.rb > 0 {
        let left_q = &query[..first_seed.qb];
        let ref_left_end = first_seed.rb as usize;
        let ref_left_span = (left_q.len() + p.band_width + 32).min(ref_left_end);
        let ref_left_start = ref_left_end - ref_left_span;
        let left_r = &reference[ref_left_start..ref_left_end];
        let left_ext = sw::extend_left(left_q, left_r, p, zdrop);
        if left_ext.score > 0 && !left_ext.ops.is_empty() {
            push_char_ops(&mut ops, &left_ext.ops);
            total_score += left_ext.score;
            total_nm += nm_from_ops(
                &left_ext.ops,
                &left_q[left_q.len() - left_ext.query_len..],
                &left_r[left_r.len() - left_ext.ref_len..],
            );
            query_start = first_seed.qb - left_ext.query_len;
            ref_start = ref_left_end - left_ext.ref_len;
        }
    }

    let k = seeds.len();
    for idx in 0..k {
        if idx > 0 {
            let prev_seed = &seeds[idx - 1];
            let curr = &seeds[idx];
            let q_gap_start = prev_seed.qe;
            let q_gap_end = curr.qb;
            let r_gap_start = prev_seed.re as usize;
            let r_gap_end = curr.rb as usize;
            let q_gap_len = q_gap_end.saturating_sub(q_gap_start);
            let r_gap_len = r_gap_end.saturating_sub(r_gap_start);
            if q_gap_len > 0 || r_gap_len > 0 {
                if q_gap_len > 0 && r_gap_len > 0 {
                    if q_gap_end <= query.len() && r_gap_end <= reference.len() {
                        let q_gap = &query[q_gap_start..q_gap_end];
                        let r_gap = &reference[r_gap_start..r_gap_end];
                        let res = sw::banded_sw_with_buf(q_gap, r_gap, p, buf);
                        if res.score > 0 && !res.cigar.is_empty() {
                            let parsed = sw::parse_cigar(&res.cigar);
                            for (op_ch, num) in parsed {
                                push_run(&mut ops, op_ch, num);
                            }
                            total_score += res.score;
                            total_nm += res.nm;
                        }
                    }
                } else if q_gap_len > 0 {
                    push_run(&mut ops, 'I', q_gap_len);
                    total_score -= p.gap_open + p.gap_extend * q_gap_len as i32;
                    total_nm += q_gap_len as u32;
                } else {
                    push_run(&mut ops, 'D', r_gap_len);
                    total_score -= p.gap_open + p.gap_extend * r_gap_len as i32;
                    total_nm += r_gap_len as u32;
                }
            }
        }

        let s = &seeds[idx];
        let len = s.qe - s.qb;
        if len > 0 {
            push_run(&mut ops, 'M', len);
            total_score += (len as i32) * p.match_score;
        }
    }

    if last_seed.qe < query.len() && (last_seed.re as usize) < reference.len() {
        let right_q = &query[last_seed.qe..];
        let ref_right_start = last_seed.re as usize;
        let ref_right_end = (ref_right_start + right_q.len() + p.band_width + 32).min(reference.len());
        let right_r = &reference[ref_right_start..ref_right_end];
        let right_ext = sw::extend_right(right_q, right_r, p, zdrop);
        if right_ext.score > 0 && !right_ext.ops.is_empty() {
            push_char_ops(&mut ops, &right_ext.ops);
            total_score += right_ext.score;
            total_nm += nm_from_ops(
                &right_ext.ops,
                &right_q[..right_ext.query_len],
                &right_r[..right_ext.ref_len],
            );
            query_end = last_seed.qe + right_ext.query_len;
            ref_end = ref_right_start + right_ext.ref_len;
        }
    }

    if query_start > 0 {
        ops.insert(0, ('S', query_start));
    }
    let right_clip = query.len().saturating_sub(query_end);
    if right_clip > 0 {
        push_run(&mut ops, 'S', right_clip);
    }

    let mut cigar = String::new();
    for (op, len) in ops {
        use std::fmt::Write as _;
        let _ = write!(&mut cigar, "{}{}", len, op);
    }

    ChainAlignResult {
        score: total_score,
        cigar,
        nm: total_nm,
        query_start,
        query_end,
        ref_start,
        ref_end,
    }
}

fn push_run(ops: &mut Vec<(char, usize)>, op: char, len: usize) {
    if len == 0 {
        return;
    }
    if let Some(last) = ops.last_mut() {
        if last.0 == op {
            last.1 += len;
            return;
        }
    }
    ops.push((op, len));
}

fn push_char_ops(ops: &mut Vec<(char, usize)>, chars: &[char]) {
    for &op in chars {
        push_run(ops, op, 1);
    }
}

fn nm_from_ops(ops: &[char], query: &[u8], reference: &[u8]) -> u32 {
    let mut qi = 0usize;
    let mut ri = 0usize;
    let mut nm = 0u32;
    for &op in ops {
        match op {
            'M' => {
                if qi < query.len() && ri < reference.len() && query[qi] != reference[ri] {
                    nm += 1;
                }
                qi += 1;
                ri += 1;
            }
            'I' => {
                nm += 1;
                qi += 1;
            }
            'D' => {
                nm += 1;
                ri += 1;
            }
            _ => {}
        }
    }
    nm
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> SwParams {
        SwParams {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 1,
            gap_extend: 0,
            band_width: 8,
        }
    }

    #[test]
    fn chain_to_alignment_single_seed() {
        let p = default_params();
        let chain = Chain {
            contig: 0,
            seeds: vec![MemSeed { contig: 0, qb: 0, qe: 4, rb: 0, re: 4 }],
            score: 4,
        };
        let res = chain_to_alignment(&chain, b"ACGT", b"ACGT", p);
        assert_eq!(res.score, 8);
        assert_eq!(res.cigar, "4M");
        assert_eq!(res.nm, 0);
    }

    #[test]
    fn chain_to_alignment_empty_chain() {
        let p = default_params();
        let chain = Chain { contig: 0, seeds: vec![], score: 0 };
        let res = chain_to_alignment(&chain, b"ACGT", b"ACGT", p);
        assert_eq!(res.score, 0);
        assert!(res.cigar.is_empty());
    }
}
