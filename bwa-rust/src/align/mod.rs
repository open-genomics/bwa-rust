pub mod seed;
pub mod chain;
pub mod sw;

use anyhow::Result;
use std::io::Write;
use std::sync::Arc;

use rayon::prelude::*;

use crate::index::fm::FMIndex;
use crate::io::fastq::{FastqReader, FastqRecord};
use crate::util::dna;

pub use sw::{SwParams, SwResult, banded_sw};
pub use seed::{MemSeed, AlnReg, find_smem_seeds, find_mem_seeds};
pub use chain::{Chain, best_chain, build_chains, filter_chains};

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

pub fn align_fastq_with_opt(
    index_path: &str,
    fastq_path: &str,
    out_path: Option<&str>,
    opt: AlignOpt,
) -> Result<()> {
    let fm = Arc::new(FMIndex::load_from_file(index_path)?);
    align_fastq_with_fm_opt(fm, fastq_path, out_path, opt)
}

pub fn align_fastq_with_fm_opt(
    fm: Arc<FMIndex>,
    fastq_path: &str,
    out_path: Option<&str>,
    opt: AlignOpt,
) -> Result<()> {

    let fq = std::fs::File::open(fastq_path)?;
    let mut reader = FastqReader::new(std::io::BufReader::new(fq));

    let mut out_box: Box<dyn Write> = if let Some(p) = out_path {
        Box::new(std::io::BufWriter::new(std::fs::File::create(p)?))
    } else {
        Box::new(std::io::BufWriter::new(std::io::stdout()))
    };

    // SAM header
    writeln!(out_box, "@HD\tVN:1.6\tSO:unsorted")?;
    for c in &fm.contigs {
        writeln!(out_box, "@SQ\tSN:{}\tLN:{}", c.name, c.len)?;
    }
    writeln!(out_box, "@PG\tID:bwa-rust\tPN:bwa-rust\tVN:0.1.0")?;

    let sw_params = SwParams {
        match_score: opt.match_score,
        mismatch_penalty: opt.mismatch_penalty,
        gap_open: opt.gap_open,
        gap_extend: opt.gap_extend,
        band_width: opt.band_width,
    };

    // 设置 rayon 线程池
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(opt.threads)
        .build()
        .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap());

    // 批量读取 reads 并行处理
    let batch_size = 1000;
    loop {
        let mut batch: Vec<FastqRecord> = Vec::with_capacity(batch_size);
        for _ in 0..batch_size {
            match reader.next_record()? {
                Some(rec) => batch.push(rec),
                None => break,
            }
        }
        if batch.is_empty() {
            break;
        }

        let fm_ref = Arc::clone(&fm);
        let results: Vec<Vec<String>> = pool.install(|| {
            batch
                .par_iter()
                .map(|rec| {
                    align_single_read(&fm_ref, rec, sw_params, &opt)
                })
                .collect()
        });

        for lines in results {
            for line in lines {
                writeln!(out_box, "{}", line)?;
            }
        }
    }

    Ok(())
}

