# Rust BWA-inspired Aligner Roadmap

目标：实现一个**受 BWA 启发的 Rust 版序列比对器**，在整体结构和算法思想上接近 BWA/BWA-MEM，但**不追求 100% 行为兼容**（命令行选项、索引格式、MAPQ 细节等允许不同）。

当前已有：
- `bwa-rust/` 内已有：
  - `index` 子命令：读取 FASTA，构建基于后缀数组+FM 的索引，并序列化为 `.fm` 文件。
  - `align` 子命令：加载 `.fm`，对 FASTQ 进行**精确匹配**，输出简化 SAM。
  - `io::fasta / io::fastq`、`util::dna`、`index::sa/bwt/fm` 等基础模块。
- `bwa-0.7.19/` 内为 C 版 BWA 源码，作为算法与结构的主要参考实现。

下面是按阶段划分的项目开发路线图与详细 TODO 列表。

---

## 阶段 0：项目基线与参考数据

**目标：** 固定项目目标与范围，准备用于回归测试和对比的基础数据与脚本。

- [ ] 明确项目目标与范围
  - [ ] 在 `bwa-rust/README.md` 中补充：
    - [ ] 明确说明：本项目是“受 BWA 启发的 Rust 实现”，**不追求与 C 版 BWA 完全兼容**。
    - [ ] 描述当前已实现的功能（index + 精确匹配 align）。

- [ ] 准备测试/参考数据集
  - [ ] 在仓库中创建 `data/` 目录（或其它你喜欢的路径），加入：
    - [ ] 小型参考序列 `toy.fa`（例如几个小 contig，总长度几万 bp）。
    - [ ] 对应的 reads FASTQ，如 `toy_reads.fq`（覆盖匹配、错配、indel 等情况）。
  - [ ] 使用 C 版 BWA 生成基准结果（可手工在 README 或单独文档中记录命令）：
    - [ ] `bwa index toy.fa`
    - [ ] `bwa mem toy.fa toy_reads.fq > toy_bwa_mem.sam`

- [ ] 构建基础开发脚本（可选，但有利于长期维护）
  - [ ] 在 `scripts/` 目录下添加：
    - [ ] `run_index.sh`：包装 `cargo run --bin bwa-rust -- index ...`。
    - [ ] `run_align.sh`：包装 `cargo run --bin bwa-rust -- align ...`。
    - [ ] `compare.sh`（可选）：
      - [ ] 调用原版 `bwa mem` 与 `bwa-rust align`，简单比较 mapped/unmapped 数量或用于手动 diff SAM。

---

## 阶段 1：索引模块稳定化（Index / FMIndex）

**目标：** 让 `index` 子命令和索引结构（FMIndex）在语义和实现上足够稳定，便于后续对齐模块在其基础上持续演进。

### 1.1 FASTA 读取与 DNA 编码的健壮性

- [ ] 为 `io::fasta::FastaReader` 增强测试用例
  - [ ] 多 contig FASTA（带/不带描述行），验证 `id`、`desc`、`seq` 是否正确。
  - [ ] 不同换行符（\n/\r\n）情况。
  - [ ] 中间穿插空行或非标准字符时的行为（是否被正确过滤或转换）。

- [ ] 为 `util::dna` 模块增加单元测试
  - [ ] `normalize_seq`：
    - [ ] A/C/G/T/U/N 的归一化逻辑正确；
    - [ ] 其它字符被映射为 N。
  - [ ] `to_alphabet` / `from_alphabet`：
    - [ ] 测试 round-trip：`from_alphabet(to_alphabet(base))` 结果合理；
    - [ ] 覆盖 `{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}` 的映射关系。
  - [ ] `revcomp`：
    - [ ] 测试 `revcomp(revcomp(seq)) == seq`（对只含 A/C/G/T/N 的序列）。

### 1.2 FMIndex 结构与序列化设计

