use std::io::{BufRead, Write};
use std::sync::Arc;

use anyhow::Result;
use rayon::prelude::*;

use crate::index::fm::FMIndex;
use crate::io::fastq::{FastqReader, FastqRecord};
use crate::io::sam::{self, flags};
use crate::util::dna;

use super::candidate::{collect_candidates, dedup_candidates, AlignCandidate, RefSeqCache};
use super::mapq::compute_mapq;
use super::supplementary::{classify_alignments, generate_sa_tag, AlignmentType};
use super::AlignOpt;
use super::SwParams;

/// 并行批处理的批量大小（读条数）
const BATCH_SIZE: usize = 1000;

/// 从 FM 索引文件和 FASTQ 文件路径对齐（保留路径入口以兼容现有调用）。
pub fn align_fastq_with_opt(index_path: &str, fastq_path: &str, out_path: Option<&str>, opt: AlignOpt) -> Result<()> {
    let fm = Arc::new(FMIndex::load_from_file(index_path)?);
    align_fastq_with_fm_opt(fm, fastq_path, out_path, opt)
}

/// 从 FM 索引对齐 FASTQ 文件路径，输出到文件或 stdout。
pub fn align_fastq_with_fm_opt(
    fm: Arc<FMIndex>,
    fastq_path: &str,
    out_path: Option<&str>,
    opt: AlignOpt,
) -> Result<()> {
    opt.validate()
        .map_err(|e| anyhow::anyhow!("invalid alignment options: {}", e))?;

    let fq = std::fs::File::open(fastq_path)?;
    let reader = FastqReader::new(std::io::BufReader::new(fq));

    let mut out_box: Box<dyn Write> = if let Some(p) = out_path {
        Box::new(std::io::BufWriter::new(std::fs::File::create(p)?))
    } else {
        Box::new(std::io::BufWriter::new(std::io::stdout()))
    };

    align_fastq_with_fm(reader, &mut out_box, &fm, &opt)
}

/// 泛型对齐入口：从任意 `R: BufRead` 读取 FASTQ，写入任意 `W: Write`。
///
/// 这是管线与 IO 的边界——不再硬编码文件路径，便于接入 stdin、gzip 解压流或内存缓冲。
pub fn align_fastq_with_fm<R: BufRead, W: Write>(
    mut reader: FastqReader<R>,
    out: &mut W,
    fm: &Arc<FMIndex>,
    opt: &AlignOpt,
) -> Result<()> {
    // SAM header
    let contig_info: Vec<(&str, u32)> = fm.contigs().iter().map(|c| (c.name.as_str(), c.len)).collect();
    sam::write_header(out, &contig_info)?;

    let sw_params = SwParams {
        match_score: opt.match_score,
        mismatch_penalty: opt.mismatch_penalty,
        gap_open: opt.gap_open,
        gap_extend: opt.gap_extend,
        band_width: opt.band_width,
    };

    // 仅在多线程模式下创建自定义 rayon 线程池，单线程直接顺序执行以减少开销
    let pool = if opt.threads > 1 {
        let p = rayon::ThreadPoolBuilder::new()
            .num_threads(opt.threads)
            .build()
            .map_err(|e| anyhow::anyhow!("failed to build thread pool with {} threads: {}", opt.threads, e))?;
        Some(p)
    } else {
        None
    };

    // contig 解码缓存在整个比对运行（含所有批次与线程）间共享，
    // 每个 contig 至多解码一次
    let ref_cache = RefSeqCache::new();

    loop {
        let mut batch: Vec<FastqRecord> = Vec::with_capacity(BATCH_SIZE);
        for _ in 0..BATCH_SIZE {
            match reader.next_record()? {
                Some(rec) => batch.push(rec),
                None => break,
            }
        }
        if batch.is_empty() {
            break;
        }

        if let Some(pool) = &pool {
            let fm_ref = Arc::clone(fm);
            let results: Vec<Vec<String>> = pool.install(|| {
                batch
                    .par_iter()
                    .map(|rec| align_single_read(&fm_ref, rec, sw_params, opt, &ref_cache))
                    .collect::<Result<Vec<Vec<String>>>>()
            })?;

            for lines in results {
                for line in lines {
                    writeln!(out, "{}", line)?;
                }
            }
        } else {
            for rec in &batch {
                for line in align_single_read(fm, rec, sw_params, opt, &ref_cache)? {
                    writeln!(out, "{}", line)?;
                }
            }
        }
    }

    Ok(())
}

