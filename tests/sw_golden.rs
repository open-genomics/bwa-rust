//! sw.rs 各 DP 变体的 golden 回归测试。
//!
//! 这些断言固化重构前的真实输出（经 `examples/golden_drain.rs` 采集），
//! 用于在消除四份 DP 复制的重构过程中检测任何输出漂移。
//! 若算法有意的行为变更需要更新此处，必须同时说明变更理由。

use bwa_rust::align::sw::{banded_sw, extend_right, global_align, semiglobal_align, SwParams};

fn p(match_score: i32, mismatch_penalty: i32, gap_open: i32, gap_extend: i32, band_width: usize) -> SwParams {
    SwParams {
        match_score,
        mismatch_penalty,
        gap_open,
        gap_extend,
        band_width,
    }
}

fn sw_str(q: &[u8], r: &[u8], prm: SwParams) -> String {
    let x = banded_sw(q, r, prm);
    format!(
        "banded|score={}|qs={}|qe={}|rs={}|re={}|cigar={}|nm={}",
        x.score, x.query_start, x.query_end, x.ref_start, x.ref_end, x.cigar, x.nm
    )
}

fn global_str(q: &[u8], r: &[u8], prm: SwParams) -> String {
    let x = global_align(q, r, prm);
    format!(
        "global|score={}|qs={}|qe={}|rs={}|re={}|cigar={}|nm={}",
        x.score, x.query_start, x.query_end, x.ref_start, x.ref_end, x.cigar, x.nm
    )
}

fn semi_str(q: &[u8], r: &[u8], prm: SwParams) -> String {
    let x = semiglobal_align(q, r, prm);
    format!(
        "semiglobal|score={}|qs={}|qe={}|rs={}|re={}|cigar={}|nm={}",
        x.score, x.query_start, x.query_end, x.ref_start, x.ref_end, x.cigar, x.nm
    )
}

fn extend_str(q: &[u8], r: &[u8], prm: SwParams, zdrop: i32) -> String {
    let x = extend_right(q, r, prm, zdrop);
    let ops: String = x.ops.iter().collect();
    format!(
        "extend|score={}|ql={}|rl={}|ops={}",
        x.score, x.query_len, x.ref_len, ops
    )
}

#[test]
fn golden_perfect_match() {
    let prm = p(2, 1, 1, 0, 8);
    let q = b"ACGT";
    let r = b"ACGT";
    assert_eq!(sw_str(q, r, prm), "banded|score=8|qs=0|qe=4|rs=0|re=4|cigar=4M|nm=0");
    assert_eq!(
        global_str(q, r, prm),
        "global|score=8|qs=0|qe=4|rs=0|re=4|cigar=4M|nm=0"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=8|qs=0|qe=4|rs=0|re=4|cigar=4M|nm=0"
    );
    assert_eq!(extend_str(q, r, prm, 100), "extend|score=8|ql=4|rl=4|ops=MMMM");
}

#[test]
fn golden_single_mismatch() {
    let prm = p(2, 1, 1, 0, 8);
    let q = b"AGGT";
    let r = b"ACGT";
    assert_eq!(sw_str(q, r, prm), "banded|score=5|qs=0|qe=4|rs=0|re=4|cigar=4M|nm=1");
    assert_eq!(
        global_str(q, r, prm),
        "global|score=5|qs=0|qe=4|rs=0|re=4|cigar=4M|nm=1"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=5|qs=0|qe=4|rs=0|re=4|cigar=4M|nm=1"
    );
    assert_eq!(extend_str(q, r, prm, 100), "extend|score=5|ql=4|rl=4|ops=MMMM");
}

#[test]
fn golden_insertion() {
    let prm = p(2, 1, 1, 0, 8);
    let q = b"ACGGT";
    let r = b"ACGT";
    assert_eq!(
        sw_str(q, r, prm),
        "banded|score=7|qs=0|qe=5|rs=0|re=4|cigar=2M1I2M|nm=1"
    );
    assert_eq!(
        global_str(q, r, prm),
        "global|score=7|qs=0|qe=5|rs=0|re=4|cigar=2M1I2M|nm=1"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=7|qs=0|qe=5|rs=0|re=4|cigar=2M1I2M|nm=1"
    );
    assert_eq!(extend_str(q, r, prm, 100), "extend|score=7|ql=5|rl=4|ops=MMIMM");
}

