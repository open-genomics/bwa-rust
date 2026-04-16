---
layout: home

hero:
  name: bwa-rust
  text: BWA Sequence Aligner in Rust
  tagline: A high-performance DNA sequence aligner inspired by BWA/BWA-MEM, implemented in Rust
  image:
    src: /logo.svg
    alt: bwa-rust logo
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
    link: /en/guide/architecture#index-construction
  - icon: 🎯
    title: BWA-MEM Style Alignment
    details: SMEM seed finding → seed chaining & filtering → banded affine-gap Smith-Waterman → semi-global refinement
    link: /en/guide/architecture#alignment-algorithm
  - icon: 📄
    title: Standard SAM Output
    details: '@HD/@SQ/@PG headers, CIGAR, MAPQ, AS/XS/NM tags, primary/secondary alignment FLAGs, fully SAM compliant'
    link: /en/guide/getting-started#cli-reference
  - icon: ⚡
    title: Multi-threaded
    details: Rayon-based read-level parallelism with custom thread pool, efficient multi-core utilization
    link: /en/guide/architecture#performance-optimization
  - icon: 🛡️
    title: Memory Protection
    details: max_occ filters repetitive seeds, max_chains limits candidates, max_alignments controls output to prevent memory explosion
    link: /en/guide/architecture#memory-protection
  - icon: 🦀
    title: Memory Safety by Rust
    details: Zero unsafe code, compile-time memory safety guarantees; jemalloc allocator for improved multi-threaded throughput
    link: /en/guide/architecture#safety-guarantees
  - icon: 🧪
    title: 168 Tests Passing
    details: 151 unit + 11 integration + 5 module + 1 doc test, criterion benchmarks, GitHub Actions CI
    link: /en/roadmap#test-coverage
  - icon: 📦
    title: Cross-platform Support
    details: Supports Linux (static/dynamic), macOS (Intel/ARM), Windows, with pre-built binaries available
    link: https://github.com/LessUp/bwa-rust/releases
---

<script setup>
const features = [
  { title: 'Fast', value: 'O(n log²n)' },
  { title: 'Accurate', value: 'BWA-MEM style' },
  { title: 'Safe', value: 'Zero unsafe' },
]
</script>

<div class="stats-container">
  <div v-for="feat in features" class="stat-item">
    <div class="stat-value">{{ feat.value }}</div>
    <div class="stat-label">{{ feat.title }}</div>
  </div>
</div>

<style>
.stats-container {
  display: flex;
  justify-content: center;
  gap: 3rem;
  margin: 2rem 0;
  padding: 1.5rem;
  border-radius: 12px;
  background: var(--vp-c-bg-soft);
}

.stat-item {
  text-align: center;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--vp-c-brand-1);
}

.stat-label {
  font-size: 0.9rem;
  color: var(--vp-c-text-2);
  margin-top: 0.25rem;
}

@media (max-width: 640px) {
  .stats-container {
    flex-direction: column;
    gap: 1rem;
  }
}
</style>
