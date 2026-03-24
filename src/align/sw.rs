use std::fmt::Write as _;

const NEG_INF: i32 = i32::MIN / 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TraceState {
    Start,
    Match,
    Ins,
    Del,
}

#[inline]
fn trace_to_u8(state: TraceState) -> u8 {
    match state {
        TraceState::Start => 0,
        TraceState::Match => 1,
        TraceState::Ins => 2,
        TraceState::Del => 3,
    }
}

#[inline]
fn u8_to_trace(code: u8) -> TraceState {
    match code {
        1 => TraceState::Match,
        2 => TraceState::Ins,
        3 => TraceState::Del,
        _ => TraceState::Start,
    }
}

#[inline]
fn penalize(score: i32, penalty: i32) -> i32 {
    if score <= NEG_INF / 2 {
        NEG_INF
    } else {
        score - penalty
    }
}

#[inline]
fn nm_from_ops(ops: &[char], query: &[u8], reference: &[u8]) -> u32 {
    let mut qi = 0usize;
    let mut rj = 0usize;
    let mut nm = 0u32;

    for &op in ops {
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

    nm
}

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

/// 端到端全覆盖比对。
/// 用于链内两个锚点之间的 gap 补齐，必须同时覆盖完整 query/reference 片段。
pub fn global_align(query: &[u8], reference: &[u8], p: SwParams) -> SwResult {
    global_align_with_buf(query, reference, p, &mut SwBuffer::new())
}

pub fn global_align_with_buf(query: &[u8], reference: &[u8], p: SwParams, buf: &mut SwBuffer) -> SwResult {
    let m = query.len();
    let n = reference.len();

    if m == 0 && n == 0 {
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

    let cols = n + 1;
    let size = (m + 1) * cols;
    buf.resize_affine(size);
    let match_mat = &mut buf.h;
    let ins_mat = &mut buf.e;
    let del_mat = &mut buf.f;
    let match_trace = &mut buf.match_trace;
    let ins_trace = &mut buf.ins_trace;
    let del_trace = &mut buf.del_trace;

    let idx = |i: usize, j: usize| i * cols + j;

    match_mat[idx(0, 0)] = 0;

    for i in 1..=m {
        let cur = idx(i, 0);
        let prev = idx(i - 1, 0);
        let open = penalize(match_mat[prev], p.gap_open + p.gap_extend);
        let extend = penalize(ins_mat[prev], p.gap_extend);
        if open >= extend {
            ins_mat[cur] = open;
            ins_trace[cur] = trace_to_u8(TraceState::Match);
        } else {
            ins_mat[cur] = extend;
            ins_trace[cur] = trace_to_u8(TraceState::Ins);
        }
    }

    for j in 1..=n {
        let cur = idx(0, j);
        let prev = idx(0, j - 1);
        let open = penalize(match_mat[prev], p.gap_open + p.gap_extend);
        let extend = penalize(del_mat[prev], p.gap_extend);
        if open >= extend {
            del_mat[cur] = open;
            del_trace[cur] = trace_to_u8(TraceState::Match);
        } else {
            del_mat[cur] = extend;
            del_trace[cur] = trace_to_u8(TraceState::Del);
        }
    }

    for i in 1..=m {
        for j in 1..=n {
            let cur = idx(i, j);
            let diag = idx(i - 1, j - 1);
            let up = idx(i - 1, j);
            let left = idx(i, j - 1);

            let subst = if query[i - 1] == reference[j - 1] {
                p.match_score
            } else {
                -p.mismatch_penalty
            };

            let mut best_prev = match_mat[diag];
            let mut best_state = TraceState::Match;
            if ins_mat[diag] > best_prev {
                best_prev = ins_mat[diag];
                best_state = TraceState::Ins;
            }
            if del_mat[diag] > best_prev {
                best_prev = del_mat[diag];
                best_state = TraceState::Del;
            }
            if best_prev > NEG_INF / 2 {
                match_mat[cur] = best_prev + subst;
                match_trace[cur] = trace_to_u8(best_state);
            }

            let open_ins = penalize(match_mat[up], p.gap_open + p.gap_extend);
            let extend_ins = penalize(ins_mat[up], p.gap_extend);
            if open_ins >= extend_ins {
                ins_mat[cur] = open_ins;
                ins_trace[cur] = trace_to_u8(TraceState::Match);
            } else {
                ins_mat[cur] = extend_ins;
                ins_trace[cur] = trace_to_u8(TraceState::Ins);
            }

            let open_del = penalize(match_mat[left], p.gap_open + p.gap_extend);
            let extend_del = penalize(del_mat[left], p.gap_extend);
            if open_del >= extend_del {
                del_mat[cur] = open_del;
                del_trace[cur] = trace_to_u8(TraceState::Match);
            } else {
                del_mat[cur] = extend_del;
                del_trace[cur] = trace_to_u8(TraceState::Del);
            }
        }
    }

    let end = idx(m, n);
    let mut score = match_mat[end];
    let mut state = TraceState::Match;
    if ins_mat[end] > score {
        score = ins_mat[end];
        state = TraceState::Ins;
    }
    if del_mat[end] > score {
        score = del_mat[end];
        state = TraceState::Del;
    }

    let mut ops: Vec<char> = Vec::with_capacity(m.max(n));
    let mut i = m;
    let mut j = n;
    while i > 0 || j > 0 {
        let cur = idx(i, j);
        match state {
            TraceState::Match => {
                ops.push('M');
                state = u8_to_trace(match_trace[cur]);
                i -= 1;
                j -= 1;
            }
            TraceState::Ins => {
                ops.push('I');
                state = u8_to_trace(ins_trace[cur]);
                i -= 1;
            }
            TraceState::Del => {
                ops.push('D');
                state = u8_to_trace(del_trace[cur]);
                j -= 1;
            }
            TraceState::Start => break,
        }
    }
    ops.reverse();

    SwResult {
        score,
        query_start: 0,
        query_end: m,
        ref_start: 0,
        ref_end: n,
        cigar: ops_to_cigar(&ops),
        nm: nm_from_ops(&ops, query, reference),
    }
}

/// Query 全长对齐到 reference 的某个局部窗口。
/// query 必须完整对齐；reference 两端允许免费裁剪，用于候选的局部重打分。
pub fn semiglobal_align(query: &[u8], reference: &[u8], p: SwParams) -> SwResult {
    semiglobal_align_with_buf(query, reference, p, &mut SwBuffer::new())
}

pub fn semiglobal_align_with_buf(query: &[u8], reference: &[u8], p: SwParams, buf: &mut SwBuffer) -> SwResult {
    let m = query.len();
    let n = reference.len();

    if m == 0 {
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
    if n == 0 {
        let cigar = ops_to_cigar(&vec!['I'; m]);
        return SwResult {
            score: -(p.gap_open + p.gap_extend * m as i32),
            query_start: 0,
            query_end: m,
            ref_start: 0,
            ref_end: 0,
            cigar,
            nm: m as u32,
        };
    }

    let cols = n + 1;
    let size = (m + 1) * cols;
    buf.resize_affine(size);
    let match_mat = &mut buf.h;
    let ins_mat = &mut buf.e;
    let del_mat = &mut buf.f;
    let match_trace = &mut buf.match_trace;
    let ins_trace = &mut buf.ins_trace;
    let del_trace = &mut buf.del_trace;

    let idx = |i: usize, j: usize| i * cols + j;

    for j in 0..=n {
        match_mat[idx(0, j)] = 0;
    }

    for i in 1..=m {
        let cur = idx(i, 0);
        let prev = idx(i - 1, 0);
        let open = penalize(match_mat[prev], p.gap_open + p.gap_extend);
        let extend = penalize(ins_mat[prev], p.gap_extend);
        if open >= extend {
            ins_mat[cur] = open;
            ins_trace[cur] = trace_to_u8(TraceState::Match);
        } else {
            ins_mat[cur] = extend;
            ins_trace[cur] = trace_to_u8(TraceState::Ins);
        }
    }

    for i in 1..=m {
        for j in 1..=n {
            let cur = idx(i, j);
            let diag = idx(i - 1, j - 1);
            let up = idx(i - 1, j);
            let left = idx(i, j - 1);

            let subst = if query[i - 1] == reference[j - 1] {
                p.match_score
            } else {
                -p.mismatch_penalty
            };

            let mut best_prev = match_mat[diag];
            let mut best_state = TraceState::Match;
            if ins_mat[diag] > best_prev {
                best_prev = ins_mat[diag];
                best_state = TraceState::Ins;
            }
            if del_mat[diag] > best_prev {
                best_prev = del_mat[diag];
                best_state = TraceState::Del;
            }
            if best_prev > NEG_INF / 2 {
                match_mat[cur] = best_prev + subst;
                match_trace[cur] = trace_to_u8(best_state);
            }

            let open_ins = penalize(match_mat[up], p.gap_open + p.gap_extend);
            let extend_ins = penalize(ins_mat[up], p.gap_extend);
            if open_ins >= extend_ins {
                ins_mat[cur] = open_ins;
                ins_trace[cur] = trace_to_u8(TraceState::Match);
            } else {
                ins_mat[cur] = extend_ins;
                ins_trace[cur] = trace_to_u8(TraceState::Ins);
            }

            let open_del = penalize(match_mat[left], p.gap_open + p.gap_extend);
            let extend_del = penalize(del_mat[left], p.gap_extend);
            if open_del >= extend_del {
                del_mat[cur] = open_del;
                del_trace[cur] = trace_to_u8(TraceState::Match);
            } else {
                del_mat[cur] = extend_del;
                del_trace[cur] = trace_to_u8(TraceState::Del);
            }
        }
    }

    let mut best_j = 0usize;
    let mut score = NEG_INF;
    let mut state = TraceState::Start;
    for j in 0..=n {
        let cur = idx(m, j);
        if match_mat[cur] > score {
            score = match_mat[cur];
            state = TraceState::Match;
            best_j = j;
        }
        if ins_mat[cur] > score {
            score = ins_mat[cur];
            state = TraceState::Ins;
            best_j = j;
        }
        if del_mat[cur] > score {
            score = del_mat[cur];
            state = TraceState::Del;
            best_j = j;
        }
    }

    let mut ops: Vec<char> = Vec::with_capacity(m.max(best_j));
    let mut i = m;
    let mut j = best_j;
    while i > 0 {
        let cur = idx(i, j);
        match state {
            TraceState::Match => {
                ops.push('M');
                state = u8_to_trace(match_trace[cur]);
                i -= 1;
                j -= 1;
            }
            TraceState::Ins => {
                ops.push('I');
                state = u8_to_trace(ins_trace[cur]);
                i -= 1;
            }
            TraceState::Del => {
                ops.push('D');
                state = u8_to_trace(del_trace[cur]);
                j -= 1;
            }
            TraceState::Start => break,
        }
    }
    ops.reverse();

    SwResult {
        score,
        query_start: 0,
        query_end: m,
        ref_start: j,
        ref_end: best_j,
        cigar: ops_to_cigar(&ops),
        nm: nm_from_ops(&ops, query, &reference[j..best_j]),
    }
}

/// DP 工作缓冲区，可跨调用复用
pub struct SwBuffer {
    h: Vec<i32>,
    e: Vec<i32>,
    f: Vec<i32>,
    match_trace: Vec<u8>,
    ins_trace: Vec<u8>,
    del_trace: Vec<u8>,
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
            match_trace: Vec::new(),
            ins_trace: Vec::new(),
            del_trace: Vec::new(),
        }
    }

    fn resize(&mut self, size: usize) {
        self.h.clear();
        self.h.resize(size, 0);
        self.e.clear();
        self.e.resize(size, NEG_INF);
        self.f.clear();
        self.f.resize(size, NEG_INF);
    }

    fn resize_affine(&mut self, size: usize) {
        self.h.clear();
        self.h.resize(size, NEG_INF);
        self.e.clear();
        self.e.resize(size, NEG_INF);
        self.f.clear();
        self.f.resize(size, NEG_INF);
        self.match_trace.clear();
        self.match_trace.resize(size, 0);
        self.ins_trace.clear();
        self.ins_trace.resize(size, 0);
        self.del_trace.clear();
        self.del_trace.resize(size, 0);
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

    let cols = n + 1;
    let size = (m + 1) * cols;
    let mut match_mat = vec![NEG_INF; size];
    let mut ins_mat = vec![NEG_INF; size];
    let mut del_mat = vec![NEG_INF; size];
    let mut match_trace = vec![0u8; size];
    let mut ins_trace = vec![0u8; size];
    let mut del_trace = vec![0u8; size];

    let idx = |i: usize, j: usize| i * cols + j;
    match_mat[idx(0, 0)] = 0;

    for i in 1..=m {
        let cur = idx(i, 0);
        let prev = idx(i - 1, 0);
        let open = penalize(match_mat[prev], p.gap_open + p.gap_extend);
        let extend = penalize(ins_mat[prev], p.gap_extend);
        if open >= extend {
            ins_mat[cur] = open;
            ins_trace[cur] = trace_to_u8(TraceState::Match);
        } else {
            ins_mat[cur] = extend;
            ins_trace[cur] = trace_to_u8(TraceState::Ins);
        }
    }

    for j in 1..=n {
        let cur = idx(0, j);
        let prev = idx(0, j - 1);
        let open = penalize(match_mat[prev], p.gap_open + p.gap_extend);
        let extend = penalize(del_mat[prev], p.gap_extend);
        if open >= extend {
            del_mat[cur] = open;
            del_trace[cur] = trace_to_u8(TraceState::Match);
        } else {
            del_mat[cur] = extend;
            del_trace[cur] = trace_to_u8(TraceState::Del);
        }
    }

    let mut best_score = 0i32;
    let mut best_i = 0usize;
    let mut best_j = 0usize;
    let mut max_score = 0i32;
    let mut best_state = TraceState::Start;

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
            let cur = idx(i, j);
            let up = idx(i - 1, j);
            let left = idx(i, j - 1);
            let diag = idx(i - 1, j - 1);

            let subst = if query[i - 1] == reference[j - 1] {
                p.match_score
            } else {
                -p.mismatch_penalty
            };

            let mut best_prev = match_mat[diag];
            let mut prev_state = TraceState::Match;
            if ins_mat[diag] > best_prev {
                best_prev = ins_mat[diag];
                prev_state = TraceState::Ins;
            }
            if del_mat[diag] > best_prev {
                best_prev = del_mat[diag];
                prev_state = TraceState::Del;
            }
            if best_prev > NEG_INF / 2 {
                match_mat[cur] = best_prev + subst;
                match_trace[cur] = trace_to_u8(prev_state);
            }

            let open_ins = penalize(match_mat[up], p.gap_open + p.gap_extend);
            let extend_ins = penalize(ins_mat[up], p.gap_extend);
            if open_ins >= extend_ins {
                ins_mat[cur] = open_ins;
                ins_trace[cur] = trace_to_u8(TraceState::Match);
            } else {
                ins_mat[cur] = extend_ins;
                ins_trace[cur] = trace_to_u8(TraceState::Ins);
            }

            let open_del = penalize(match_mat[left], p.gap_open + p.gap_extend);
            let extend_del = penalize(del_mat[left], p.gap_extend);
            if open_del >= extend_del {
                del_mat[cur] = open_del;
                del_trace[cur] = trace_to_u8(TraceState::Match);
            } else {
                del_mat[cur] = extend_del;
                del_trace[cur] = trace_to_u8(TraceState::Del);
            }

            let mut cell_best = match_mat[cur];
            let mut cell_state = TraceState::Match;
            if ins_mat[cur] > cell_best {
                cell_best = ins_mat[cur];
                cell_state = TraceState::Ins;
            }
            if del_mat[cur] > cell_best {
                cell_best = del_mat[cur];
                cell_state = TraceState::Del;
            }

            if cell_best > best_score {
                best_score = cell_best;
                best_i = i;
                best_j = j;
                best_state = cell_state;
            }
            if cell_best > max_score {
                max_score = cell_best;
            }
        }

        // z-drop: if max score seen in this row is too far below global max, stop
        let row_best = (j_lo..=j_hi)
            .map(|j| {
                let cur = idx(i, j);
                match_mat[cur].max(ins_mat[cur]).max(del_mat[cur])
            })
            .max()
            .unwrap_or(NEG_INF);
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
    let mut state = best_state;
    while i > 0 || j > 0 {
        let cur = idx(i, j);
        match state {
            TraceState::Match => {
                ops.push('M');
                state = u8_to_trace(match_trace[cur]);
                i -= 1;
                j -= 1;
            }
            TraceState::Ins => {
                ops.push('I');
                state = u8_to_trace(ins_trace[cur]);
                i -= 1;
            }
            TraceState::Del => {
                ops.push('D');
                state = u8_to_trace(del_trace[cur]);
                j -= 1;
            }
            TraceState::Start => break,
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

    #[test]
    fn global_align_keeps_full_gap() {
        let p = SwParams {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 8,
        };
        let res = global_align(b"CCCC", b"TTTTCCCC", p);
        assert_eq!(res.cigar, "4D4M");
        assert_eq!(res.nm, 4);
        assert_eq!(res.score, 2);
    }

    #[test]
    fn semiglobal_align_finds_single_insertion() {
        let p = SwParams {
            match_score: 1,
            mismatch_penalty: 4,
            gap_open: 6,
            gap_extend: 1,
            band_width: 32,
        };
        let res = semiglobal_align(b"GGCCAAATTGGCCAATTGGCC", b"TTTGGCCAATTGGCCAATTGGCCTTT", p);
        assert_eq!(res.ref_start, 3);
        assert_eq!(res.ref_end, 23);
        assert!(res.cigar.contains('I'));
        assert!(!res.cigar.contains('S'));
        assert_eq!(res.nm, 1);
    }
}