#[test]
fn golden_deletion() {
    let prm = p(2, 1, 1, 0, 8);
    let q = b"ACGT";
    let r = b"ACGGT";
    assert_eq!(
        sw_str(q, r, prm),
        "banded|score=7|qs=0|qe=4|rs=0|re=5|cigar=2M1D2M|nm=1"
    );
    assert_eq!(
        global_str(q, r, prm),
        "global|score=7|qs=0|qe=4|rs=0|re=5|cigar=2M1D2M|nm=1"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=7|qs=0|qe=4|rs=0|re=5|cigar=2M1D2M|nm=1"
    );
    assert_eq!(extend_str(q, r, prm, 100), "extend|score=7|ql=4|rl=5|ops=MMDMM");
}

#[test]
fn golden_partial_match_in_longer_ref() {
    let prm = p(2, 1, 1, 0, 8);
    let q = b"ACGT";
    let r = b"TTTTACGTTTTT";
    // 局部 SW 只取最佳局部段
    assert_eq!(sw_str(q, r, prm), "banded|score=8|qs=0|qe=4|rs=4|re=8|cigar=4M|nm=0");
    // global 强制整段覆盖，含大量 gap
    assert_eq!(
        global_str(q, r, prm),
        "global|score=6|qs=0|qe=4|rs=0|re=12|cigar=4D3M4D1M|nm=8"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=8|qs=0|qe=4|rs=4|re=8|cigar=4M|nm=0"
    );
    assert_eq!(extend_str(q, r, prm, 100), "extend|score=7|ql=4|rl=8|ops=DDDDMMMM");
}

#[test]
fn golden_global_keeps_full_gap() {
    let prm = p(2, 1, 2, 1, 8);
    let q = b"CCCC";
    let r = b"TTTTCCCC";
    assert_eq!(sw_str(q, r, prm), "banded|score=8|qs=0|qe=4|rs=4|re=8|cigar=4M|nm=0");
    assert_eq!(
        global_str(q, r, prm),
        "global|score=2|qs=0|qe=4|rs=0|re=8|cigar=4D4M|nm=4"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=8|qs=0|qe=4|rs=4|re=8|cigar=4M|nm=0"
    );
    assert_eq!(extend_str(q, r, prm, 100), "extend|score=2|ql=4|rl=8|ops=DDDDMMMM");
}

#[test]
fn golden_semiglobal_insertion() {
    let prm = p(1, 4, 6, 1, 32);
    let q = b"GGCCAAATTGGCCAATTGGCC";
    let r = b"TTTGGCCAATTGGCCAATTGGCCTTT";
    assert_eq!(
        sw_str(q, r, prm),
        "banded|score=16|qs=5|qe=21|rs=7|re=23|cigar=16M|nm=0"
    );
    assert_eq!(
        global_str(q, r, prm),
        "global|score=-5|qs=0|qe=21|rs=0|re=26|cigar=3D4M1I16M3D|nm=7"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=13|qs=0|qe=21|rs=3|re=23|cigar=4M1I16M|nm=1"
    );
    assert_eq!(
        extend_str(q, r, prm, 100),
        "extend|score=4|ql=21|rl=23|ops=DDDMMMMIMMMMMMMMMMMMMMMM"
    );
}

#[test]
fn golden_long_match() {
    let prm = p(1, 4, 6, 1, 100);
    let q = b"ACGTACGTACGTACGT";
    let r = b"ACGTACGTACGTACGT";
    assert_eq!(
        sw_str(q, r, prm),
        "banded|score=16|qs=0|qe=16|rs=0|re=16|cigar=16M|nm=0"
    );
    assert_eq!(
        global_str(q, r, prm),
        "global|score=16|qs=0|qe=16|rs=0|re=16|cigar=16M|nm=0"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=16|qs=0|qe=16|rs=0|re=16|cigar=16M|nm=0"
    );
    assert_eq!(
        extend_str(q, r, prm, 100),
        "extend|score=16|ql=16|rl=16|ops=MMMMMMMMMMMMMMMM"
    );
}

