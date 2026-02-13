use anyhow::Result;
use std::io::Write;

use crate::index::fm::FMIndex;
use crate::io::fastq::FastqReader;
use crate::util::dna;

const NEG_INF: i32 = i32::MIN / 4;

#[derive(Clone, Copy, Debug)]
pub struct SwParams {
    pub match_score: i32,
    pub mismatch_penalty: i32,
    pub gap_open: i32,
    pub gap_extend: i32,
    pub band_width: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct AlignOpt {
    pub match_score: i32,
    pub mismatch_penalty: i32,
    pub gap_open: i32,
    pub gap_extend: i32,
    pub band_width: usize,
    pub score_threshold: i32,
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
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SwResult {
    pub score: i32,
    pub query_start: usize,
    pub query_end: usize,
    pub ref_start: usize,
    pub ref_end: usize,
    pub cigar: String,
    pub nm: u32,
}

pub fn banded_sw(query: &[u8], reference: &[u8], p: SwParams) -> SwResult {
    let m = query.len();
    let n = reference.len();

    if m == 0 || n == 0 {
        return SwResult {
            score: 0,
            query_start: 0,
            query_end: 0,
            ref_start: 0,
            ref_end: 0,
            cigar: String::new(),
            nm: 0,
        };
    }

    let rows = m + 1;
    let cols = n + 1;
    let size = rows * cols;

    let mut h = vec![0i32; size];
    let mut e = vec![NEG_INF; size];
    let mut f = vec![NEG_INF; size];

    let band = p.band_width as isize;

    let mut best_score = 0i32;
    let mut best_i = 0usize;
    let mut best_j = 0usize;

    for i in 1..=m {
        let i_isize = i as isize;
        let mut j_start = 1usize;
        let mut j_end = n;
        if band >= 0 {
            let js = i_isize - band;
            let je = i_isize + band;
            if js > 1 {
                j_start = js as usize;
            }
            if je < n as isize {
                j_end = je as usize;
            }
        }
        if j_start > j_end {
            continue;
        }

        for j in j_start..=j_end {
            let idx = i * cols + j;
            let up_idx = (i - 1) * cols + j;
            let left_idx = i * cols + (j - 1);
            let diag_idx = (i - 1) * cols + (j - 1);

            // affine gap: E = gap from up (deletion)
            let e_open = h[up_idx] - p.gap_open - p.gap_extend;
            let e_ext = e[up_idx] - p.gap_extend;
            e[idx] = e_open.max(e_ext);

            // affine gap: F = gap from left (insertion)
            let f_open = h[left_idx] - p.gap_open - p.gap_extend;
            let f_ext = f[left_idx] - p.gap_extend;
            f[idx] = f_open.max(f_ext);

            let subst = if query[i - 1] == reference[j - 1] {
                p.match_score
            } else {
                -p.mismatch_penalty
            };

            let mut val = h[diag_idx] + subst;
            if e[idx] > val {
                val = e[idx];
            }
            if f[idx] > val {
                val = f[idx];
            }
            if val < 0 {
                val = 0;
            }
            h[idx] = val;

            if val > best_score {
                best_score = val;
                best_i = i;
                best_j = j;
            }
        }
    }

    if best_score <= 0 {
        return SwResult {
            score: 0,
            query_start: 0,
            query_end: 0,
            ref_start: 0,
            ref_end: 0,
            cigar: String::new(),
            nm: 0,
        };
    }

    // backtrack from best cell
    let mut ops: Vec<char> = Vec::new();
    let mut i = best_i;
    let mut j = best_j;

    while i > 0 && j > 0 {
        let idx = i * cols + j;
        let h_here = h[idx];
        if h_here == 0 {
            break;
        }

        let diag_idx = (i - 1) * cols + (j - 1);
        let _up_idx = (i - 1) * cols + j;
        let _left_idx = i * cols + (j - 1);

        let subst = if query[i - 1] == reference[j - 1] {
            p.match_score
        } else {
            -p.mismatch_penalty
        };

        let diag_val = h[diag_idx] + subst;
        let e_val = e[idx];
        let f_val = f[idx];

        if h_here == diag_val {
            ops.push('M');
            i -= 1;
            j -= 1;
        } else if h_here == e_val {
            ops.push('I');
            i -= 1;
        } else if h_here == f_val {
            ops.push('D');
            j -= 1;
        } else {
            break;
        }
    }

    let query_start = i;
    let ref_start = j;
    let query_end = best_i;
    let ref_end = best_j;

    ops.reverse();

    let mut nm = 0u32;
    let mut qi = query_start;
    let mut rj = ref_start;
    for &op in &ops {
        match op {
            'M' => {
                if query[qi] != reference[rj] {
                    nm += 1;
                }
                qi += 1;
                rj += 1;
            }
            'I' => {
                nm += 1;
                qi += 1;
            }
            'D' => {
                nm += 1;
                rj += 1;
            }
            _ => {}
        }
    }

    let mut cigar = String::new();
    if !ops.is_empty() {
        let mut cur = ops[0];
        let mut len = 1usize;
        for &op in &ops[1..] {
            if op == cur {
                len += 1;
            } else {
                use std::fmt::Write as _;
                let _ = write!(&mut cigar, "{}{}", len, cur);
                cur = op;
                len = 1;
            }
        }
        use std::fmt::Write as _;
        let _ = write!(&mut cigar, "{}{}", len, cur);
    }

    SwResult {
        score: best_score,
        query_start,
        query_end,
        ref_start,
        ref_end,
        cigar,
        nm,
    }
}


pub fn align_fastq_with_opt(
    index_path: &str,
    fastq_path: &str,
    out_path: Option<&str>,
    opt: AlignOpt,
) -> Result<()> {
    // load FM index
    let fm = FMIndex::load_from_file(index_path)?;

    // open FASTQ
    let fq = std::fs::File::open(fastq_path)?;
    let mut reader = FastqReader::new(std::io::BufReader::new(fq));

    // writer
    let mut out_box: Box<dyn Write> = if let Some(p) = out_path {
        Box::new(std::io::BufWriter::new(std::fs::File::create(p)?))
    } else {
        Box::new(std::io::BufWriter::new(std::io::stdout()))
    };

    // SAM header (minimal)
    for c in &fm.contigs {
        writeln!(out_box, "@SQ\tSN:{}\tLN:{}", c.name, c.len)?;
    }

    // 临时固定的一组 SW 参数（后续可由 CLI 传入）
    let sw_params = SwParams {
        match_score: opt.match_score,
        mismatch_penalty: opt.mismatch_penalty,
        gap_open: opt.gap_open,
        gap_extend: opt.gap_extend,
        band_width: opt.band_width,
    };

    // iterate reads
    while let Some(rec) = reader.next_record()? {
        let qname = &rec.id;
        let seq = &rec.seq;
        let qual = &rec.qual;

        if seq.is_empty() {
            // treat empty read as unmapped
            let flag = 4;
            writeln!(
                out_box,
                "{}\t{}\t*\t0\t0\t*\t*\t0\t0\t{}\t{}",
                qname,
                flag,
                String::from_utf8_lossy(seq),
                String::from_utf8_lossy(qual),
            )?;
            continue;
        }

        // prepare forward
        let fwd_norm = dna::normalize_seq(seq);
        let fwd_alpha: Vec<u8> = fwd_norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        // prepare reverse complement
        let rev_seq = dna::revcomp(seq);
        let rev_norm = dna::normalize_seq(&rev_seq);
        let rev_alpha: Vec<u8> = rev_norm.iter().map(|&b| dna::to_alphabet(b)).collect();

        let fwd_res = align_one_direction(&fm, &fwd_norm, &fwd_alpha, sw_params);
        let rev_res = align_one_direction(&fm, &rev_norm, &rev_alpha, sw_params);

        let mut has_best = false;
        let mut best_is_rev = false;
        let mut best_ci = 0usize;
        let mut best_pos = 0u32;
        let mut best_cigar = String::new();
        let mut best_score = 0i32;
        let mut best_nm: u32 = 0;
        let mut second_best_score = 0i32;

        match (fwd_res, rev_res) {
            (None, None) => {}
            (Some(f), None) => {
                best_is_rev = false;
                best_ci = f.best_ci;
                best_pos = f.best_pos;
                best_cigar = f.best_cigar;
                best_score = f.best_score;
                best_nm = f.best_nm;
                second_best_score = f.second_best_score;
                has_best = true;
            }
            (None, Some(r)) => {
                best_is_rev = true;
                best_ci = r.best_ci;
                best_pos = r.best_pos;
                best_cigar = r.best_cigar;
                best_score = r.best_score;
                best_nm = r.best_nm;
                second_best_score = r.second_best_score;
                has_best = true;
            }
            (Some(f), Some(r)) => {
                if f.best_score >= r.best_score {
                    best_is_rev = false;
                    best_ci = f.best_ci;
                    best_pos = f.best_pos;
                    best_cigar = f.best_cigar;
                    best_score = f.best_score;
                    best_nm = f.best_nm;
                    second_best_score = r.best_score;
                    if f.second_best_score > second_best_score {
                        second_best_score = f.second_best_score;
                    }
                } else {
                    best_is_rev = true;
                    best_ci = r.best_ci;
                    best_pos = r.best_pos;
                    best_cigar = r.best_cigar;
                    best_score = r.best_score;
                    best_nm = r.best_nm;
                    second_best_score = f.best_score;
                    if r.second_best_score > second_best_score {
                        second_best_score = r.second_best_score;
                    }
                }
                has_best = true;
            }
        }

        if has_best && best_score >= opt.score_threshold {
            let contig = &fm.contigs[best_ci];
            let flag = if best_is_rev { 16 } else { 0 };
            let rname = &contig.name;
            let pos1 = best_pos + 1;
            let mapq = compute_mapq(best_score, second_best_score);
            let seq_str = String::from_utf8_lossy(seq);
            let qual_str = String::from_utf8_lossy(qual);
            writeln!(
                out_box,
                "{}\t{}\t{}\t{}\t{}\t{}\t*\t0\t0\t{}\t{}\tAS:i:{}\tXS:i:{}\tNM:i:{}",
                qname,
                flag,
                rname,
                pos1,
                mapq,
                best_cigar,
                seq_str,
                qual_str,
                best_score,
                second_best_score,
                best_nm,
            )?;
        } else {
            let flag = 4;
            writeln!(
                out_box,
                "{}\t{}\t*\t0\t0\t*\t*\t0\t0\t{}\t{}",
                qname,
                flag,
                String::from_utf8_lossy(seq),
                String::from_utf8_lossy(qual),
            )?;
        }
    }

    Ok(())
}


fn compute_mapq(best_score: i32, second_best_score: i32) -> u8 {
    if best_score <= 0 {
        return 0;
    }
    let diff = (best_score - second_best_score).max(0) as i64;
    let denom = best_score as i64;
    if denom <= 0 {
        return 0;
    }
    let mut q = (diff * 60 / denom) as i32;
    if q < 0 {
        q = 0;
    }
    if q > 60 {
        q = 60;
    }
    q as u8
}

fn align_one_direction(
    fm: &FMIndex,
    query_norm: &[u8],
    query_alpha: &[u8],
    sw_params: SwParams,
) -> Option<DirectionBest> {
    let len = query_alpha.len();
    if len == 0 {
        return None;
    }

    // 使用 MEM/链 进行候选生成
    let min_mem_len = len.min(20).max(1);
    let seeds = find_mem_seeds(fm, query_alpha, min_mem_len);
    if seeds.is_empty() {
        return None;
    }

    let mut best: Option<DirectionBest> = None;
    let mut second_best_score = 0i32;

    for (ci, contig) in fm.contigs.iter().enumerate() {
        let seeds_ci: Vec<MemSeed> = seeds
            .iter()
            .filter(|s| s.contig == ci)
            .cloned()
            .collect();
        if seeds_ci.is_empty() {
            continue;
        }

        if let Some(chain) = best_chain(&seeds_ci, len) {
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

            let res = chain_to_alignment(&chain, query_norm, &ref_seq, sw_params);
            if res.score <= 0 || res.cigar.is_empty() {
                continue;
            }

            let rb_min = chain
                .seeds
                .iter()
                .map(|s| s.rb)
                .min()
                .unwrap_or(0);

            let cand_score = res.score;
            if let Some(ref mut b) = best {
                if cand_score > b.best_score {
                    if b.best_score > second_best_score {
                        second_best_score = b.best_score;
                    }
                    *b = DirectionBest {
                        best_score: cand_score,
                        best_ci: ci,
                        best_pos: rb_min,
                        best_cigar: res.cigar,
                        best_nm: res.nm,
                        second_best_score,
                    };
                } else if cand_score > second_best_score {
                    second_best_score = cand_score;
                }
            } else {
                best = Some(DirectionBest {
                    best_score: cand_score,
                    best_ci: ci,
                    best_pos: rb_min,
                    best_cigar: res.cigar,
                    best_nm: res.nm,
                    second_best_score: 0,
                });
            }
        }
    }

    if let Some(mut b) = best {
        b.second_best_score = second_best_score;
        Some(b)
    } else {
        None
    }
}

#[derive(Debug)]
struct DirectionBest {
    best_score: i32,
    best_ci: usize,
    best_pos: u32,
    best_cigar: String,
    best_nm: u32,
    second_best_score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemSeed {
    pub contig: usize,
    pub qb: usize,
    pub qe: usize,
    pub rb: u32,
    pub re: u32,
}

pub fn find_mem_seeds(
    fm: &FMIndex,
    query_alpha: &[u8],
    min_len: usize,
) -> Vec<MemSeed> {
    let n = query_alpha.len();
    if min_len == 0 || n == 0 || min_len > n {
        return Vec::new();
    }

    let mut seeds = Vec::new();

    for qb in 0..n {
        if qb + min_len > n {
            break;
        }

        let mut best_len = 0usize;
        let mut best_l = 0usize;
        let mut best_r = 0usize;

        let mut len = min_len;
        while qb + len <= n {
            let pat = &query_alpha[qb..qb + len];
            match fm.backward_search(pat) {
                Some((l, r)) if l < r => {
                    best_len = len;
                    best_l = l;
                    best_r = r;
                    len += 1;
                }
                _ => break,
            }
        }

        if best_len >= min_len {
            for &pos in fm.sa_interval_positions(best_l, best_r) {
                if let Some((ci, off)) = fm.map_text_pos(pos) {
                    let rb = off;
                    let re = off + best_len as u32;
                    seeds.push(MemSeed {
                        contig: ci,
                        qb,
                        qe: qb + best_len,
                        rb,
                        re,
                    });
                }
            }
        }
    }

    seeds
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain {
    pub contig: usize,
    pub seeds: Vec<MemSeed>,
    pub score: u32,
}

pub fn best_chain(seeds: &[MemSeed], max_gap: usize) -> Option<Chain> {
    if seeds.is_empty() {
        return None;
    }

    let mut idxs: Vec<usize> = (0..seeds.len()).collect();
    idxs.sort_by_key(|&i| {
        let s = &seeds[i];
        (s.contig, s.qb, s.rb)
    });

    let n = idxs.len();
    let mut dp: Vec<u32> = vec![0; n];
    let mut prev: Vec<Option<usize>> = vec![None; n];
    let mut best_i: Option<usize> = None;

    for (t, &i) in idxs.iter().enumerate() {
        let si = &seeds[i];
        let len_i = (si.qe - si.qb) as u32;
        dp[t] = len_i;

        for (u, &j) in idxs[..t].iter().enumerate() {
            let sj = &seeds[j];
            if sj.contig != si.contig {
                continue;
            }
            if sj.qe > si.qb {
                continue;
            }
            if sj.re > si.rb {
                continue;
            }
            let gap_q = si.qb - sj.qe;
            let gap_r = (si.rb - sj.re) as usize;
            if gap_q > max_gap || gap_r > max_gap {
                continue;
            }
            let cand = dp[u] + len_i;
            if cand > dp[t] {
                dp[t] = cand;
                prev[t] = Some(u);
            }
        }

        if best_i
            .map(|bi| dp[t] > dp[bi])
            .unwrap_or(true)
        {
            best_i = Some(t);
        }
    }

    let best_t = best_i?;
    let mut chain_idxs: Vec<usize> = Vec::new();
    let mut cur = Some(best_t);
    while let Some(t) = cur {
        chain_idxs.push(idxs[t]);
        cur = prev[t];
    }
    chain_idxs.reverse();

    let contig = seeds[chain_idxs[0]].contig;
    let seeds_vec: Vec<MemSeed> = chain_idxs.into_iter().map(|i| seeds[i].clone()).collect();
    let score = dp[best_t];

    Some(Chain {
        contig,
        seeds: seeds_vec,
        score,
    })
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChainAlignResult {
    pub score: i32,
    pub cigar: String,
    pub nm: u32,
}

pub fn chain_to_alignment(
    chain: &Chain,
    query: &[u8],
    reference: &[u8],
    p: SwParams,
) -> ChainAlignResult {
    if chain.seeds.is_empty() {
        return ChainAlignResult {
            score: 0,
            cigar: String::new(),
            nm: 0,
        };
    }

    let mut seeds = chain.seeds.clone();
    seeds.sort_by_key(|s| (s.qb, s.rb));

    let mut ops: Vec<(char, usize)> = Vec::new();
    let mut total_score: i32 = 0;
    let mut total_nm: u32 = 0;

    let k = seeds.len();
    for idx in 0..k {
        if idx > 0 {
            let prev = &seeds[idx - 1];
            let curr = &seeds[idx];
            let q_gap_start = prev.qe;
            let q_gap_end = curr.qb;
            let r_gap_start = prev.re as usize;
            let r_gap_end = curr.rb as usize;
            if q_gap_end > q_gap_start && r_gap_end > r_gap_start {
                let q_gap = &query[q_gap_start..q_gap_end];
                let r_gap = &reference[r_gap_start..r_gap_end];
                let res = banded_sw(q_gap, r_gap, p);
                if res.score > 0 && !res.cigar.is_empty() {
                    let mut num = 0usize;
                    for ch in res.cigar.chars() {
                        if ch.is_ascii_digit() {
                            num = num * 10 + (ch as usize - '0' as usize);
                        } else {
                            if num > 0 {
                                let op_ch = ch;
                                if let Some(last) = ops.last_mut() {
                                    if last.0 == op_ch {
                                        last.1 += num;
                                    } else {
                                        ops.push((op_ch, num));
                                    }
                                } else {
                                    ops.push((op_ch, num));
                                }
                                num = 0;
                            }
                        }
                    }
                    total_score += res.score;
                    total_nm += res.nm;
                }
            }
        }

        let s = &seeds[idx];
        let len = s.qe - s.qb;
        if len > 0 {
            if let Some(last) = ops.last_mut() {
                if last.0 == 'M' {
                    last.1 += len;
                } else {
                    ops.push(('M', len));
                }
            } else {
                ops.push(('M', len));
            }
            total_score += (len as i32) * p.match_score;
        }
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{bwt, sa};
    use crate::index::fm::{Contig, FMIndex};
    use crate::util::dna;

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
    fn sw_perfect_match() {
        let p = default_params();
        let q = b"ACGT";
        let r = b"ACGT";
        let res = banded_sw(q, r, p);
        assert_eq!(res.score, 8);
        assert_eq!(res.query_start, 0);
        assert_eq!(res.query_end, 4);
        assert_eq!(res.ref_start, 0);
        assert_eq!(res.ref_end, 4);
        assert_eq!(res.cigar, "4M");
        assert_eq!(res.nm, 0);
    }

    #[test]
    fn sw_single_mismatch_still_aligns_full() {
        let p = default_params();
        let q = b"AGGT";
        let r = b"ACGT";
        let res = banded_sw(q, r, p);
        assert_eq!(res.cigar, "4M");
        assert_eq!(res.query_start, 0);
        assert_eq!(res.query_end, 4);
        assert_eq!(res.ref_start, 0);
        assert_eq!(res.ref_end, 4);
        assert_eq!(res.score, 3 * 2 - 1);
        assert_eq!(res.nm, 1);
    }

    #[test]
    fn sw_single_insertion() {
        let p = default_params();
        let q = b"ACGGT";
        let r = b"ACGT";
        let res = banded_sw(q, r, p);
        assert_eq!(res.score, 7);
        assert_eq!(res.query_start, 0);
        assert_eq!(res.query_end, 5);
        assert_eq!(res.ref_start, 0);
        assert_eq!(res.ref_end, 4);
        assert_eq!(res.cigar, "2M1I2M");
        assert_eq!(res.nm, 1);
    }

    #[test]
    fn mapq_simple_model() {
        assert_eq!(compute_mapq(50, 0), 60);
        assert_eq!(compute_mapq(50, 25), 30);
        assert_eq!(compute_mapq(10, 10), 0);
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
        let sa = sa::build_sa(&text);
        let bwt = bwt::build_bwt(&text, &sa);
        FMIndex::build(text, bwt, sa, contigs, dna::SIGMA as u8, 4)
    }

    #[test]
    fn mem_seeds_basic() {
        let fm = build_test_fm(b"ACGTACGT");
        let read = b"CGTA";
        let norm = dna::normalize_seq(read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_mem_seeds(&fm, &alpha, 2);
        assert!(
            seeds
                .iter()
                .any(|s| s.contig == 0 && s.qb == 0 && s.qe == 4 && s.rb == 1 && s.re == 5)
        );
    }

    #[test]
    fn mem_seeds_respect_min_len() {
        let fm = build_test_fm(b"ACGTACGT");
        let read = b"CGTA";
        let norm = dna::normalize_seq(read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_mem_seeds(&fm, &alpha, 5);
        assert!(seeds.is_empty());
    }

    #[test]
    fn best_chain_simple_diagonal() {
        let seeds = vec![
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 0,
                re: 4,
            },
            MemSeed {
                contig: 0,
                qb: 4,
                qe: 8,
                rb: 4,
                re: 8,
            },
        ];
        let chain = best_chain(&seeds, 10).expect("chain");
        assert_eq!(chain.contig, 0);
        assert_eq!(chain.seeds.len(), 2);
        assert_eq!(chain.score, 8);
    }

    #[test]
    fn best_chain_avoids_overlapping_and_far_gaps() {
        let seeds = vec![
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 0,
                re: 4,
            },
            // overlap on read / ref，不能接在第一个后面
            MemSeed {
                contig: 0,
                qb: 3,
                qe: 6,
                rb: 3,
                re: 6,
            },
            // 离第一个太远，超过 max_gap
            MemSeed {
                contig: 0,
                qb: 20,
                qe: 24,
                rb: 20,
                re: 24,
            },
            // 合理间距，可与第一个组成链
            MemSeed {
                contig: 0,
                qb: 4,
                qe: 8,
                rb: 4,
                re: 8,
            },
        ];

        let chain = best_chain(&seeds, 10).expect("chain");
        assert_eq!(chain.seeds.len(), 2);
        assert_eq!(chain.seeds[0].qb, 0);
        assert_eq!(chain.seeds[1].qb, 4);
        assert_eq!(chain.score, 8);
    }

    #[test]
    fn sw_deletion() {
        let p = default_params();
        let q = b"ACGT";
        let r = b"ACGGT";
        let res = banded_sw(q, r, p);
        assert!(res.score > 0);
        assert!(res.cigar.contains('D') || res.cigar.contains('M'));
    }

    #[test]
    fn sw_empty_inputs() {
        let p = default_params();
        let res = banded_sw(b"", b"ACGT", p);
        assert_eq!(res.score, 0);
        let res2 = banded_sw(b"ACGT", b"", p);
        assert_eq!(res2.score, 0);
    }

    #[test]
    fn sw_all_mismatch() {
        let p = default_params();
        let q = b"AAAA";
        let r = b"TTTT";
        let res = banded_sw(q, r, p);
        // With match_score=2, mismatch_penalty=1, local SW should still try
        // but score may be low or zero if penalty accumulates
        assert!(res.score >= 0);
    }

    #[test]
    fn align_one_direction_exact_match() {
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
        let res = align_one_direction(&fm, &norm, &alpha, sw);
        assert!(res.is_some());
        let db = res.unwrap();
        assert!(db.best_score > 0);
        assert!(db.best_cigar.contains('M'));
        assert_eq!(db.best_nm, 0);
    }

    #[test]
    fn align_one_direction_with_mismatch() {
        // Use a long unique reference so both sides of a mismatch can form >=20bp seeds
        let reference = b"ACGTACGTAGCTGATCGTAGCTAGCTAGCTGATCGTAGCTAGCTAGCTGAT";
        let fm = build_test_fm(reference);
        // Take first 40bp of reference and introduce a mismatch at position 20
        let mut read = reference[..40].to_vec();
        // Flip one base in the middle
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
        let res = align_one_direction(&fm, &norm, &alpha, sw);
        // Should still align (seeds on flanking exact regions >= 20bp)
        assert!(res.is_some());
        let db = res.unwrap();
        assert!(db.best_score > 0);
    }

    #[test]
    fn chain_to_alignment_single_seed() {
        let p = default_params();
        let query = b"ACGT";
        let reference = b"ACGT";
        let chain = Chain {
            contig: 0,
            seeds: vec![MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 0,
                re: 4,
            }],
            score: 4,
        };
        let res = chain_to_alignment(&chain, query, reference, p);
        assert_eq!(res.score, 8); // 4 bases * match_score(2)
        assert_eq!(res.cigar, "4M");
        assert_eq!(res.nm, 0);
    }

    #[test]
    fn chain_to_alignment_empty_chain() {
        let p = default_params();
        let chain = Chain {
            contig: 0,
            seeds: vec![],
            score: 0,
        };
        let res = chain_to_alignment(&chain, b"ACGT", b"ACGT", p);
        assert_eq!(res.score, 0);
        assert!(res.cigar.is_empty());
    }

    #[test]
    fn mapq_edge_cases() {
        assert_eq!(compute_mapq(0, 0), 0);
        assert_eq!(compute_mapq(-5, 0), 0);
        assert_eq!(compute_mapq(100, 0), 60);
        assert_eq!(compute_mapq(100, 100), 0);
        assert_eq!(compute_mapq(100, 50), 30);
    }
}
