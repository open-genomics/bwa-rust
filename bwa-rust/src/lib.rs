//! # bwa-rust
//!
//! 受 [BWA](https://github.com/lh3/bwa) 启发的 Rust 版序列比对器。
//!
//! 本 crate 提供了基于 FM 索引的 DNA 序列比对功能，包括：
//!
//! - **索引构建**：从 FASTA 参考序列构建 FM 索引（后缀数组 + BWT）
//! - **种子查找**：SMEM（超级最大精确匹配）种子搜索
//! - **序列比对**：带状仿射间隙 Smith-Waterman 局部对齐
//! - **链构建**：种子链构建与过滤
//!
//! ## 快速示例
//!
//! ```rust,no_run
//! use bwa_rust::index::{sa, bwt, fm};
//! use bwa_rust::util::dna;
//!
//! // 构建 FM 索引
//! let reference = b"ACGTACGTAGCTGATCGTAG";
//! let norm = dna::normalize_seq(reference);
//! let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
//! let len = text.len() as u32;
//! let contigs = vec![fm::Contig { name: "ref".to_string(), len, offset: 0 }];
//! text.push(0);
//!
//! let sa_arr = sa::build_sa(&text);
//! let bwt_arr = bwt::build_bwt(&text, &sa_arr);
//! let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, dna::SIGMA as u8, 16);
//!
//! // 精确匹配搜索
//! let pattern: Vec<u8> = b"GCTGATC".iter().map(|&b| dna::to_alphabet(b)).collect();
//! if let Some((l, r)) = fm_idx.backward_search(&pattern) {
//!     let positions = fm_idx.sa_interval_positions(l, r);
//!     println!("Found {} occurrences", positions.len());
//! }
//! ```
//!
//! ## 模块说明
//!
//! - [`io`] — FASTA / FASTQ 文件解析
//! - [`index`] — FM 索引构建（后缀数组、BWT、FM 索引）
//! - [`align`] — 序列比对算法（SMEM 种子、链构建、Smith-Waterman）
//! - [`util`] — DNA 编码 / 解码 / 反向互补等工具函数

pub mod io;
pub mod index;
pub mod util;
pub mod align;
