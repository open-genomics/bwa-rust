---
layout: home

hero:
  name: bwa-rust
  text: BWA in Rust
  tagline: High-Performance · Memory-Safe · Zero Unsafe | Modern bioinformatics tool inspired by BWA-MEM
  image:
    src: /logo.svg
    alt: bwa-rust logo
  actions:
    - theme: brand
      text: 🚀 Get Started
      link: /en/guide/getting-started
    - theme: alt
      text: 📖 Documentation
      link: /en/guide/architecture
    - theme: alt
      text: 💻 GitHub
      link: https://github.com/LessUp/bwa-rust

features:
  - icon: 🧬
    title: FM Index
    details: Suffix Array + BWT + Sparse SA Sampling, single .fm file with magic number
    link: /en/guide/index-building
  - icon: 🎯
    title: BWA-MEM Style
    details: SMEM seeds → Chain building → Banded Smith-Waterman → Semi-global refinement
    link: /en/guide/alignment
  - icon: 📄
    title: SAM Output
    details: Full @HD/@SQ/@PG headers, CIGAR, MAPQ, AS/XS/NM tags, SAM compliant
    link: /en/guide/getting-started
  - icon: ⚡
    title: Multi-threaded
    details: Rayon-based read-level parallelism with custom thread pool
    link: /en/guide/performance
  - icon: 🛡️
    title: Memory Protection
    details: max_occ / max_chains / max_alignments limits prevent memory explosion
    link: /en/guide/memory-protection
  - icon: 🦀
    title: Rust Safety
    details: Zero unsafe code with compile-time safety guarantees
---

<div class="stats-section">
  <div class="stats-grid">
    <div class="stat-card">
      <div class="stat-value">100<span class="stat-suffix">%</span></div>
      <div class="stat-label">Test Coverage</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">0<span class="stat-suffix"></span></div>
      <div class="stat-label">Unsafe Code</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">32<span class="stat-suffix">+</span></div>
      <div class="stat-label">Max Threads</div>
    </div>
  </div>
</div>

<div class="quick-features">
  <div class="quick-feat">
    <span class="quick-icon">⚡</span>
    <div class="quick-content">
      <div class="quick-title">O(n log²n)</div>
      <div class="quick-desc">SA Build</div>
    </div>
  </div>
  <div class="quick-feat">
    <span class="quick-icon">🎯</span>
    <div class="quick-content">
      <div class="quick-title">BWA-MEM</div>
      <div class="quick-desc">Style</div>
    </div>
  </div>
  <div class="quick-feat">
    <span class="quick-icon">🦀</span>
    <div class="quick-content">
      <div class="quick-title">Zero unsafe</div>
      <div class="quick-desc">Memory Safe</div>
    </div>
  </div>
</div>

<div class="architecture-flow">
  <h2 class="section-title">🔄 Alignment Pipeline</h2>
  <div class="flow-container">
    <div class="flow-step">
      <div class="flow-icon">🧬</div>
      <div class="flow-text">Read FASTQ</div>
    </div>
    <div class="flow-arrow">→</div>
    <div class="flow-step">
      <div class="flow-icon">🎯</div>
      <div class="flow-text">SMEM Seeds</div>
    </div>
    <div class="flow-arrow">→</div>
    <div class="flow-step">
      <div class="flow-icon">🔗</div>
      <div class="flow-text">Chain</div>
    </div>
    <div class="flow-arrow">→</div>
    <div class="flow-step">
      <div class="flow-icon">✂️</div>
      <div class="flow-text">SW Align</div>
    </div>
    <div class="flow-arrow">→</div>
    <div class="flow-step">
      <div class="flow-icon">📄</div>
      <div class="flow-text">SAM Output</div>
    </div>
  </div>
</div>

<div class="code-showcase">
  <h2 class="section-title">💻 One Command Start</h2>

::: code-group

```bash [Build index]
# Build FM index from FASTA reference
bwa-rust index reference.fa -o ref
```

```bash [Align reads]
# BWA-MEM style one-step alignment
bwa-rust mem ref.fa reads.fq -t 8 -o output.sam
```

:::

</div>

<div class="performance-section">
  <h2 class="section-title">📊 Comparison with BWA</h2>
  <div class="comparison-table">
    <div class="comp-row comp-header">
      <div class="comp-cell">Feature</div>
      <div class="comp-cell">BWA (C)</div>
      <div class="comp-cell highlight">bwa-rust</div>
    </div>
    <div class="comp-row">
      <div class="comp-cell">Index Format</div>
      <div class="comp-cell">Multiple files</div>
      <div class="comp-cell highlight">Single .fm file</div>
    </div>
    <div class="comp-row">
      <div class="comp-cell">Memory Safety</div>
      <div class="comp-cell">unsafe C code</div>
      <div class="comp-cell highlight">🦀 Zero unsafe</div>
    </div>
    <div class="comp-row">
      <div class="comp-cell">Parallel</div>
      <div class="comp-cell">pthread</div>
      <div class="comp-cell highlight">rayon (Rust)</div>
    </div>
    <div class="comp-row">
      <div class="comp-cell">Build Time</div>
      <div class="comp-cell">Fast (O(n))</div>
      <div class="comp-cell highlight">Good (O(n log²n))</div>
    </div>
  </div>
