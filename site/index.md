---
layout: home

hero:
  name: bwa-rust
  text: Rust 版 BWA 序列比对器
  tagline: 高性能 · 内存安全 · 零 unsafe 代码 | 受 BWA-MEM 启发的现代生物信息学工具
  image:
    src: /logo.svg
    alt: bwa-rust logo
  actions:
    - theme: brand
      text: 🚀 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 📖 查看文档
      link: /guide/architecture
    - theme: alt
      text: 💻 GitHub
      link: https://github.com/LessUp/bwa-rust

features:
  - icon: 🧬
    title: FM 索引构建
    details: 后缀数组 + BWT + 稀疏 SA 采样，单一 .fm 文件，magic number 与版本兼容检查
    link: /guide/index-building
  - icon: 🎯
    title: BWA-MEM 风格比对
    details: SMEM 种子查找 → 链构建 → 带状 Smith-Waterman → semi-global 精细化
    link: /guide/alignment
  - icon: 📄
    title: 标准 SAM 输出
    details: 完整 @HD/@SQ/@PG header、CIGAR、MAPQ、AS/XS/NM 标签
    link: /guide/getting-started
  - icon: ⚡
    title: 多线程并行
    details: 基于 rayon 的 reads 级并行，自定义线程池，多核充分利用
    link: /guide/performance
  - icon: 🛡️
    title: 内存安全防护
    details: max_occ / max_chains / max_alignments 三层防护，防止内存爆炸
    link: /guide/memory-protection
  - icon: 🦀
    title: Rust 内存安全
    details: 零 unsafe 代码，编译期安全保证；jemalloc 提升多线程吞吐
---

<!-- 动态统计区域 -->
<div class="stats-section">
  <div class="stats-grid">
    <div class="stat-card">
      <div class="stat-value">100<span class="stat-suffix">%</span></div>
      <div class="stat-label">测试通过率</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">0<span class="stat-suffix"></span></div>
      <div class="stat-label">unsafe 代码</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">32<span class="stat-suffix">+</span></div>
      <div class="stat-label">并发线程</div>
    </div>
  </div>
</div>

<!-- 快速特性展示 -->
<div class="quick-features">
  <div class="quick-feat">
    <span class="quick-icon">⚡</span>
    <div class="quick-content">
      <div class="quick-title">O(n log²n)</div>
      <div class="quick-desc">SA 构建复杂度</div>
    </div>
  </div>
  <div class="quick-feat">
    <span class="quick-icon">🎯</span>
    <div class="quick-content">
      <div class="quick-title">BWA-MEM</div>
      <div class="quick-desc">算法风格</div>
    </div>
  </div>
  <div class="quick-feat">
    <span class="quick-icon">🦀</span>
    <div class="quick-content">
      <div class="quick-title">Zero unsafe</div>
      <div class="quick-desc">内存安全</div>
    </div>
  </div>
</div>

<!-- 架构流程图 -->
<div class="architecture-flow">
  <h2 class="section-title">🔄 比对流程</h2>
  <div class="flow-container">
    <div class="flow-step">
      <div class="flow-icon">🧬</div>
      <div class="flow-text">读取 FASTQ</div>
    </div>
    <div class="flow-arrow">→</div>
    <div class="flow-step">
      <div class="flow-icon">🎯</div>
      <div class="flow-text">SMEM 种子</div>
    </div>
    <div class="flow-arrow">→</div>
    <div class="flow-step">
      <div class="flow-icon">🔗</div>
      <div class="flow-text">链构建</div>
    </div>
    <div class="flow-arrow">→</div>
    <div class="flow-step">
      <div class="flow-icon">✂️</div>
      <div class="flow-text">SW 对齐</div>
    </div>
    <div class="flow-arrow">→</div>
    <div class="flow-step">
      <div class="flow-icon">📄</div>
      <div class="flow-text">SAM 输出</div>
    </div>
  </div>
</div>

<!-- 代码示例区域 -->
<div class="code-showcase">
  <h2 class="section-title">💻 一行命令开始</h2>

::: code-group

```bash [构建索引]
# 从 FASTA 参考序列构建 FM 索引
bwa-rust index reference.fa -o ref
```

```bash [比对 reads]
# BWA-MEM 风格一步比对
bwa-rust mem ref.fa reads.fq -t 8 -o output.sam
```

:::

</div>

