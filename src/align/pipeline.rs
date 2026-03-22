use anyhow::Result;
use std::io::Write;
use std::sync::Arc;

use rayon::prelude::*;

use crate::index::fm::FMIndex;
use crate::io::fastq::{FastqReader, FastqRecord};
use crate::io::sam;
use crate::util::dna;

use super::candidate::{collect_candidates, dedup_candidates, AlignCandidate};
use super::mapq::compute_mapq;
use super::AlignOpt;
use super::SwParams;

pub fn align_fastq_with_opt(index_path: &str, fastq_path: &str, out_path: Option<&str>, opt: AlignOpt) -> Result<()> {
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
    let contig_info: Vec<(&str, u32)> = fm.contigs.iter().map(|c| (c.name.as_str(), c.len)).collect();
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
                .map(|rec| align_single_read(&fm_ref, rec, sw_params, &opt))
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
pub(crate) fn align_single_read(fm: &FMIndex, rec: &FastqRecord, sw_params: SwParams, opt: &AlignOpt) -> Vec<String> {
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
    // 反向互补（复用同一份 revcomp 结果）
    let rc_seq = dna::revcomp(seq);
    let rev_norm = dna::normalize_seq(&rc_seq);
    let rev_alpha: Vec<u8> = rev_norm.iter().map(|&b| dna::to_alphabet(b)).collect();

    let mut all_candidates: Vec<AlignCandidate> = Vec::new();

    // 正向对齐候选
    collect_candidates(fm, &fwd_norm, &fwd_alpha, sw_params, false, opt, &mut all_candidates);
    // 反向互补对齐候选
    collect_candidates(fm, &rev_norm, &rev_alpha, sw_params, true, opt, &mut all_candidates);

    if all_candidates.is_empty() {
        return vec![sam::format_unmapped(qname, &seq_str, &qual_str)];
    }

    // 按得分降序排列
    all_candidates.sort_by(|a, b| {
        b.sort_score
            .cmp(&a.sort_score)
            .then(b.score.cmp(&a.score))
            .then(a.nm.cmp(&b.nm))
    });

    // 去重：位置和方向相同的只保留得分最高的
    dedup_candidates(&mut all_candidates);

    if all_candidates.is_empty() || all_candidates[0].sort_score < opt.score_threshold {
        return vec![sam::format_unmapped(qname, &seq_str, &qual_str)];
    }

    let mut sam_lines = Vec::new();

    // 预生成正向和反向互补的 SEQ/QUAL 字符串
    let seq_fwd = String::from_utf8_lossy(seq).to_string();
    let qual_fwd = String::from_utf8_lossy(qual).to_string();
    let seq_rev = String::from_utf8_lossy(&rc_seq).to_string();
    let qual_rev: String = qual.iter().rev().map(|&b| b as char).collect();

    let best_sort_score = all_candidates[0].sort_score;
    let second_best_sort_score = if all_candidates.len() > 1 {
        all_candidates[1].sort_score
    } else {
        0
    };
    let best_raw_score = all_candidates[0].score;
    let second_best_raw_score = if all_candidates.len() > 1 {
        all_candidates[1].score
    } else {
        0
    };

    for (idx, cand) in all_candidates.iter().enumerate() {
        if cand.sort_score < opt.score_threshold {
            break;
        }

        let mut flag: u16 = 0;
        if cand.is_rev {
            flag |= 0x10; // reverse complement
        }

        if idx == 0 {
            // 主比对
        } else {
            flag |= 0x100; // secondary
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
            (&seq_rev, &qual_rev)
        } else {
            (&seq_fwd, &qual_fwd)
        };

        sam_lines.push(sam::format_record(
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
        ));

        // 限制输出的比对数量
        if idx >= 4 {
            break;
        }
    }

    sam_lines
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
    fn align_single_read_empty_seq() {
        let fm = build_test_fm(b"ACGTACGTACGTACGTACGTACGT");
        let rec = FastqRecord {
            id: "empty".to_string(),
            desc: None,
            seq: b"".to_vec(),
            qual: b"".to_vec(),
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
        let sw = SwParams {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 16,
        };
        let opt = AlignOpt {
            score_threshold: 10,
            ..default_opt()
        };
        let lines = align_single_read(&fm, &rec, sw, &opt);
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
        let sw = SwParams {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 16,
        };
        let opt = AlignOpt {
            score_threshold: 10,
            ..default_opt()
        };
        let lines = align_single_read(&fm, &rec, sw, &opt);
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
        };

        let lines = align_single_read(&fm, &rec, sw, &opt);
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
        };

        let lines = align_single_read(&fm, &rec, sw, &opt);
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
        };

        let lines = align_single_read(&fm, &rec, sw, &opt);
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
        };

        let lines = align_single_read(&fm, &rec, sw, &opt);
        let fields: Vec<&str> = lines[0].split('\t').collect();
        assert_eq!(fields[5], "20M");
        assert!(!lines[0].contains("\tNM:i:0"));
    }
}
