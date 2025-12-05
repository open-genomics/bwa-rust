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
}

impl Default for AlignOpt {
    fn default() -> Self {
        Self {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 16,
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
        let up_idx = (i - 1) * cols + j;
        let left_idx = i * cols + (j - 1);

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
            ops.push('D');
            i -= 1;
        } else if h_here == f_val {
            ops.push('I');
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

pub fn align_fastq(index_path: &str, fastq_path: &str, out_path: Option<&str>) -> Result<()> {
    let opt = AlignOpt::default();
    align_fastq_with_opt(index_path, fastq_path, out_path, opt)
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

        if has_best {
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

const MAX_SEED_HITS: usize = 16;

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

    // 取中间的一段作为 seed
    let seed_len = len.min(20);
    if seed_len == 0 {
        return None;
    }
    let seed_start = (len - seed_len) / 2;
    let seed = &query_alpha[seed_start..seed_start + seed_len];

    let (l, r) = match fm.backward_search(seed) {
        Some(v) => v,
        None => return None,
    };
    if l >= r {
        return None;
    }

    let hits = fm.sa_interval_positions(l, r);
    if hits.is_empty() {
        return None;
    }

    let max_hits = MAX_SEED_HITS.min(hits.len());
    let mut best_score = 0i32;
    let mut best_ci = 0usize;
    let mut best_pos: u32 = 0;
    let mut best_cigar = String::new();
    let mut best_nm: u32 = 0;
    let mut second_best_score = 0i32;
    let mut has_best = false;

    for &pos in &hits[..max_hits] {
        if let Some((ci, off_in_contig)) = fm.map_text_pos(pos) {
            let contig = &fm.contigs[ci];
            let contig_len = contig.len as usize;
            let off = off_in_contig as usize;
            if contig_len == 0 {
                continue;
            }

            // 参考窗口：以 seed 起点为中心，左右各扩展约一个 read 长度
            let flank = query_norm.len().min(contig_len);
            let win_start_in_contig = off.saturating_sub(flank);
            let win_end_in_contig = (off + seed_len + flank).min(contig_len);
            if win_start_in_contig >= win_end_in_contig {
                continue;
            }

            let text_start = contig.offset as usize + win_start_in_contig;
            let text_end = text_start + (win_end_in_contig - win_start_in_contig);

            let mut ref_window: Vec<u8> = Vec::with_capacity(win_end_in_contig - win_start_in_contig);
            for &code in &fm.text[text_start..text_end] {
                if code == 0 {
                    break; // 不跨越 contig 分隔符
                }
                ref_window.push(dna::from_alphabet(code));
            }
            if ref_window.is_empty() {
                continue;
            }

            let sw_res = banded_sw(query_norm, &ref_window, sw_params);
            if sw_res.score <= 0 || sw_res.cigar.is_empty() {
                continue;
            }

            let global_off_in_contig = win_start_in_contig + sw_res.ref_start;
            if global_off_in_contig >= contig_len {
                continue;
            }

            let score = sw_res.score;
            if !has_best || score > best_score {
                if has_best && best_score > second_best_score {
                    second_best_score = best_score;
                }
                best_score = score;
                best_ci = ci;
                best_pos = global_off_in_contig as u32;
                best_cigar = sw_res.cigar;
                best_nm = sw_res.nm;
                has_best = true;
            } else if score > second_best_score {
                second_best_score = score;
            }
        }
    }

    if has_best {
        Some(DirectionBest {
            best_score,
            best_ci,
            best_pos,
            best_cigar,
            best_nm,
            second_best_score,
        })
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
}