- [ ] 明确定义 FM 索引的文件格式
  - [ ] 在 `index::fm::FMIndex` 中新增：
    - [ ] 索引版本或 magic 字段，用于未来格式升级时做兼容性检查。
    - [ ] 可选的元数据结构（如构建时参考文件名、命令行参数、构建时间戳等）。
  - [ ] 在 `README.md` 或 `docs/` 中简单记录 `.fm` 文件格式（高层说明即可）。

- [ ] 为 `FMIndex::build/save_to_file/load_from_file` 添加测试
  - [ ] 使用小型 toy 文本：
    - [ ] 构建 FM 索引；
    - [ ] 序列化到文件；
    - [ ] 再从文件加载，验证关键字段是否一致（`sigma/block/c/bwt/sa/contigs`）。

### 1.3 后缀数组（SA）与 BWT 的正确性

- [ ] 为 `index::sa::build_sa` 增强测试
  - [ ] 针对少量人工构造的文本验证输出顺序（已存在的基本测试可拓展）。
  - [ ] 在测试环境下实现一个朴素 O(n² log n) SA 构造，用它来验证当前倍增算法对随机文本的正确性。

- [ ] 为 `index::bwt::build_bwt` 增强测试
  - [ ] 验证单 contig 文本生成的 BWT 是否符合预期；
  - [ ] 验证包含多个 `$` 分隔符（多 contig）的文本也能正确构建 BWT。

### 1.4 CLI 与错误处理

- [ ] 改进 `run_index` 的健壮性
  - [ ] 当输入 FASTA 为空或非常短时，给出明确错误或警告。
  - [ ] 当输出路径不可写时，返回清晰的错误信息（anyhow 上报）。

- [ ] 文档
  - [ ] 在 `bwa-rust/README.md` 中增加一个“索引构建与索引格式简介”小节。

---

## 阶段 2：对齐 MVP 增强（从精确匹配到“带错配的局部对齐”）

**目标：** 将当前“整条 read 精确匹配”的 MVP，提升为“允许一定错配/小 indel 的局部对齐”，输出带 CIGAR 的合理 SAM 行，为后续 BWA-MEM 风格算法打基础。

### 2.1 抽象对齐配置结构

- [ ] 在 `align` 模块中定义 `AlignOpt`（或类似）结构
  - [ ] 字段示例：
    - [ ] `match_score`、`mismatch_penalty`；
    - [ ] `gap_open`、`gap_extend`；
    - [ ] `band_width`（带状 SW 的带宽）；
    - [ ] `max_mismatch` 或 `score_threshold` 等简化配置。

- [ ] 将 CLI 与 `AlignOpt` 关联
  - [ ] 在 `Commands::Align` 中增加可选参数：
    - [ ] `--match` / `--mismatch`；
    - [ ] `--gap-open` / `--gap-ext`；
    - [ ] `--band-width`；
  - [ ] 在 `run_align` 中解析这些参数并传入 `align_fastq` 或新的对齐入口函数。

### 2.2 从精确匹配扩展到“种子 + 简单局部 SW”

- [ ] FM 索引中增加位置获取能力
  - [ ] 在 `FMIndex` 中新增方法：
    - [ ] 从 `[l, r)` 区间返回**所有** SA 位置（而不仅仅是第一个），用于生成多个候选位置。

- [ ] 在 `align` 模块中实现“种子 + 局部 DP”流程（简化版）
  - [ ] 设计简单的种子策略：
    - [ ] 将 read 分成一定长度的固定种子（例如 20–32bp），或滑窗选取若干种子。
    - [ ] 对每个种子使用 FM index 执行 exact backward search，得到候选区间 `[l, r)`。
    - [ ] 展开到若干具体候选位置（`sa_interval_positions`）。
  - [ ] 对每个候选位置周围参考片段执行带状 SW：
    - [ ] 提取参考上 `[pos - L, pos + read_len + L]` 的窗口（注意边界）。
    - [ ] 在 `align` 模块内实现一个简化版 SW 或 NW（带状矩阵即可）。
    - [ ] 输出对齐得分、CIGAR 和 edit distance（NM）。
  - [ ] 在所有候选中选取得分最高的一条作为最终比对结果。

