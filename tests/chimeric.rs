//! 默认参数下的部分匹配与嵌合 read 回归测试。
//!
//! 默认打分必须让非同源尾部被软裁剪，而不是沿随机匹配继续延伸；
//! 两个不相邻的精确片段必须输出 supplementary 记录并带 SA:Z。

use std::io::Cursor;
use std::sync::Arc;

use bwa_rust::align::{align_fastq_with_fm, AlignOpt};
use bwa_rust::index::builder::build_fm_index;
use bwa_rust::io::fastq::FastqReader;

/// 确定性伪随机 ACGT 序列（LCG），避免在测试中引入随机数依赖。
fn pseudo_random(mut state: u64, len: usize) -> Vec<u8> {
    let mut seq = Vec::with_capacity(len);
    for _ in 0..len {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        seq.push(b"ACGT"[((state >> 33) & 3) as usize]);
    }
    seq
}

fn align(reference: &[u8], reads: &[u8]) -> Vec<String> {
    let mut fasta = b">chr1\n".to_vec();
    fasta.extend_from_slice(reference);
    fasta.push(b'\n');
    let fm = Arc::new(build_fm_index(Cursor::new(&fasta[..]), 16).unwrap().fm);
    let reader = FastqReader::new(Cursor::new(reads));
    let mut out = Vec::new();
    align_fastq_with_fm(reader, &mut out, &fm, &AlignOpt::default()).unwrap();
    String::from_utf8(out)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with('@'))
        .map(str::to_string)
        .collect()
}

fn fastq_record(name: &str, seq: &[u8]) -> Vec<u8> {
    let mut fq = format!("@{}\n", name).into_bytes();
    fq.extend_from_slice(seq);
    fq.extend_from_slice(b"\n+\n");
    fq.extend_from_slice(&vec![b'I'; seq.len()]);
    fq.push(b'\n');
    fq
}

fn fields(record: &str) -> Vec<&str> {
    record.split('\t').collect()
}

#[test]
fn partial_match_is_soft_clipped_at_default_params() {
    let reference = pseudo_random(1, 600);
    let mut read = reference[50..90].to_vec();
    read.extend_from_slice(&pseudo_random(9_999, 40));

    let records = align(&reference, &fastq_record("partial", &read));
    assert_eq!(records.len(), 1, "expected one alignment, got {:?}", records);
    let f = fields(&records[0]);
    assert_eq!(f[5], "40M40S", "expected soft-clipped partial match, got {:?}", records);
    assert!(
        records[0].contains("NM:i:0"),
        "unrelated tail must not add mismatches: {:?}",
        records
    );
}

#[test]
fn chimeric_read_emits_supplementary_with_sa_tag() {
    let reference = pseudo_random(2, 600);
    let mut read = reference[50..110].to_vec();
    read.extend_from_slice(&reference[400..460]);

    let records = align(&reference, &fastq_record("chimeric", &read));
    assert_eq!(records.len(), 2, "expected primary + supplementary, got {:?}", records);

    let flags: Vec<u16> = records.iter().map(|r| fields(r)[1].parse().unwrap()).collect();
    assert!(
        flags.iter().any(|flag| flag & 0x800 != 0),
        "expected a supplementary record, got flags {:?}",
        flags
    );
    let cigars: Vec<&str> = records.iter().map(|r| fields(r)[5]).collect();
    assert!(
        cigars.contains(&"60M60S") && cigars.contains(&"60S60M"),
        "expected split 60bp alignments, got {:?}",
        cigars
    );
    assert!(
        records.iter().all(|r| r.contains("SA:Z:")),
        "primary and supplementary must carry SA:Z: {:?}",
        records
    );

    // SA:Z 中的 mapQ 必须是它指向的那条记录自身的 MAPQ，而不是占位 0。
    let mapqs: Vec<u8> = records.iter().map(|r| fields(r)[4].parse().unwrap()).collect();
    for (i, record) in records.iter().enumerate() {
        let sa = record.split("SA:Z:").nth(1).expect("SA:Z tag").split(';').next().unwrap();
        let sa_mapq: u8 = sa.split(',').nth(4).unwrap().parse().unwrap();
        assert_eq!(
            sa_mapq,
            mapqs[1 - i],
            "SA:Z mapQ must match the referenced record's MAPQ: {:?}",
            records
        );
    }
}