<!-- 性能对比 -->
<div class="performance-section">
  <h2 class="section-title">📊 与 BWA 对比</h2>
  <div class="comparison-table">
    <div class="comp-row comp-header">
      <div class="comp-cell">特性</div>
      <div class="comp-cell">BWA (C)</div>
      <div class="comp-cell highlight">bwa-rust</div>
    </div>
    <div class="comp-row">
      <div class="comp-cell">索引格式</div>
      <div class="comp-cell">多文件 (.bwt/.sa/.pac)</div>
      <div class="comp-cell highlight">单一 .fm 文件</div>
    </div>
    <div class="comp-row">
      <div class="comp-cell">内存安全</div>
      <div class="comp-cell">unsafe C 代码</div>
      <div class="comp-cell highlight">🦀 零 unsafe</div>
    </div>
    <div class="comp-row">
      <div class="comp-cell">并行框架</div>
      <div class="comp-cell">pthread</div>
      <div class="comp-cell highlight">rayon (Rust)</div>
    </div>
    <div class="comp-row">
      <div class="comp-cell">构建时间</div>
      <div class="comp-cell">快 (O(n))</div>
      <div class="comp-cell highlight">适中 (O(n log²n))</div>
    </div>
  </div>
</div>

<!-- 信任徽章 -->
  <div class="trust-badges">
   <div class="badge-item">
     <span class="badge-icon">✅</span>
     <span>175 项测试全通过</span>
   </div>
   <div class="badge-item">
     <span class="badge-icon">🚀</span>
     <span>GitHub Actions CI</span>
   </div>
   <div class="badge-item">
     <span class="badge-icon">📦</span>
     <span>跨平台支持</span>
   </div>
   <div class="badge-item">
     <span class="badge-icon">🌐</span>
     <span>中英双语文档</span>
   </div>
   <div class="badge-item">
     <span class="badge-icon">🔬</span>
     <span>v0.3.0 开发中</span>
   </div>
 </div>

<style>
/* ===== 动态统计区域 ===== */
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
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s, box-shadow 0.2s;
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1);
}

.stat-value {
  font-size: 3rem;
  font-weight: 800;
  background: linear-gradient(135deg, var(--vp-c-brand-1), var(--vp-c-brand-2));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.stat-suffix {
  font-size: 1.5rem;
  font-weight: 600;
}

.stat-label {
  margin-top: 0.5rem;
  font-size: 1rem;
  color: var(--vp-c-text-2);
  font-weight: 500;
}

/* ===== 快速特性 ===== */
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

.quick-icon {
  font-size: 1.5rem;
}

.quick-title {
  font-weight: 700;
  font-size: 1.1rem;
  color: var(--vp-c-brand-1);
}

.quick-desc {
  font-size: 0.875rem;
  color: var(--vp-c-text-2);
}

/* ===== 架构流程图 ===== */
.architecture-flow {
  margin: 4rem 0;
}

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

.flow-icon {
  font-size: 2rem;
  margin-bottom: 0.5rem;
}

.flow-text {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--vp-c-text-1);
}

.flow-arrow {
  font-size: 2rem;
  color: var(--vp-c-brand-1);
  font-weight: 700;
}

/* ===== 代码展示 ===== */
.code-showcase {
  margin: 4rem 0;
}

/* ===== 性能对比 ===== */
.performance-section {
  margin: 4rem 0;
}

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

.comp-row:last-child {
  border-bottom: none;
}

.comp-header {
  background: var(--vp-c-bg-soft);
  font-weight: 700;
  color: var(--vp-c-text-1);
}

.comp-cell {
  font-size: 0.95rem;
}

.comp-cell.highlight {
  color: var(--vp-c-brand-1);
  font-weight: 600;
}

/* ===== 信任徽章 ===== */
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

.badge-icon {
  font-size: 1.25rem;
}

/* ===== 响应式设计 ===== */
@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }
  
  .quick-features {
    flex-direction: column;
    align-items: center;
  }
  
  .flow-container {
    flex-direction: column;
  }
  
  .flow-arrow {
    transform: rotate(90deg);
  }
  
  .comp-row {
    grid-template-columns: 1fr;
    text-align: center;
  }
  
  .comp-header {
    display: none;
  }
}

.dark .stat-card {
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
}

.dark .stat-card:hover {
  box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.4);
}
</style>