### 2.3 更新 `align_fastq` 输出逻辑

- [ ] 替换当前的“整条 forward/backward 精确匹配”逻辑
  - [ ] 对每条 read：
    - [ ] 同时尝试正向和反向互补方向（前处理同当前实现）。
    - [ ] 使用上述“种子 + 局部 SW”流程得到 1 个或多个候选对齐；
    - [ ] 选择得分最高者，生成包含：
      - [ ] FLAG（0 或 16）；
      - [ ] RNAME、POS（1-based）、MAPQ（暂时可用简单函数估计）；
      - [ ] CIGAR（含 M/I/D）；
      - [ ] NM Tag（可选，使用 edit distance）。
  - [ ] 若所有候选得分都低于阈值，则按 unmapped 输出（保留现有 FLAG 4 行）。

- [ ] 为增强后的 MVP 写测试
  - [ ] 人工构造小参考和读段，验证：
    - [ ] 有简单错配/插入/缺失时，是否能找到合理对齐；
    - [ ] CIGAR 与 POS 是否符合预期。

---

## 阶段 3：BWA-MEM 风格的单端对齐

**目标：** 借鉴 BWA-MEM 的整体流程：MEM/SMEM 种子 → 种子链 → 局部扩展 → 主/次比对，形成更智能的单端对齐算法。

### 3.1 MEM / SMEM 种子查找

- [ ] 设计 Rust 版种子/对齐区域结构
  - [ ] 类似 `bwamem.h` 中的 `mem_alnreg_t` 等：
    - [ ] 包含 read 区间 `[qb, qe)`、参考区间 `[rb, re)`、得分等信息。

- [ ] 在 FM index 基础上实现 SMEM/MEM 搜索
  - [ ] 在新的模块（可为 `align::seed` 或 `index::mem`）中实现：
    - [ ] 对 read 的每个位置，寻找覆盖该位置的最长匹配（MEM/SMEM）。
    - [ ] 借鉴 BWA 中 `bwt_smem1/bwt_smem1a` 的思想，但不要求一字不差。
  - [ ] 形成一组 MEM 种子集合（包含参考位置和 read 上的位置）。

### 3.2 种子链构建与过滤

- [ ] 实现类似 `mem_chain` 的链构建
  - [ ] 将种子看成二维平面上的点（read 坐标 vs ref 坐标）。
  - [ ] 按某种贪心或 DP 策略选取覆盖度高、间距合理的一条或多条链。

- [ ] 过滤和去重
  - [ ] 类似 `mem_chain_flt`，根据：
    - [ ] 链的总种子长度/覆盖度；
    - [ ] 与其他链的重叠情况；
    过滤掉弱链和冗余链。

### 3.3 从链到完整对齐

- [ ] 沿链做 DP 扩展与 CIGAR 合并
  - [ ] 在链的各段之间执行局部 SW（可复用阶段 2 的 DP 实现）。
  - [ ] 合并得到完整 CIGAR、对齐得分、edit distance 等。

- [ ] 去重与排序
  - [ ] 类似 BWA 的 `mem_sort_dedup_patch`：
    - [ ] 对所有候选按得分、覆盖、位置排序；
    - [ ] 去除几乎完全重复的结果；
    - [ ] 标记主/次/补充比对。

### 3.4 输出与简单 MAPQ 策略

- [ ] 设计一个简化 MAPQ 模型
  - [ ] 基于主次候选得分差、覆盖度等信息估算 MAPQ。
  - [ ] 不必完全模仿 BWA 的公式，但行为要合理：
    - [ ] 顶级结果明显好于次级结果时给较高 MAPQ；
    - [ ] 有多个相近候选时降低 MAPQ。

- [ ] `align_fastq` 输出多条 SAM 行时：
  - [ ] 正确设置 FLAG：
    - [ ] 主对齐：正常 FLAG；
    - [ ] supplementary / secondary 对齐：设置对应比特。
  - [ ] 可选输出 `AS`（对齐分数）、`XS`（次优分数）、`NM`（edit distance）等标签。

