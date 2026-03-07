//! 集成测试：端到端验证 bwa-rust 的索引构建和比对流程

use std::io::Cursor;

use bwa_rust::index::builder::build_fm_index;
use bwa_rust::index::fm::FMIndex;
use bwa_rust::index::{sa, bwt};
use bwa_rust::align::{
    SwParams, find_smem_seeds, build_chains, filter_chains,
    chain_to_alignment,
};
use bwa_rust::io::fasta::FastaReader;
use bwa_rust::io::fastq::FastqReader;
use bwa_rust::io::sam;
use bwa_rust::util::dna;

/// 辅助函数：从 ASCII 序列构建 FM 索引
fn build_fm_from_seq(seq: &[u8]) -> FMIndex {
    let norm = dna::normalize_seq(seq);
    let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    let len = text.len() as u32;
    let contigs = vec![bwa_rust::index::fm::Contig {
        name: "chr1".to_string(),
        len,
        offset: 0,
    }];
    text.push(0);
    let sa_arr = sa::build_sa(&text);
    let bwt_arr = bwt::build_bwt(&text, &sa_arr);
    FMIndex::build(text, bwt_arr, sa_arr, contigs, dna::SIGMA as u8, 16)
}

/// 辅助函数：从多条序列构建 FM 索引
fn build_fm_from_fasta_str(fasta: &[u8]) -> FMIndex {
    let cursor = Cursor::new(fasta);
    let result = build_fm_index(cursor, 16).unwrap();
    result.fm
}

// ─── 端到端：FASTA -> FM 索引 -> 精确搜索 ────────────────────

#[test]
fn e2e_build_index_and_exact_search() {
    let fasta = b">chr1\nACGTACGTACGTACGTACGT\n>chr2\nGGCCAATTGGCCAATT\n";
    let fm = build_fm_from_fasta_str(fasta);

    assert_eq!(fm.contigs.len(), 2);
    assert_eq!(fm.contigs[0].name, "chr1");
    assert_eq!(fm.contigs[1].name, "chr2");

    // 在 chr1 中精确搜索
    let pat: Vec<u8> = b"ACGTACGT".iter().map(|&b| dna::to_alphabet(b)).collect();
    let res = fm.backward_search(&pat);
    assert!(res.is_some());
    let (l, r) = res.unwrap();
    let positions = fm.sa_interval_positions(l, r);
    assert!(!positions.is_empty());

    // 在 chr2 中精确搜索
    let pat2: Vec<u8> = b"GGCCAATT".iter().map(|&b| dna::to_alphabet(b)).collect();
    let res2 = fm.backward_search(&pat2);
    assert!(res2.is_some());
}

// ─── 端到端：SMEM + 链构建 + SW 对齐 ─────────────────────────

#[test]
fn e2e_seed_chain_align_exact() {
    let reference = b"ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT";
    let fm = build_fm_from_seq(reference);

    let read = b"ACGTACGTACGTACGTACGTACGT";
    let norm = dna::normalize_seq(read);
    let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();

    // SMEM
    let seeds = find_smem_seeds(&fm, &alpha, 4);
    assert!(!seeds.is_empty());

    // 链构建
    let mut chains = build_chains(&seeds, read.len());
    assert!(!chains.is_empty());
    filter_chains(&mut chains, 0.3);
    assert!(!chains.is_empty());

    // SW 对齐
    let ref_seq: Vec<u8> = fm.text[..fm.contigs[0].len as usize]
        .iter()
        .map(|&c| dna::from_alphabet(c))
        .collect();
    let p = SwParams {
        match_score: 2,
        mismatch_penalty: 1,
        gap_open: 2,
        gap_extend: 1,
        band_width: 16,
    };
    let res = chain_to_alignment(&chains[0], &norm, &ref_seq, p);
    assert!(res.score > 0);
    assert!(res.cigar.contains('M'));
    assert_eq!(res.nm, 0);
}

