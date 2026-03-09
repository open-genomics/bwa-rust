---
layout: home

hero:
  name: bwa-rust
  text: BWA Sequence Aligner in Rust
  tagline: A high-performance DNA sequence aligner inspired by BWA/BWA-MEM, implemented in Rust
  actions:
    - theme: brand
      text: Getting Started
      link: /en/guide/getting-started
    - theme: alt
      text: Architecture
      link: /en/guide/architecture
    - theme: alt
      text: GitHub
      link: https://github.com/LessUp/bwa-rust

features:
  - icon: 🧬
    title: FM Index Construction
    details: Suffix Array + BWT + Sparse SA Sampling, serialized into a single .fm file with magic number and version compatibility checks
  - icon: 🎯
    title: BWA-MEM Style Alignment
    details: SMEM seed finding → seed chaining & filtering → banded affine-gap Smith-Waterman
  - icon: 📄
    title: Standard SAM Output
    details: '@HD/@SQ/@PG headers, CIGAR, MAPQ, AS/XS/NM tags, primary/secondary alignment FLAGs'
  - icon: ⚡
    title: Multi-threaded
    details: Rayon-based read-level parallelism for efficient multi-core utilization
  - icon: 🦀
    title: Memory Safety by Rust
    details: Zero unsafe code, compile-time memory safety guarantees; jemalloc allocator for improved multi-threaded throughput
  - icon: 🧪
    title: 133 Tests Passing
    details: 121 unit + 11 integration + 1 doc test, criterion benchmarks, GitHub Actions CI
---