/// 对单条 read 进行比对，返回一个或多个 SAM 行
fn align_single_read(
    fm: &FMIndex,
    rec: &FastqRecord,
    sw_params: SwParams,
    opt: &AlignOpt,
) -> Vec<String> {
    let qname = &rec.id;
    let seq = &rec.seq;
    let qual = &rec.qual;

    if seq.is_empty() {
        return vec![format!(
            "{}\t4\t*\t0\t0\t*\t*\t0\t0\t{}\t{}",
            qname,
            String::from_utf8_lossy(seq),
            String::from_utf8_lossy(qual),
        )];
    }

    // 正向
    let fwd_norm = dna::normalize_seq(seq);
    let fwd_alpha: Vec<u8> = fwd_norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    // 反向互补
    let rev_seq = dna::revcomp(seq);
    let rev_norm = dna::normalize_seq(&rev_seq);
    let rev_alpha: Vec<u8> = rev_norm.iter().map(|&b| dna::to_alphabet(b)).collect();

    let mut all_candidates: Vec<AlignCandidate> = Vec::new();

    // 正向对齐候选
    collect_candidates(fm, &fwd_norm, &fwd_alpha, sw_params, false, opt, &mut all_candidates);
    // 反向互补对齐候选
    collect_candidates(fm, &rev_norm, &rev_alpha, sw_params, true, opt, &mut all_candidates);

    if all_candidates.is_empty() || all_candidates[0].score < opt.score_threshold {
        return vec![format!(
            "{}\t4\t*\t0\t0\t*\t*\t0\t0\t{}\t{}",
            qname,
            String::from_utf8_lossy(seq),
            String::from_utf8_lossy(qual),
        )];
    }

    // 按得分降序排列
    all_candidates.sort_by(|a, b| b.score.cmp(&a.score));

    // 去重：位置和方向相同的只保留得分最高的
    dedup_candidates(&mut all_candidates);

    let mut sam_lines = Vec::new();

    // 预生成正向和反向互补的 SEQ/QUAL 字符串
    let seq_fwd = String::from_utf8_lossy(seq).to_string();
    let qual_fwd = String::from_utf8_lossy(qual).to_string();
    let rc_seq = dna::revcomp(seq);
    let seq_rev = String::from_utf8_lossy(&rc_seq).to_string();
    let qual_rev: String = qual.iter().rev().map(|&b| b as char).collect();

    let best_score = all_candidates[0].score;
    let second_best_score = if all_candidates.len() > 1 {
        all_candidates[1].score
    } else {
        0
    };

    for (idx, cand) in all_candidates.iter().enumerate() {
        if cand.score < opt.score_threshold {
            break;
        }

        let mut flag: u16 = 0;
        if cand.is_rev {
            flag |= 0x10; // reverse complement
        }

        if idx == 0 {
            // 主比对
        } else if cand.score == best_score {
            flag |= 0x100; // secondary
        } else {
            flag |= 0x100; // secondary
        }

        let mapq = if idx == 0 {
            compute_mapq(best_score, second_best_score)
        } else {
            0
        };

        let sub_score = if idx == 0 {
            second_best_score
        } else {
            best_score
        };

        // SAM 规范：FLAG 含 0x10 时，SEQ 为原始 read 的反向互补，QUAL 反转
        let (out_seq, out_qual) = if cand.is_rev {
            (&seq_rev, &qual_rev)
        } else {
            (&seq_fwd, &qual_fwd)
        };

        sam_lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t*\t0\t0\t{}\t{}\tAS:i:{}\tXS:i:{}\tNM:i:{}",
            qname,
            flag,
            cand.rname,
            cand.pos1,
            mapq,
            cand.cigar,
            out_seq,
            out_qual,
            cand.score,
            sub_score,
            cand.nm,
        ));

        // 限制输出的比对数量
        if idx >= 4 {
            break;
        }
    }

    sam_lines
}

#[derive(Debug, Clone)]
struct AlignCandidate {
    score: i32,
    is_rev: bool,
    rname: String,
    pos1: u32,
    cigar: String,
    nm: u32,
    contig_idx: usize,
}

fn collect_candidates(
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

    for ch in &chains {
        let ci = ch.contig;
        let contig = &fm.contigs[ci];
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

        let res = chain_to_alignment_buf(ch, query_norm, &ref_seq, sw_params, &mut sw_buf);
        if res.score <= 0 || res.cigar.is_empty() {
            continue;
        }

        candidates.push(AlignCandidate {
            score: res.score,
            is_rev,
            rname: contig.name.clone(),
            pos1: (res.ref_start as u32) + 1,
            cigar: res.cigar,
            nm: res.nm,
            contig_idx: ci,
        });
    }
}

