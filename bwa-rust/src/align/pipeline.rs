use anyhow::Result;
use std::io::Write;
use std::sync::Arc;

use rayon::prelude::*;

use crate::index::fm::FMIndex;
use crate::io::fastq::{FastqReader, FastqRecord};
use crate::io::sam;
use crate::util::dna;

use super::{AlignOpt, SwParams, chain_to_alignment_buf};
use super::{build_chains, filter_chains, find_smem_seeds};
use super::sw;

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
    let contig_info: Vec<(&str, u32)> = fm.contigs.iter()
        .map(|c| (c.name.as_str(), c.len))
        .collect();
    sam::write_header(&mut out_box, &contig_info)?;

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
pub(crate) fn align_single_read(
    fm: &FMIndex,
    rec: &FastqRecord,
    sw_params: SwParams,
    opt: &AlignOpt,
) -> Vec<String> {
    let qname = &rec.id;
    let seq = &rec.seq;
    let qual = &rec.qual;

    let seq_str = String::from_utf8_lossy(seq);
    let qual_str = String::from_utf8_lossy(qual);

    if seq.is_empty() {
        return vec![sam::format_unmapped(qname, &seq_str, &qual_str)];
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
        return vec![sam::format_unmapped(qname, &seq_str, &qual_str)];
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

        sam_lines.push(sam::format_record(
            qname, flag, &cand.rname, cand.pos1, mapq, &cand.cigar,
            out_seq, out_qual, cand.score, sub_score, cand.nm,
        ));

        // 限制输出的比对数量
        if idx >= 4 {
            break;
        }
    }

    sam_lines
}

#[derive(Debug, Clone)]
pub(crate) struct AlignCandidate {
    pub score: i32,
    pub is_rev: bool,
    pub rname: String,
    pub pos1: u32,
    pub cigar: String,
    pub nm: u32,
    pub contig_idx: usize,
}

pub(crate) fn collect_candidates(
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

pub(crate) fn dedup_candidates(candidates: &mut Vec<AlignCandidate>) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{bwt, sa};
    use crate::index::fm::{Contig, FMIndex};
    use crate::io::fastq::FastqRecord;
    use crate::util::dna;

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