#[test]
fn golden_all_mismatches() {
    let prm = p(2, 1, 1, 0, 8);
    let q = b"AAAA";
    let r = b"TTTT";
    assert_eq!(sw_str(q, r, prm), "banded|score=0|qs=0|qe=0|rs=0|re=0|cigar=|nm=0");
    assert_eq!(
        global_str(q, r, prm),
        "global|score=-3|qs=0|qe=4|rs=0|re=4|cigar=3D1M3I|nm=7"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=-1|qs=0|qe=4|rs=0|re=0|cigar=4I|nm=4"
    );
    assert_eq!(extend_str(q, r, prm, 100), "extend|score=0|ql=0|rl=0|ops=");
}

#[test]
fn golden_left_gap() {
    let prm = p(2, 1, 1, 0, 8);
    let q = b"CGT";
    let r = b"AACGT";
    assert_eq!(sw_str(q, r, prm), "banded|score=6|qs=0|qe=3|rs=2|re=5|cigar=3M|nm=0");
    assert_eq!(
        global_str(q, r, prm),
        "global|score=5|qs=0|qe=3|rs=0|re=5|cigar=2D3M|nm=2"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=6|qs=0|qe=3|rs=2|re=5|cigar=3M|nm=0"
    );
    assert_eq!(extend_str(q, r, prm, 100), "extend|score=5|ql=3|rl=5|ops=DDMMM");
}

#[test]
fn golden_zdrop_limits_extension() {
    // 前半匹配，后半大段错配：zdrop=3 应截断延伸，zdrop=100 应延伸到底
    let prm = p(2, 1, 2, 1, 8);
    let q = b"AAAACCCCCCCC";
    let r = b"AAAAGGGGGGGG";

    // 严格 zdrop
    assert_eq!(sw_str(q, r, prm), "banded|score=8|qs=0|qe=4|rs=0|re=4|cigar=4M|nm=0");
    assert_eq!(
        global_str(q, r, prm),
        "global|score=0|qs=0|qe=12|rs=0|re=12|cigar=12M|nm=8"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=0|qs=0|qe=12|rs=0|re=12|cigar=12M|nm=8"
    );
    assert_eq!(extend_str(q, r, prm, 3), "extend|score=8|ql=4|rl=4|ops=MMMM");

    // 宽松 zdrop
    assert_eq!(sw_str(q, r, prm), "banded|score=8|qs=0|qe=4|rs=0|re=4|cigar=4M|nm=0");
    assert_eq!(
        global_str(q, r, prm),
        "global|score=0|qs=0|qe=12|rs=0|re=12|cigar=12M|nm=8"
    );
    assert_eq!(
        semi_str(q, r, prm),
        "semiglobal|score=0|qs=0|qe=12|rs=0|re=12|cigar=12M|nm=8"
    );
    assert_eq!(extend_str(q, r, prm, 100), "extend|score=8|ql=4|rl=4|ops=MMMM");
}

#[test]
fn golden_empty_inputs() {
    let prm = p(2, 1, 1, 0, 8);
    // 空 query
    assert_eq!(
        sw_str(b"", b"ACGT", prm),
        "banded|score=0|qs=0|qe=0|rs=0|re=0|cigar=|nm=0"
    );
    assert_eq!(
        global_str(b"", b"ACGT", prm),
        "global|score=-1|qs=0|qe=0|rs=0|re=4|cigar=4D|nm=4"
    );
    assert_eq!(
        semi_str(b"", b"ACGT", prm),
        "semiglobal|score=0|qs=0|qe=0|rs=0|re=0|cigar=|nm=0"
    );
    assert_eq!(extend_str(b"", b"ACGT", prm, 100), "extend|score=0|ql=0|rl=0|ops=");
    // 空 ref
    assert_eq!(
        sw_str(b"ACGT", b"", prm),
        "banded|score=0|qs=0|qe=0|rs=0|re=0|cigar=|nm=0"
    );
    assert_eq!(
        global_str(b"ACGT", b"", prm),
        "global|score=-1|qs=0|qe=4|rs=0|re=0|cigar=4I|nm=4"
    );
    assert_eq!(
        semi_str(b"ACGT", b"", prm),
        "semiglobal|score=-1|qs=0|qe=4|rs=0|re=0|cigar=4I|nm=4"
    );
    assert_eq!(extend_str(b"ACGT", b"", prm, 100), "extend|score=0|ql=0|rl=0|ops=");
}