fn dedup_candidates(candidates: &mut Vec<AlignCandidate>) {
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

/// BWA 风格的 MAPQ 计算
/// 参考 BWA mem_approx_mapq_se: mapq = MEM_MAPQ_COEF * (1 - sub/best) * ln(best)
/// MEM_MAPQ_COEF = 30, MEM_MAPQ_MAX = 60
fn compute_mapq(best_score: i32, second_best_score: i32) -> u8 {
    const MAPQ_COEF: f64 = 30.0;
    const MAPQ_MAX: u8 = 60;

    if best_score <= 0 {
        return 0;
    }

    let best = best_score as f64;

    if second_best_score <= 0 {
        // 唯一比对：q = coef * ln(best)，上限 MAPQ_MAX
        let q = (MAPQ_COEF * best.ln()).round() as i32;
        return (q.clamp(0, MAPQ_MAX as i32)) as u8;
    }

    let sub = second_best_score as f64;
    let ratio = sub / best;
    // q = coef * (1 - sub/best) * ln(best)
    let q = (MAPQ_COEF * (1.0 - ratio) * best.ln()).round() as i32;
    (q.clamp(0, MAPQ_MAX as i32)) as u8
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
    use crate::index::{bwt, sa};
    use crate::index::fm::{Contig, FMIndex};
    use crate::io::fastq::FastqRecord;
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

    fn default_opt() -> AlignOpt {
        AlignOpt {
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
        let sa_arr = sa::build_sa(&text);
        let bwt_arr = bwt::build_bwt(&text, &sa_arr);
        FMIndex::build(text, bwt_arr, sa_arr, contigs, dna::SIGMA as u8, 4)
    }

    #[test]
    fn sw_perfect_match() {
        let p = default_params();
        let res = banded_sw(b"ACGT", b"ACGT", p);
        assert_eq!(res.score, 8);
        assert_eq!(res.cigar, "4M");
        assert_eq!(res.nm, 0);
    }

    #[test]
    fn sw_single_mismatch_still_aligns_full() {
        let p = default_params();
        let res = banded_sw(b"AGGT", b"ACGT", p);
        assert_eq!(res.cigar, "4M");
        assert_eq!(res.score, 3 * 2 - 1);
        assert_eq!(res.nm, 1);
    }

    #[test]
    fn sw_single_insertion() {
        let p = default_params();
        let res = banded_sw(b"ACGGT", b"ACGT", p);
        assert_eq!(res.score, 7);
        assert_eq!(res.cigar, "2M1I2M");
        assert_eq!(res.nm, 1);
    }

    #[test]
    fn mapq_model() {
        // 唯一比对：q = 30 * ln(best)，上限 60
        assert!(compute_mapq(50, 0) > 50);
        assert!(compute_mapq(100, 0) == 60);
        // 有次优：q = 30 * (1 - sub/best) * ln(best)
        assert!(compute_mapq(50, 25) > 0);
        // 相同分数 -> 0
        assert_eq!(compute_mapq(10, 10), 0);
        assert_eq!(compute_mapq(100, 100), 0);
        // 无效分数
        assert_eq!(compute_mapq(0, 0), 0);
        assert_eq!(compute_mapq(-5, 0), 0);
        // 唯一比对且分数较高
        assert!(compute_mapq(30, 0) > 30);
    }

    #[test]
    fn mem_seeds_basic() {
        let fm = build_test_fm(b"ACGTACGT");
        let read = b"CGTA";
        let norm = dna::normalize_seq(read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_mem_seeds(&fm, &alpha, 2);
        assert!(seeds.iter().any(|s| s.contig == 0 && s.qe - s.qb >= 4));
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
    fn align_single_read_unmapped() {
        let fm = build_test_fm(b"ACGTACGTACGTACGTACGTACGT");
        let rec = FastqRecord {
            id: "unmapped".to_string(),
            desc: None,
            seq: b"TTTTTTTTTTTTTTTTTTTT".to_vec(),
            qual: b"IIIIIIIIIIIIIIIIIIII".to_vec(),
        };
        let sw = SwParams {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 16,
        };
        let opt = default_opt();
        let lines = align_single_read(&fm, &rec, sw, &opt);
        assert!(!lines.is_empty());
        assert!(lines[0].contains("\t4\t")); // FLAG=4 unmapped
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

    #[test]
    fn sw_deletion() {
        let p = default_params();
        let res = banded_sw(b"ACGT", b"ACGGT", p);
        assert!(res.score > 0);
    }

    #[test]
    fn sw_empty_inputs() {
        let p = default_params();
        assert_eq!(banded_sw(b"", b"ACGT", p).score, 0);
        assert_eq!(banded_sw(b"ACGT", b"", p).score, 0);
    }

    #[test]
    fn dedup_candidates_removes_duplicates() {
        let mut cands = vec![
            AlignCandidate { score: 50, is_rev: false, rname: "chr1".into(), pos1: 10, cigar: "20M".into(), nm: 0, contig_idx: 0 },
            AlignCandidate { score: 40, is_rev: false, rname: "chr1".into(), pos1: 10, cigar: "20M".into(), nm: 1, contig_idx: 0 },
            AlignCandidate { score: 45, is_rev: true, rname: "chr1".into(), pos1: 10, cigar: "20M".into(), nm: 0, contig_idx: 0 },
        ];
        dedup_candidates(&mut cands);
        assert_eq!(cands.len(), 2); // same pos+dir removed, different dir kept
    }
}