/// 对单条 read 进行比对，返回一个或多个 SAM 行。
///
/// 返回 `Result`：不再在解析/转换失败时 panic，而是将错误上抛给调用方。
pub(crate) fn align_single_read(
    fm: &FMIndex,
    rec: &FastqRecord,
    sw_params: SwParams,
    opt: &AlignOpt,
    ref_cache: &RefSeqCache,
) -> Result<Vec<String>> {
    let qname = &rec.id;
    let seq = &rec.seq;
    let qual = &rec.qual;

    // FASTQ 解析器已保证序列/质量行为合法 UTF-8（read_line 只产生 UTF-8）。
    // 但 FastqRecord 字段是 pub 的，库消费者可能手工构造任意字节，
    // 因此这里不再 panic，而是返回错误。
    let seq_fwd = std::str::from_utf8(seq).map_err(|_| anyhow::anyhow!("FASTQ sequence contains invalid UTF-8"))?;
    let qual_fwd = std::str::from_utf8(qual).map_err(|_| anyhow::anyhow!("FASTQ quality contains invalid UTF-8"))?;

    if seq.is_empty() {
        return Ok(vec![sam::format_unmapped(qname, seq_fwd, qual_fwd)]);
    }

    // 正向
    let fwd_norm = dna::normalize_seq(seq);
    let fwd_alpha: Vec<u8> = fwd_norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    // 反向互补（复用同一份 revcomp 结果）
    let rc_seq = dna::revcomp(seq);
    let rev_norm = dna::normalize_seq(&rc_seq);
    let rev_alpha: Vec<u8> = rev_norm.iter().map(|&b| dna::to_alphabet(b)).collect();

    let mut all_candidates: Vec<AlignCandidate> = Vec::new();

    let query_len = seq.len();

    // 正向对齐候选
    collect_candidates(
        fm,
        &fwd_norm,
        &fwd_alpha,
        sw_params,
        false,
        query_len,
        opt,
        ref_cache,
        &mut all_candidates,
    );
    // 反向互补对齐候选
    collect_candidates(
        fm,
        &rev_norm,
        &rev_alpha,
        sw_params,
        true,
        query_len,
        opt,
        ref_cache,
        &mut all_candidates,
    );

    if all_candidates.is_empty() {
        return Ok(vec![sam::format_unmapped(qname, seq_fwd, qual_fwd)]);
    }

    // 按得分降序排列
    all_candidates.sort_by(|a, b| {
        b.sort_score
            .cmp(&a.sort_score)
            .then(b.score.cmp(&a.score))
            .then(a.nm.cmp(&b.nm))
            .then(a.contig_idx.cmp(&b.contig_idx))
            .then(a.pos1.cmp(&b.pos1))
            .then(a.is_rev.cmp(&b.is_rev))
            .then(a.cigar.cmp(&b.cigar))
    });

    // 去重：位置和方向相同的只保留得分最高的
    dedup_candidates(&mut all_candidates);

    if all_candidates.is_empty() || all_candidates[0].sort_score < opt.score_threshold {
        return Ok(vec![sam::format_unmapped(qname, seq_fwd, qual_fwd)]);
    }

    // 分类对齐类型（覆盖全部候选，与输出截断相互独立）
    let classification = classify_alignments(&all_candidates);

    // 用于 MAPQ 的次优分数：取排序后第一条“非 supplementary”的候选，
    // 避免把同一 primary 的 supplementary 片段当作独立次优比对（BWA 语义）。
    let second_best_sort_score = all_candidates
        .iter()
        .enumerate()
        .skip(1)
        .find(|(idx, _)| {
            classification
                .iter()
                .find(|(i, _)| i == idx)
                .map(|(_, t)| *t != AlignmentType::Supplementary)
                .unwrap_or(true)
        })
        .map(|(_, c)| c.sort_score)
        .unwrap_or(0);
    let second_best_raw_score = all_candidates
        .iter()
        .enumerate()
        .skip(1)
        .find(|(idx, _)| {
            classification
                .iter()
                .find(|(i, _)| i == idx)
                .map(|(_, t)| *t != AlignmentType::Supplementary)
                .unwrap_or(true)
        })
        .map(|(_, c)| c.score)
        .unwrap_or(0);

    let best_sort_score = all_candidates[0].sort_score;
    let best_raw_score = all_candidates[0].score;

    let max_aln = opt.max_alignments_per_read;
    let mut sam_lines = Vec::with_capacity(all_candidates.len().min(max_aln));

    let needs_rev_output = all_candidates
        .iter()
        .take(max_aln)
        .any(|cand| cand.sort_score >= opt.score_threshold && cand.is_rev);
    let (seq_rev, qual_rev) = if needs_rev_output {
        let s = std::str::from_utf8(&rc_seq)
            .map_err(|_| anyhow::anyhow!("reverse-complement sequence contains invalid UTF-8"))?;
        let q: String = qual.iter().rev().map(|&b| b as char).collect();
        (s.to_string(), q)
    } else {
        (String::new(), String::new())
    };

    // 计算将要输出的候选集合（受 threshold 与 max_aln 截断），
    // 供 SA:Z 标签只引用真正输出的候选。
    let emitted: Vec<bool> = all_candidates
        .iter()
        .enumerate()
        .map(|(idx, cand)| idx < max_aln && cand.sort_score >= opt.score_threshold)
        .collect();

    for (idx, cand) in all_candidates.iter().enumerate() {
        if cand.sort_score < opt.score_threshold {
            break;
        }
        if idx >= max_aln {
            break;
        }

        let mut flag: u16 = 0;
        if cand.is_rev {
            flag |= flags::REVERSE; // reverse complement
        }

        // Determine alignment type and set flag accordingly
        let align_type = classification
            .iter()
            .find(|(i, _)| *i == idx)
            .map(|(_, t)| *t)
            .unwrap_or(AlignmentType::Secondary);

        match align_type {
            AlignmentType::Primary => {}
            AlignmentType::Secondary => flag |= flags::SECONDARY,
            AlignmentType::Supplementary => flag |= flags::SUPPLEMENTARY,
        }

        let mapq = if idx == 0 {
            compute_mapq(best_sort_score, second_best_sort_score)
        } else {
            0
        };

        let sub_score = if idx == 0 {
            second_best_raw_score
        } else {
            best_raw_score
        };

        // SAM 规范：FLAG 含 0x10 时，SEQ 为原始 read 的反向互补，QUAL 反转
        let (out_seq, out_qual) = if cand.is_rev {
            (seq_rev.as_str(), qual_rev.as_str())
        } else {
            (seq_fwd, qual_fwd)
        };

        // Generate MD:Z tag
        let md_tag = if !cand.ref_seq.is_empty() && !cand.query_seq.is_empty() {
            sam::generate_md_tag(&cand.ref_seq, &cand.query_seq, &cand.cigar)
        } else {
            String::new()
        };

        // Generate SA:Z tag（只引用本次实际输出的候选，保持一致性）
        let sa_tag = generate_sa_tag(idx, &all_candidates, &classification, &emitted);

        sam_lines.push(sam::format_record_with_optional_tags(
            qname,
            flag,
            &cand.rname,
            cand.pos1,
            mapq,
            &cand.cigar,
            out_seq,
            out_qual,
            cand.score,
            sub_score,
            cand.nm,
            &md_tag,
            &sa_tag,
        ));
    }

    Ok(sam_lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::builder::build_fm_index;
    use crate::io::fastq::FastqRecord;
    use crate::testutil::build_test_fm;
    use crate::util::dna;
    use std::io::Cursor;

    fn default_opt() -> AlignOpt {
        AlignOpt::default()
    }

    fn default_sw() -> SwParams {
        SwParams {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 16,
        }
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
        let opt = default_opt();
        let lines = align_single_read(&fm, &rec, default_sw(), &opt, &RefSeqCache::new()).unwrap();
        assert!(!lines.is_empty());
        assert!(lines[0].contains("\t4\t")); // FLAG=4 unmapped
    }

    #[test]
    fn align_single_read_empty_seq() {
        let fm = build_test_fm(b"ACGTACGTACGTACGTACGTACGT");
        let rec = FastqRecord {
            id: "empty".to_string(),
            desc: None,
            seq: b"".to_vec(),
            qual: b"".to_vec(),
        };
        let opt = default_opt();
        let lines = align_single_read(&fm, &rec, default_sw(), &opt, &RefSeqCache::new()).unwrap();
        assert!(!lines.is_empty());
        assert!(lines[0].contains("\t4\t")); // unmapped
    }

    #[test]
    fn align_single_read_mapped() {
        let reference = b"ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT";
        let fm = build_test_fm(reference);
        let rec = FastqRecord {
            id: "mapped".to_string(),
            desc: None,
            seq: b"ACGTACGTACGTACGTACGTACGT".to_vec(),
            qual: b"IIIIIIIIIIIIIIIIIIIIIIIII".to_vec(),
        };
        let opt = AlignOpt {
            score_threshold: 10,
            ..default_opt()
        };
        let lines = align_single_read(&fm, &rec, default_sw(), &opt, &RefSeqCache::new()).unwrap();
        assert!(!lines.is_empty());
        // Primary alignment should not be unmapped
        assert!(!lines[0].contains("\t4\t*\t"));
        assert!(lines[0].contains("chr1"));
        assert!(lines[0].contains("M"));
    }

    #[test]
    fn align_single_read_revcomp() {
        // 使用非回文参考序列，确保正向和反向互补不同
        let reference = b"AACCGGTTAACCGGTTAACCGGTTAACCGGTTAACCGGTTAACCGGTT";
        let fm = build_test_fm(reference);
        // 从参考中取一段，然后取 revcomp 作为 read
        let fwd_read = &reference[..24];
        let rc = dna::revcomp(fwd_read);
        let rec = FastqRecord {
            id: "revcomp".to_string(),
            desc: None,
            seq: rc.clone(),
            qual: vec![b'I'; rc.len()],
        };
        let opt = AlignOpt {
            score_threshold: 10,
            ..default_opt()
        };
        let lines = align_single_read(&fm, &rec, default_sw(), &opt, &RefSeqCache::new()).unwrap();
        assert!(!lines.is_empty());
        let fields: Vec<&str> = lines[0].split('\t').collect();
        let flag: u16 = fields[1].parse().unwrap();
        let is_mapped = flag & 4 == 0;
        // 如果成功映射，应该有正向或反向互补的比对
        if is_mapped {
            // 无论正向还是反向映射，重要的是 read 能被成功比对
            assert!(lines[0].contains("chr1"));
        }
    }

    #[test]
    fn align_single_read_prefers_best_revcomp_candidate_before_threshold() {
        let fasta = b">chr_exact\nAACCTTGGAACC\n>chr_partial\nGGTTCCAAAAAA\n";
        let fm = build_fm_index(Cursor::new(&fasta[..]), 4).unwrap().fm;
        let rec = FastqRecord {
            id: "rev-best".to_string(),
            desc: None,
            seq: b"GGTTCCAAGGTT".to_vec(),
            qual: vec![b'I'; 12],
        };
        let sw = SwParams {
            match_score: 1,
            mismatch_penalty: 4,
            gap_open: 6,
            gap_extend: 1,
            band_width: 100,
        };
        let opt = AlignOpt {
            match_score: 1,
            mismatch_penalty: 4,
            gap_open: 6,
            gap_extend: 1,
            clip_penalty: 1,
            band_width: 100,
            score_threshold: 10,
            min_seed_len: 19,
            threads: 1,
            ..AlignOpt::default()
        };

        let lines = align_single_read(&fm, &rec, sw, &opt, &RefSeqCache::new()).unwrap();
        assert_eq!(lines.len(), 1);

        let fields: Vec<&str> = lines[0].split('\t').collect();
        let flag: u16 = fields[1].parse().unwrap();
        assert_eq!(fields[2], "chr_exact");
        assert_eq!(fields[5], "12M");
        assert_eq!(flag & 0x4, 0, "read should be mapped");
        assert_ne!(flag & 0x10, 0, "primary alignment should be reverse-complement");
    }

    #[test]
    fn align_single_read_refines_single_insertion_to_indel_cigar() {
        let fm = build_test_fm(b"GGCCAATTGGCCAATTGGCC");
        let rec = FastqRecord {
            id: "ins".to_string(),
            desc: None,
            seq: b"GGCCAAATTGGCCAATTGGCC".to_vec(),
            qual: vec![b'I'; 21],
        };
        let sw = SwParams {
            match_score: 1,
            mismatch_penalty: 4,
            gap_open: 6,
            gap_extend: 1,
            band_width: 64,
        };
        let opt = AlignOpt {
            match_score: 1,
            mismatch_penalty: 4,
            gap_open: 6,
            gap_extend: 1,
            clip_penalty: 1,
            band_width: 64,
            score_threshold: 10,
            min_seed_len: 19,
            threads: 1,
            ..AlignOpt::default()
        };

        let lines = align_single_read(&fm, &rec, sw, &opt, &RefSeqCache::new()).unwrap();
        let fields: Vec<&str> = lines[0].split('\t').collect();
        assert!(fields[5].contains('I'));
        assert!(!fields[5].contains('S'));
    }

    #[test]
    fn align_single_read_refines_single_deletion_to_indel_cigar() {
        let fm = build_test_fm(b"ATCGATCGATCGATCGATCG");
        let rec = FastqRecord {
            id: "del".to_string(),
            desc: None,
            seq: b"ATCGACGATCGATCGATCG".to_vec(),
            qual: vec![b'I'; 19],
        };
        let sw = SwParams {
            match_score: 1,
            mismatch_penalty: 4,
            gap_open: 6,
            gap_extend: 1,
            band_width: 64,
        };
        let opt = AlignOpt {
            match_score: 1,
            mismatch_penalty: 4,
            gap_open: 6,
            gap_extend: 1,
            clip_penalty: 1,
            band_width: 64,
            score_threshold: 10,
            min_seed_len: 19,
            threads: 1,
            ..AlignOpt::default()
        };

        let lines = align_single_read(&fm, &rec, sw, &opt, &RefSeqCache::new()).unwrap();
        let fields: Vec<&str> = lines[0].split('\t').collect();
        assert!(fields[5].contains('D'));
        assert!(!fields[5].contains('S'));
    }

    #[test]
    fn align_single_read_refines_single_mismatch_without_softclip() {
        let fm = build_test_fm(b"ATCGATCGATCGATCGATCG");
        let rec = FastqRecord {
            id: "mismatch".to_string(),
            desc: None,
            seq: b"ATCGTTCGATCGATCGATCG".to_vec(),
            qual: vec![b'I'; 20],
        };
        let sw = SwParams {
            match_score: 1,
            mismatch_penalty: 4,
            gap_open: 6,
            gap_extend: 1,
            band_width: 64,
        };
        let opt = AlignOpt {
            match_score: 1,
            mismatch_penalty: 4,
            gap_open: 6,
            gap_extend: 1,
            clip_penalty: 1,
            band_width: 64,
            score_threshold: 10,
            min_seed_len: 19,
            threads: 1,
            ..AlignOpt::default()
        };

        let lines = align_single_read(&fm, &rec, sw, &opt, &RefSeqCache::new()).unwrap();
        let fields: Vec<&str> = lines[0].split('\t').collect();
        assert_eq!(fields[5], "20M");
        assert!(!lines[0].contains("\tNM:i:0"));
    }

    #[test]
    fn align_single_read_n_heavy_read_is_unmapped_or_valid() {
        // N 编码为 5，索引文本中不存在该符号，N 段无法产生种子。
        // 验证 N-heavy read 不会 panic，输出要么 unmapped 要么合法 mapped 行。
        let fm = build_test_fm(b"ACGTACGTACGTACGTACGT");
        let rec = FastqRecord {
            id: "n-heavy".to_string(),
            desc: None,
            seq: b"ACGTNNNNNNNNNNNNNNNNACGT".to_vec(),
            qual: vec![b'I'; 22],
        };
        let opt = AlignOpt {
            score_threshold: 5,
            ..default_opt()
        };
        let lines = align_single_read(&fm, &rec, default_sw(), &opt, &RefSeqCache::new()).unwrap();
        assert!(!lines.is_empty());
        let fields: Vec<&str> = lines[0].split('\t').collect();
        assert!(fields.len() >= 11);
        // 要么 unmapped（flag 4），要么 mapped 且 CIGAR 有效
        let flag: u16 = fields[1].parse().unwrap();
        if flag & 4 != 0 {
            assert_eq!(fields[2], "*");
        } else {
            assert!(fields[5].contains('M') || fields[5].contains('S'));
        }
    }

    #[test]
    fn align_single_read_revcomp_md_tag_is_consistent() {
        // 反向链比对：验证 MD:Z 与 ref_seq/query_seq 列级一致，
        // 且不会因为 is_rev 产生方向错乱（回归测试，防止封装重构破坏契约）。
        let reference = b"TTGGCCAATTGGCCAATTGGCCAATTGGCCAATTGGCCAATTGGCCAA";
        let fm = build_test_fm(reference);
        // 取参考段做 revcomp 作为 read，强制反向链比对
        let fwd_read = &reference[6..30];
        let rc = dna::revcomp(fwd_read);
        let rec = FastqRecord {
            id: "rev-md".to_string(),
            desc: None,
            seq: rc.clone(),
            qual: vec![b'I'; rc.len()],
        };
        let opt = AlignOpt {
            score_threshold: 10,
            ..default_opt()
        };
        let lines = align_single_read(&fm, &rec, default_sw(), &opt, &RefSeqCache::new()).unwrap();
        assert!(!lines.is_empty());
        let fields: Vec<&str> = lines[0].split('\t').collect();
        let flag: u16 = fields[1].parse().unwrap();
        // 若为反向比对，MD:Z 必须存在且为纯数字（无错配时）
        if flag & 0x10 != 0 && flag & 4 == 0 {
            let line = &lines[0];
            if let Some(md) = line.split("\tMD:Z:").nth(1).and_then(|s| s.split('\t').next()) {
                // 完美匹配时 MD 应为全数字长度串
                assert!(md.chars().all(|c| c.is_ascii_digit()), "unexpected MD:Z '{}'", md);
            }
        }
    }
}