#[test]
fn e2e_seed_chain_align_with_mismatch() {
    let reference = b"ACGTACGTAGCTGATCGTAGCTAGCTAGCTGATCGTAGCTAGCTAGCTGAT";
    let fm = build_fm_from_seq(reference);

    // 带一个 mismatch 的 read
    let mut read = reference[5..35].to_vec();
    read[15] = if read[15] == b'A' { b'T' } else { b'A' };
    let norm = dna::normalize_seq(&read);
    let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();

    let seeds = find_smem_seeds(&fm, &alpha, 4);
    if !seeds.is_empty() {
        let mut chains = build_chains(&seeds, read.len());
        filter_chains(&mut chains, 0.3);
        if !chains.is_empty() {
            let ref_seq: Vec<u8> = fm.text[..fm.contigs[0].len as usize]
                .iter()
                .map(|&c| dna::from_alphabet(c))
                .collect();
            let p = SwParams {
                match_score: 2,
                mismatch_penalty: 1,
                gap_open: 2,
                gap_extend: 1,
                band_width: 16,
            };
            let res = chain_to_alignment(&chains[0], &norm, &ref_seq, p);
            assert!(res.score > 0);
            assert!(res.cigar.contains('M'));
        }
    }
}

// ─── 端到端：反向互补比对 ─────────────────────────────────────

#[test]
fn e2e_revcomp_alignment() {
    let reference = b"ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT";
    let fm = build_fm_from_seq(reference);

    let read_fwd = b"ACGTACGTACGTACGTACGTACGT";
    let read_rev = dna::revcomp(read_fwd);

    let norm_rev = dna::normalize_seq(&read_rev);
    let alpha_rev: Vec<u8> = norm_rev.iter().map(|&b| dna::to_alphabet(b)).collect();

    let seeds = find_smem_seeds(&fm, &alpha_rev, 4);
    // 反向互补 read 仍然可以找到 SMEM 种子（因为 ACGT 的 revcomp 也是 ACGT）
    assert!(!seeds.is_empty());
}

// ─── 端到端：SAM 输出完整性 ──────────────────────────────────

#[test]
fn e2e_sam_output_format() {
    let mut buf = Vec::new();
    let contigs = vec![("chr1", 1000u32), ("chr2", 2000u32)];
    sam::write_header(&mut buf, &contigs).unwrap();

    let unmapped = sam::format_unmapped("read1", "ACGTACGT", "IIIIIIII");
    let mapped = sam::format_record(
        "read2", 0, "chr1", 100, 60, "8M", "ACGTACGT", "IIIIIIII", 16, 0, 0,
    );

    // 验证 SAM 格式正确性
    let header = String::from_utf8(buf).unwrap();
    assert!(header.starts_with("@HD\t"));
    assert!(header.contains("@SQ\tSN:chr1\tLN:1000"));
    assert!(header.contains("@PG\tID:bwa-rust"));

    // unmapped 行
    let fields: Vec<&str> = unmapped.split('\t').collect();
    assert_eq!(fields.len(), 11);
    assert_eq!(fields[1], "4"); // FLAG

    // mapped 行
    let fields: Vec<&str> = mapped.split('\t').collect();
    assert!(fields.len() >= 11);
    assert_eq!(fields[0], "read2");
    assert_eq!(fields[2], "chr1");
    assert_eq!(fields[3], "100"); // 1-based POS
}

// ─── FASTA + FASTQ 解析联合测试 ──────────────────────────────

#[test]
fn e2e_parse_fasta_and_fastq() {
    let fasta_data = b">chr1\nACGTACGT\n>chr2\nGGCCAAGG\n";
    let fastq_data = b"@read1\nACGTACGT\n+\nIIIIIIII\n@read2\nGGCCAAGG\n+\nHHHHHHHH\n";

    let mut fasta = FastaReader::new(Cursor::new(&fasta_data[..]));
    let mut fastq = FastqReader::new(Cursor::new(&fastq_data[..]));

    let fa1 = fasta.next_record().unwrap().unwrap();
    let fq1 = fastq.next_record().unwrap().unwrap();
    assert_eq!(fa1.seq, fq1.seq); // FASTA 大写化 == FASTQ 原始

    let fa2 = fasta.next_record().unwrap().unwrap();
    let fq2 = fastq.next_record().unwrap().unwrap();
    assert_eq!(fa2.seq, fq2.seq);

    assert!(fasta.next_record().unwrap().is_none());
    assert!(fastq.next_record().unwrap().is_none());
}

// ─── FM 索引 序列化 / 反序列化 + 搜索一致性 ──────────────────

