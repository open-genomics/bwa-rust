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

#[derive(Debug, PartialEq, Eq)]
pub struct SwResult {
    pub score: i32,
    pub query_start: usize,
    pub query_end: usize,
    pub ref_start: usize,
    pub ref_end: usize,
    pub cigar: String,
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
    }
}

pub fn align_fastq(index_path: &str, fastq_path: &str, out_path: Option<&str>) -> Result<()> {
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

    // iterate reads
    while let Some(rec) = reader.next_record()? {
        let qname = &rec.id;
        let seq = &rec.seq;
        let qual = &rec.qual;

        // prepare forward
        let fwd_norm = dna::normalize_seq(seq);
        let fwd_alpha: Vec<u8> = fwd_norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        // prepare reverse complement
        let rev = dna::revcomp(seq);
        let rev_alpha: Vec<u8> = rev.iter().map(|&b| dna::to_alphabet(b)).collect();

        let mut write_unmapped = true;

        // try forward
        if let Some((l, r)) = fm.backward_search(&fwd_alpha) {
            if r > l {
                let pos = fm.sa_interval_positions(l, r)[0];
                if let Some((ci, off)) = fm.map_text_pos(pos) {
                    let contig = &fm.contigs[ci];
                    // FLAG 0: forward strand
                    let flag = 0;
                    let rname = &contig.name;
                    let pos1 = off + 1; // 1-based
                    let mapq = 255;
                    let cigar = format!("{}M", seq.len());
                    writeln!(
                        out_box,
                        "{}\t{}\t{}\t{}\t{}\t{}\t*\t0\t0\t{}\t{}",
                        qname,
                        flag,
                        rname,
                        pos1,
                        mapq,
                        cigar,
                        String::from_utf8_lossy(seq),
                        String::from_utf8_lossy(qual),
                    )?;
                    write_unmapped = false;
                }
            }
        }

        // try reverse if forward failed
        if write_unmapped {
            if let Some((l, r)) = fm.backward_search(&rev_alpha) {
                if r > l {
                    let pos = fm.sa_interval_positions(l, r)[0];
                    if let Some((ci, off)) = fm.map_text_pos(pos) {
                        let contig = &fm.contigs[ci];
                        // FLAG 16: reverse complemented
                        let flag = 16;
                        let rname = &contig.name;
                        let pos1 = off + 1; // 1-based
                        let mapq = 255;
                        let cigar = format!("{}M", seq.len());
                        writeln!(
                            out_box,
                            "{}\t{}\t{}\t{}\t{}\t{}\t*\t0\t0\t{}\t{}",
                            qname,
                            flag,
                            rname,
                            pos1,
                            mapq,
                            cigar,
                            String::from_utf8_lossy(seq),
                            String::from_utf8_lossy(qual),
                        )?;
                        write_unmapped = false;
                    }
                }
            }
        }

        if write_unmapped {
            // unmapped: FLAG 4, RNEXT/PNEXT/SEQ/QUAL as per SAM minimal
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
    }
}