---

## 阶段 4：性能优化与工程化

**目标：** 在功能基本稳定的基础上优化性能、提升并行能力，并完善工程实践（测试、CI 等）。

### 4.1 基准测试与 Profiling

- [ ] 使用 toy 数据和更大一些的测试集，对比：
  - [ ] 原版 BWA 与 `bwa-rust` 的运行时间与内存消耗（定性即可）。
- [ ] 使用 `criterion` 或 `cargo bench` 编写基准：
  - [ ] `FMIndex::backward_search`；
  - [ ] 种子查找（MEM/SMEM）函数；
  - [ ] DP 扩展函数；
  - [ ] 整体 `align_fastq` 流程。

### 4.2 多线程支持

- [ ] 为 `Align` 子命令增加 `--threads` 参数
  - [ ] 默认 1 线程，用户可设置为 N。

- [ ] 使用 Rayon 或自定义线程池实现 reads 级并行
  - [ ] 确保 FM index 为只读，在多线程下安全共享；
  - [ ] 控制内存占用（避免为每个线程复制大块结构）。

- [ ] SAM 输出顺序与同步
  - [ ] 保证在多线程情况下，输出顺序可控：
    - [ ] 默认保持输入 order；或
    - [ ] 提供 `--keep-order` 开关。

### 4.3 内存与数据结构优化

- [ ] 优化 FM index 的存储形式
  - [ ] 考虑稀疏 SA 采样，减少内存使用；
  - [ ] 更高效的 Occ 采样/压缩结构。

- [ ] 优化 DP 实现
  - [ ] 使用带状矩阵，减少空间复杂度；
  - [ ] 复用工作缓冲区，避免频繁分配。

---

## 阶段 5：文档、示例与长期维护

**目标：** 让项目更易于理解和使用，支持长期维护和迭代。

### 5.1 文档

- [ ] 在 `docs/architecture.md` 中描述整体架构
  - [ ] 模块划分：IO / Index / Align / Util；
  - [ ] 索引格式、对齐算法流程图；
  - [ ] 与 BWA/BWA-MEM 的主要差异与借鉴点。

- [ ] 完善 `bwa-rust/README.md`
  - [ ] 给出从 `index` 到 `align` 的完整示例；
  - [ ] 说明支持/不支持的功能（例如：目前仅支持单端 vs 未来支持 PE）。

### 5.2 示例与教程

- [ ] 添加 `examples/` 目录
  - [ ] `simple_align.rs`：演示如何在 library 模式下：
    - [ ] 加载 FM index；
    - [ ] 调用对齐 API 返回结构化结果；
    - [ ] 不通过 CLI 而是直接在代码中使用。

- [ ] 写一篇简短教程（可放在 `docs/`）
  - [ ] 主题例如：“从 0 实现一个 BWA 风格的 Rust FM 索引和对齐器”。

### 5.3 长期维护

- [ ] 配置 CI（例如 GitHub Actions）
  - [ ] 在每次提交/PR 时运行：
    - [ ] `cargo fmt -- --check`；
    - [ ] `cargo clippy -- -D warnings`；
    - [ ] `cargo test`。

- [ ] 版本管理与发布（如有需要）
  - [ ] 在 `Cargo.toml` 中按语义化版本号管理版本；
  - [ ] 考虑将核心库部分拆分并发布到 crates.io（如果你希望他人复用）。

---

## 总结

本路线图以“**受 BWA 启发的 Rust 版序列比对器**”为目标，分多个阶段逐步推进：

1. 先夯实现有索引和精确匹配 MVP（阶段 0–1）。
2. 再用“种子 + 局部 DP”增强对齐质量（阶段 2）。
3. 随后引入 BWA-MEM 风格的 MEM/链/扩展机制（阶段 3）。
4. 在功能基本完整后再集中优化性能和工程实践（阶段 4–5）。

后续可以按照该文档中的 TODO 勾选推进，也可以在此基础上继续细化每个阶段的子任务。
