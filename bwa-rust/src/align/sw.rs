use std::fmt::Write as _;

const NEG_INF: i32 = i32::MIN / 4;

/// 扩展对齐结果（用于链端延伸）
#[derive(Debug, Clone)]
pub struct ExtendResult {
    /// 扩展后的对齐得分
    pub score: i32,
    /// query 上实际被覆盖的长度（从起始端延伸了多少）
    pub query_len: usize,
    /// ref 上实际被覆盖的长度
    pub ref_len: usize,
    /// CIGAR ops（顺序与延伸方向一致）
    pub ops: Vec<char>,
}

#[derive(Clone, Copy, Debug)]
pub struct SwParams {
    pub match_score: i32,
    pub mismatch_penalty: i32,
    pub gap_open: i32,
    pub gap_extend: i32,
    pub band_width: usize,
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

/// 带状仿射间隙 Smith-Waterman 局部对齐
/// 使用可复用的缓冲区以减少内存分配
pub fn banded_sw(query: &[u8], reference: &[u8], p: SwParams) -> SwResult {
    banded_sw_with_buf(query, reference, p, &mut SwBuffer::new())
}

/// DP 工作缓冲区，可跨调用复用
pub struct SwBuffer {
    h: Vec<i32>,
    e: Vec<i32>,
    f: Vec<i32>,
}

impl Default for SwBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl SwBuffer {
    pub fn new() -> Self {
        Self {
            h: Vec::new(),
            e: Vec::new(),
            f: Vec::new(),
        }
    }

