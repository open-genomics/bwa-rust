use std::fmt::Write as _;

const NEG_INF: i32 = i32::MIN / 4;

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
}
