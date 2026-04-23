//! 真实数据测试：与 BWA 输出对比验证
//!
//! 这些测试需要真实的参考基因组和测序数据。
//! 默认跳过，通过 `cargo test --features real-data` 启用。
//!
//! 测试数据获取：
//! ```bash
//! # 下载测试数据到 tests/data/
//! mkdir -p tests/data
//! # 示例：使用大肠杆菌基因组
//! wget -O tests/data/e_coli.fa.gz ftp://ftp.ncbi.nlm.nih.gov/genomes/all/GCF/000/005/845/GCF_000005845.2_ASM584v2/GCF_000005845.2_ASM584v2_genomic.fna.gz
//! gunzip tests/data/e_coli.fa.gz
//! # 生成测试 reads
//! # ...
//! ```

#[cfg(feature = "real-data")]
mod tests {
    use std::process::Command;

    /// 检查 BWA 是否可用
    fn bwa_available() -> bool {
        Command::new("bwa").arg("version").output().is_ok()
    }

    /// 检查测试数据是否存在
    fn test_data_exists() -> bool {
        std::path::Path::new("tests/data/e_coli.fa").exists()
    }

    #[test]
    #[ignore = "需要 --features real-data 和测试数据"]
    fn e2e_compare_with_bwa_e_coli() {
        if !bwa_available() || !test_data_exists() {
            eprintln!("跳过：BWA 或测试数据不可用");
            return;
        }

        // TODO: 实现 BWA 对比测试
        // 1. 用 bwa-rust 构建索引
        // 2. 用 bwa mem 比对
        // 3. 用 bwa-rust mem 比对
        // 4. 对比 SAM 输出（mapping rate, positions, CIGAR 等）
        unimplemented!("待实现")
    }

    #[test]
    #[ignore = "需要 --features real-data 和测试数据"]
    fn e2e_index_build_performance() {
        if !test_data_exists() {
            eprintln!("跳过：测试数据不可用");
            return;
        }

        use bwa_rust::index::builder::build_fm_from_fasta;
        use std::time::Instant;

        let start = Instant::now();
        let result = build_fm_from_fasta("tests/data/e_coli.fa", 512).expect("索引构建失败");
        let elapsed = start.elapsed();

        println!("索引构建时间: {:?}", elapsed);
        println!("序列数: {}", result.n_seqs);
        println!("总长度: {} bp", result.total_len);

        // 性能基准：大肠杆菌基因组 (~4.6Mbp) 应在 60 秒内完成
        #[cfg(debug_assertions)]
        assert!(elapsed.as_secs() < 120, "Debug 模式索引构建超时");
        #[cfg(not(debug_assertions))]
        assert!(elapsed.as_secs() < 60, "Release 模式索引构建超时");
    }
}
