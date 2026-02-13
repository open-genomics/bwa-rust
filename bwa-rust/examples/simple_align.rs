//! 演示如何在 library 模式下使用 bwa-rust 进行序列比对。
//!
//! 运行方式：
//! ```bash
//! cargo run --example simple_align
//! ```

use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::align::{self, SwParams};
use bwa_rust::util::dna;

fn main() {
    // 1. 构建参考序列
    let reference = b"ACGTACGTAGCTGATCGTAGCTAGCTAGCTGATCGTAGCTAGCTAGCTGAT";
    println!("参考序列: {}", std::str::from_utf8(reference).unwrap());
    println!("参考长度: {} bp", reference.len());

    // 2. 构建 FM 索引
    let norm = dna::normalize_seq(reference);
    let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    let len = text.len() as u32;
    let contigs = vec![fm::Contig {
        name: "ref1".to_string(),
        len,
        offset: 0,
    }];
    text.push(0); // sentinel

    let sa_arr = sa::build_sa(&text);
    let bwt_arr = bwt::build_bwt(&text, &sa_arr);
    let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, dna::SIGMA as u8, 16);

    println!("FM 索引构建完成：BWT 长度={}, SA 长度={}", fm_idx.bwt.len(), fm_idx.sa.len());

    // 3. 精确匹配搜索
    let pattern = b"GCTGATCGTAG";
    let pattern_alpha: Vec<u8> = dna::normalize_seq(pattern)
        .iter()
        .map(|&b| dna::to_alphabet(b))
        .collect();

    if let Some((l, r)) = fm_idx.backward_search(&pattern_alpha) {
        let positions = fm_idx.sa_interval_positions(l, r);
        println!("\n精确匹配 '{}': 找到 {} 处", std::str::from_utf8(pattern).unwrap(), positions.len());
        for pos in &positions {
            if let Some((ci, off)) = fm_idx.map_text_pos(*pos) {
                println!("  contig={}, offset={}", fm_idx.contigs[ci].name, off);
            }
        }
    }

    // 4. SMEM 种子查找
    let read = b"ACGTACGTAGCTGATCGTAG";
    let read_norm = dna::normalize_seq(read);
    let read_alpha: Vec<u8> = read_norm.iter().map(|&b| dna::to_alphabet(b)).collect();

    let seeds = align::find_smem_seeds(&fm_idx, &read_alpha, 5);
    println!("\nSMEM 种子（read='{}'）:", std::str::from_utf8(read).unwrap());
    for s in &seeds {
        println!("  read[{}..{}] -> ref[{}..{}] (contig={})",
            s.qb, s.qe, s.rb, s.re, s.contig);
    }

    // 5. 带状 Smith-Waterman 对齐
    let query = b"ACGTACGXAGCTGATCGTAG"; // 带一个错配
    let ref_seq = &reference[..20];
    let sw_params = SwParams {
        match_score: 2,
        mismatch_penalty: 1,
        gap_open: 2,
        gap_extend: 1,
        band_width: 8,
    };

    let result = align::banded_sw(query, ref_seq, sw_params);
    println!("\nSmith-Waterman 对齐:");
    println!("  Query:  {}", std::str::from_utf8(query).unwrap());
    println!("  Ref:    {}", std::str::from_utf8(ref_seq).unwrap());
    println!("  Score:  {}", result.score);
    println!("  CIGAR:  {}", result.cigar);
    println!("  NM:     {}", result.nm);
    println!("  Query区间: [{}, {})", result.query_start, result.query_end);
    println!("  Ref区间:   [{}, {})", result.ref_start, result.ref_end);

    // 6. 种子链构建
    if !seeds.is_empty() {
        let chains = align::build_chains(&seeds, 50);
        println!("\n构建了 {} 条种子链:", chains.len());
        for (i, ch) in chains.iter().enumerate() {
            println!("  链{}: contig={}, 种子数={}, 得分={}",
                i, ch.contig, ch.seeds.len(), ch.score);
        }
    }

    println!("\n完成！");
}