#[test]
fn e2e_fm_index_serialize_deserialize_search() {
    let fasta = b">chr1\nACGTACGTACGTACGTACGT\n";
    let fm = build_fm_from_fasta_str(fasta);

    let tmp = std::env::temp_dir().join("bwa_rust_integration_test.fm");
    let path = tmp.to_str().unwrap();
    fm.save_to_file(path).unwrap();
    let loaded = FMIndex::load_from_file(path).unwrap();

    // 搜索结果应一致
    let pat: Vec<u8> = b"CGTACGT".iter().map(|&b| dna::to_alphabet(b)).collect();
    let r1 = fm.backward_search(&pat);
    let r2 = loaded.backward_search(&pat);
    assert_eq!(r1, r2);

    if let Some((l, r)) = r1 {
        let p1 = fm.sa_interval_positions(l, r);
        let p2 = loaded.sa_interval_positions(l, r);
        assert_eq!(p1, p2);
    }

    std::fs::remove_file(path).ok();
}

// ─── DNA 工具函数综合测试 ─────────────────────────────────────

#[test]
fn e2e_dna_encode_decode_roundtrip() {
    let sequences: &[&[u8]] = &[
        b"ACGTACGT",
        b"NNNNNNNN",
        b"AAAAAAAAA",
        b"TTTTTTTTT",
        b"ACGTNNACGT",
    ];
    for &seq in sequences {
        let norm = dna::normalize_seq(seq);
        let encoded: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let decoded: Vec<u8> = encoded.iter().map(|&a| dna::from_alphabet(a)).collect();
        assert_eq!(decoded, norm, "encode/decode roundtrip failed for {:?}", std::str::from_utf8(seq));
    }
}

#[test]
fn e2e_dna_revcomp_preserves_length() {
    for len in [1, 4, 10, 50, 100] {
        let seq: Vec<u8> = (0..len).map(|i| b"ACGT"[i % 4]).collect();
        let rc = dna::revcomp(&seq);
        assert_eq!(rc.len(), seq.len());
        let back = dna::revcomp(&rc);
        assert_eq!(back, seq);
    }
}

// ─── 多 contig 比对 ──────────────────────────────────────────

#[test]
fn e2e_multi_contig_search() {
    let fasta = b">chr1\nAAAAAAAAAAAAAAAAAAAAA\n>chr2\nCCCCCCCCCCCCCCCCCCCC\n>chr3\nGGGGGGGGGGGGGGGGGGGG\n";
    let fm = build_fm_from_fasta_str(fasta);

    // 搜索只在 chr1 中的模式
    let pat_a: Vec<u8> = b"AAAAA".iter().map(|&b| dna::to_alphabet(b)).collect();
    let res_a = fm.backward_search(&pat_a).unwrap();
    let pos_a = fm.sa_interval_positions(res_a.0, res_a.1);
    for &p in &pos_a {
        let mapped = fm.map_text_pos(p);
        assert!(mapped.is_some());
        assert_eq!(mapped.unwrap().0, 0, "AAAAA should be in chr1 (contig 0)");
    }

    // 搜索只在 chr2 中的模式
    let pat_c: Vec<u8> = b"CCCCC".iter().map(|&b| dna::to_alphabet(b)).collect();
    let res_c = fm.backward_search(&pat_c).unwrap();
    let pos_c = fm.sa_interval_positions(res_c.0, res_c.1);
    for &p in &pos_c {
        let mapped = fm.map_text_pos(p);
        assert!(mapped.is_some());
        assert_eq!(mapped.unwrap().0, 1, "CCCCC should be in chr2 (contig 1)");
    }

    // 搜索只在 chr3 中的模式
    let pat_g: Vec<u8> = b"GGGGG".iter().map(|&b| dna::to_alphabet(b)).collect();
    let res_g = fm.backward_search(&pat_g).unwrap();
    let pos_g = fm.sa_interval_positions(res_g.0, res_g.1);
    for &p in &pos_g {
        let mapped = fm.map_text_pos(p);
        assert!(mapped.is_some());
        assert_eq!(mapped.unwrap().0, 2, "GGGGG should be in chr3 (contig 2)");
    }
}

// ─── SA 构建正确性端到端验证 ─────────────────────────────────

#[test]
fn e2e_sa_produces_sorted_suffixes() {
    let seq = b"ACGTACGTNNACGT";
    let norm = dna::normalize_seq(seq);
    let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    text.push(0);
    let sa_arr = sa::build_sa(&text);

    // 验证后缀数组确实按字典序排列
    for i in 1..sa_arr.len() {
        let s1 = &text[sa_arr[i - 1] as usize..];
        let s2 = &text[sa_arr[i] as usize..];
        assert!(s1 <= s2, "SA not sorted at position {}", i);
    }
}