    fn resize(&mut self, size: usize) {
        self.h.resize(size, 0);
        self.e.resize(size, NEG_INF);
        self.f.resize(size, NEG_INF);
        self.h.iter_mut().for_each(|v| *v = 0);
        self.e.iter_mut().for_each(|v| *v = NEG_INF);
        self.f.iter_mut().for_each(|v| *v = NEG_INF);
    }
}

pub fn banded_sw_with_buf(query: &[u8], reference: &[u8], p: SwParams, buf: &mut SwBuffer) -> SwResult {
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

    buf.resize(size);
    let h = &mut buf.h;
    let e = &mut buf.e;
    let f = &mut buf.f;

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

            let e_open = h[up_idx] - p.gap_open - p.gap_extend;
            let e_ext = e[up_idx] - p.gap_extend;
            e[idx] = e_open.max(e_ext);

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

    let cigar = ops_to_cigar(&ops);

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

pub fn ops_to_cigar(ops: &[char]) -> String {
    let mut cigar = String::new();
    if ops.is_empty() {
        return cigar;
    }
    let mut cur = ops[0];
    let mut len = 1usize;
    for &op in &ops[1..] {
        if op == cur {
            len += 1;
        } else {
            let _ = write!(&mut cigar, "{}{}", len, cur);
            cur = op;
            len = 1;
        }
    }
    let _ = write!(&mut cigar, "{}{}", len, cur);
    cigar
}

/// 从 (0,0) 向右做半全局扩展对齐（类似 BWA ksw_extend）。
/// query/reference 均从左往右，延伸直到序列末尾或得分跌落超过 zdrop。
/// 返回实际延伸到的位置和 CIGAR。
pub fn extend_right(query: &[u8], reference: &[u8], p: SwParams, zdrop: i32) -> ExtendResult {
    let m = query.len();
    let n = reference.len();
    if m == 0 || n == 0 {
        return ExtendResult {
            score: 0,
            query_len: 0,
            ref_len: 0,
            ops: vec![],
        };
    }

    let rows = m + 1;
    let cols = n + 1;
    let mut h = vec![NEG_INF; rows * cols];
    let mut e = vec![NEG_INF; rows * cols];
    let mut f = vec![NEG_INF; rows * cols];

    // 初始化：允许从任何 ref 位置免费开始（半全局：query 从头开始，ref 可从任意位置开始）
    // 这里我们做 query 和 ref 都从 0 开始的全局化延伸（ksw_extend 风格：固定起始端）
    for item in h.iter_mut().take(n + 1) {
        *item = 0;
    }

    let mut best_score = 0i32;
    let mut best_i = 0usize;
    let mut best_j = 0usize;
    let mut max_score = 0i32;

    for i in 1..=m {
        let i_isize = i as isize;
        let band = p.band_width as isize;
        let j_lo = if band >= 0 { (i_isize - band).max(1) as usize } else { 1 };
        let j_hi = if band >= 0 {
            (i_isize + band).min(n as isize) as usize
        } else {
            n
        };

        for j in j_lo..=j_hi {
            let idx = i * cols + j;
            let up_idx = (i - 1) * cols + j;
            let left_idx = i * cols + (j - 1);
            let diag_idx = (i - 1) * cols + (j - 1);

            let e_val = if h[up_idx] != NEG_INF {
                (h[up_idx] - p.gap_open - p.gap_extend).max(if e[up_idx] != NEG_INF {
                    e[up_idx] - p.gap_extend
                } else {
                    NEG_INF
                })
            } else {
                NEG_INF
            };
            e[idx] = e_val;

            let f_val = if h[left_idx] != NEG_INF {
                (h[left_idx] - p.gap_open - p.gap_extend).max(if f[left_idx] != NEG_INF {
                    f[left_idx] - p.gap_extend
                } else {
                    NEG_INF
                })
            } else {
                NEG_INF
            };
            f[idx] = f_val;

            let subst = if query[i - 1] == reference[j - 1] {
                p.match_score
            } else {
                -p.mismatch_penalty
            };
            let diag_val = if h[diag_idx] != NEG_INF {
                h[diag_idx] + subst
            } else {
                NEG_INF
            };

            let mut val = diag_val;
            if e_val > val {
                val = e_val;
            }
            if f_val > val {
                val = f_val;
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
            if val > max_score {
                max_score = val;
            }
        }

        // z-drop: if max score seen in this row is too far below global max, stop
        let row_best = (j_lo..=j_hi).map(|j| h[i * cols + j]).max().unwrap_or(NEG_INF);
        if zdrop > 0 && max_score - row_best >= zdrop {
            break;
        }
    }

    if best_score <= 0 {
        return ExtendResult {
            score: 0,
            query_len: 0,
            ref_len: 0,
            ops: vec![],
        };
    }

    // 回溯
    let mut ops: Vec<char> = Vec::new();
    let mut i = best_i;
    let mut j = best_j;
    while i > 0 && j > 0 {
        let idx = i * cols + j;
        let hv = h[idx];
        if hv == 0 {
            break;
        }
        let diag_idx = (i - 1) * cols + (j - 1);
        let subst = if query[i - 1] == reference[j - 1] {
            p.match_score
        } else {
            -p.mismatch_penalty
        };
        let dv = if h[diag_idx] != NEG_INF {
            h[diag_idx] + subst
        } else {
            NEG_INF
        };
        if hv == dv {
            ops.push('M');
            i -= 1;
            j -= 1;
        } else if hv == e[idx] {
            ops.push('I');
            i -= 1;
        } else if hv == f[idx] {
            ops.push('D');
            j -= 1;
        } else {
            break;
        }
    }
    ops.reverse();

    ExtendResult {
        score: best_score,
        query_len: best_i,
        ref_len: best_j,
        ops,
    }
}

/// 从 query/ref 末尾向左做半全局扩展（将两者翻转后调用 extend_right，再翻转结果）。
pub fn extend_left(query: &[u8], reference: &[u8], p: SwParams, zdrop: i32) -> ExtendResult {
    let rq: Vec<u8> = query.iter().rev().cloned().collect();
    let rr: Vec<u8> = reference.iter().rev().cloned().collect();
    let mut res = extend_right(&rq, &rr, p, zdrop);
    res.ops.reverse();
    res
}

pub fn parse_cigar(cigar: &str) -> Vec<(char, usize)> {
    let mut result = Vec::new();
    let mut num = 0usize;
    for ch in cigar.chars() {
        if ch.is_ascii_digit() {
            num = num * 10 + (ch as usize - '0' as usize);
        } else {
            if num > 0 {
                result.push((ch, num));
            }
            num = 0;
        }
    }
    result
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
    fn sw_single_mismatch() {
        let p = default_params();
        let q = b"AGGT";
        let r = b"ACGT";
        let res = banded_sw(q, r, p);
        assert_eq!(res.cigar, "4M");
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
        assert_eq!(res.cigar, "2M1I2M");
        assert_eq!(res.nm, 1);
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
        assert_eq!(banded_sw(b"", b"ACGT", p).score, 0);
        assert_eq!(banded_sw(b"ACGT", b"", p).score, 0);
    }

    #[test]
    fn sw_buffer_reuse() {
        let p = default_params();
        let mut buf = SwBuffer::new();
        let r1 = banded_sw_with_buf(b"ACGT", b"ACGT", p, &mut buf);
        assert_eq!(r1.score, 8);
        let r2 = banded_sw_with_buf(b"AGGT", b"ACGT", p, &mut buf);
        assert_eq!(r2.nm, 1);
    }

    #[test]
    fn ops_to_cigar_empty() {
        assert_eq!(ops_to_cigar(&[]), "");
    }

    #[test]
    fn ops_to_cigar_single_op() {
        assert_eq!(ops_to_cigar(&['M']), "1M");
    }

    #[test]
    fn ops_to_cigar_run_length() {
        assert_eq!(ops_to_cigar(&['M', 'M', 'M', 'I', 'M', 'M']), "3M1I2M");
    }

    #[test]
    fn ops_to_cigar_all_different() {
        assert_eq!(ops_to_cigar(&['M', 'I', 'D', 'M']), "1M1I1D1M");
    }

    #[test]
    fn parse_cigar_basic() {
        let parsed = parse_cigar("3M1I2M");
        assert_eq!(parsed, vec![('M', 3), ('I', 1), ('M', 2)]);
    }

    #[test]
    fn parse_cigar_empty() {
        let parsed = parse_cigar("");
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_cigar_complex() {
        let parsed = parse_cigar("10M2D5M1I3M");
        assert_eq!(parsed, vec![('M', 10), ('D', 2), ('M', 5), ('I', 1), ('M', 3)]);
    }

    #[test]
    fn parse_cigar_roundtrip() {
        let ops = vec!['M', 'M', 'M', 'I', 'D', 'M', 'M'];
        let cigar = ops_to_cigar(&ops);
        let parsed = parse_cigar(&cigar);
        let mut reconstructed = Vec::new();
        for (op, count) in parsed {
            for _ in 0..count {
                reconstructed.push(op);
            }
        }
        assert_eq!(reconstructed, ops);
    }

    #[test]
    fn extend_right_perfect_match() {
        let p = default_params();
        let res = extend_right(b"ACGT", b"ACGT", p, 100);
        assert!(res.score > 0);
        assert_eq!(res.query_len, 4);
        assert_eq!(res.ref_len, 4);
        assert_eq!(res.ops.len(), 4);
        assert!(res.ops.iter().all(|&op| op == 'M'));
    }

    #[test]
    fn extend_right_empty_input() {
        let p = default_params();
        let res = extend_right(b"", b"ACGT", p, 100);
        assert_eq!(res.score, 0);
        assert_eq!(res.query_len, 0);
        let res2 = extend_right(b"ACGT", b"", p, 100);
        assert_eq!(res2.score, 0);
    }

    #[test]
    fn extend_left_perfect_match() {
        let p = default_params();
        let res = extend_left(b"ACGT", b"ACGT", p, 100);
        assert!(res.score > 0);
        assert_eq!(res.query_len, 4);
        assert_eq!(res.ref_len, 4);
    }

    #[test]
    fn extend_left_empty_input() {
        let p = default_params();
        let res = extend_left(b"", b"ACGT", p, 100);
        assert_eq!(res.score, 0);
    }

    #[test]
    fn sw_all_mismatches() {
        let p = default_params();
        let q = b"AAAA";
        let r = b"TTTT";
        let res = banded_sw(q, r, p);
        // With match_score=2, mismatch=-1, score per position = -1
        // Smith-Waterman local: score should be 0 (all mismatches, local resets to 0)
        assert_eq!(res.score, 0);
    }

    #[test]
    fn sw_longer_sequences() {
        let p = SwParams {
            match_score: 1,
            mismatch_penalty: 4,
            gap_open: 6,
            gap_extend: 1,
            band_width: 100,
        };
        let q = b"ACGTACGTACGTACGT";
        let r = b"ACGTACGTACGTACGT";
        let res = banded_sw(q, r, p);
        assert_eq!(res.score, 16);
        assert_eq!(res.cigar, "16M");
        assert_eq!(res.nm, 0);
    }

    #[test]
    fn sw_partial_match_in_longer_ref() {
        let p = default_params();
        let q = b"ACGT";
        let r = b"TTTTACGTTTTT";
        let res = banded_sw(q, r, p);
        assert_eq!(res.score, 8);
        assert_eq!(res.cigar, "4M");
        assert_eq!(res.ref_start, 4);
        assert_eq!(res.ref_end, 8);
    }
}