</div>

  <div class="trust-badges">
   <div class="badge-item">
     <span class="badge-icon">✅</span>
     <span>175 Tests Passing</span>
   </div>
   <div class="badge-item">
     <span class="badge-icon">🚀</span>
     <span>GitHub Actions CI</span>
   </div>
   <div class="badge-item">
     <span class="badge-icon">📦</span>
     <span>Cross-Platform</span>
   </div>
   <div class="badge-item">
     <span class="badge-icon">🌐</span>
     <span>Bilingual Docs</span>
   </div>
   <div class="badge-item">
     <span class="badge-icon">🔬</span>
     <span>v0.3.0 In Development</span>
   </div>
 </div>

<style>
.stats-section {
  margin: 3rem 0;
  padding: 2rem;
  border-radius: 16px;
  background: linear-gradient(135deg, var(--vp-c-bg-soft) 0%, var(--vp-c-bg) 100%);
  border: 1px solid var(--vp-c-border);
}
.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 2rem;
}
.stat-card {
  text-align: center;
  padding: 1.5rem;
  border-radius: 12px;
  background: var(--vp-c-bg);
  box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1);
  transition: transform 0.2s, box-shadow 0.2s;
}
.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 10px 25px -5px rgba(0,0,0,0.1);
}
.stat-value {
  font-size: 3rem;
  font-weight: 800;
  background: linear-gradient(135deg, var(--vp-c-brand-1), var(--vp-c-brand-2));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}
.stat-suffix { font-size: 1.5rem; font-weight: 600; }
.stat-label { margin-top: 0.5rem; font-size: 1rem; color: var(--vp-c-text-2); font-weight: 500; }

.quick-features {
  display: flex;
  justify-content: center;
  gap: 2rem;
  margin: 2rem 0;
  flex-wrap: wrap;
}
.quick-feat {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-radius: 12px;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-border);
}
.quick-icon { font-size: 1.5rem; }
.quick-title { font-weight: 700; font-size: 1.1rem; color: var(--vp-c-brand-1); }
.quick-desc { font-size: 0.875rem; color: var(--vp-c-text-2); }

.architecture-flow { margin: 4rem 0; }
.section-title {
  text-align: center;
  font-size: 1.75rem;
  font-weight: 700;
  margin-bottom: 2rem;
  color: var(--vp-c-text-1);
}
.flow-container {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-wrap: wrap;
  gap: 1rem;
  padding: 2rem;
  border-radius: 16px;
  background: var(--vp-c-bg-soft);
}
.flow-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1rem 1.5rem;
  border-radius: 12px;
  background: var(--vp-c-bg);
  border: 2px solid var(--vp-c-border);
  min-width: 100px;
  transition: all 0.3s;
}
.flow-step:hover {
  border-color: var(--vp-c-brand-1);
  transform: scale(1.05);
}
.flow-icon { font-size: 2rem; margin-bottom: 0.5rem; }
.flow-text { font-size: 0.9rem; font-weight: 600; color: var(--vp-c-text-1); }
.flow-arrow { font-size: 2rem; color: var(--vp-c-brand-1); font-weight: 700; }

.code-showcase { margin: 4rem 0; }

.performance-section { margin: 4rem 0; }
.comparison-table {
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--vp-c-border);
}
.comp-row {
  display: grid;
  grid-template-columns: 1.2fr 1fr 1fr;
  gap: 1rem;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--vp-c-border);
  background: var(--vp-c-bg);
}
.comp-row:last-child { border-bottom: none; }
.comp-header {
  background: var(--vp-c-bg-soft);
  font-weight: 700;
  color: var(--vp-c-text-1);
}
.comp-cell { font-size: 0.95rem; }
.comp-cell.highlight { color: var(--vp-c-brand-1); font-weight: 600; }

.trust-badges {
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  gap: 1.5rem;
  margin: 4rem 0;
  padding: 2rem;
  border-radius: 16px;
  background: var(--vp-c-bg-soft);
}
.badge-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.95rem;
  color: var(--vp-c-text-2);
}
.badge-icon { font-size: 1.25rem; }

@media (max-width: 768px) {
  .stats-grid { grid-template-columns: 1fr; }
  .quick-features { flex-direction: column; align-items: center; }
  .flow-container { flex-direction: column; }
  .flow-arrow { transform: rotate(90deg); }
  .comp-row { grid-template-columns: 1fr; text-align: center; }
  .comp-header { display: none; }
}
.dark .stat-card { box-shadow: 0 4px 6px -1px rgba(0,0,0,0.3); }
.dark .stat-card:hover { box-shadow: 0 10px 25px -5px rgba(0,0,0,0.4); }
</style>
