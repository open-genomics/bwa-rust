# Rust 版 BWA 全量实现计划

## 目标
1. **CLI 兼容**：复刻 `bwa` 的主要子命令（`index`, `aln/samse/sampe`, `mem`, `bwasw`, `fastmap`, `bwamem`, `bwa` 主程序控制流）。
2. **核心算法**：按阶段迁移 BWT/FMD 索引、BWA-backtrack、BWA-SW、BWA-MEM（含配对、分段链、局部比对、KSW 扩展等）。
3. **工程能力**：提供可序列化索引、流式 FASTA/Q 读取、并行流水、SIMD/多线程优化、完整 SAM/BAM 输出。
4. **一致性验证**：使用原版测试集与 benchmark 确保结果一致或可解释偏差。

## 当前基线
- `bwa-rust` 已具备最小化 `index` 与精确匹配 `align` 功能（单读、无错）。
- 缺失：FMD 索引、压缩 Occ、部分 SA、k-步排序；所有高级子命令和算法。

## 阶段性路线

### 阶段 0：基准梳理（进行中）

| 模块 | 主要源文件 | 职责 | 关键依赖 |
| --- | --- | --- | --- |
| **顶层 CLI** | `main.c`, `bwa.c`, `bwa.h` | 解析子命令（`index/aln/samse/sampe/mem/bwasw/fastmap`），驱动批处理，设置 RG/PG | `bntseq`, `bwt`, `bwamem`, `bwase`, `bamlite`, `utils` |
| **序列元信息** | `bntseq.c/h` | 读取 FASTA，构建 `.pac/.ann/.amb`，提供 `bns`（contig 注释、AMB 区） | `kseq`, `kstring`, `utils` |
| **索引构建** | `bwt.c`, `bwt_gen.c`, `bwtindex.c`, `bwtsw2_*` | SA/BWT/FMD 生成、压缩 Occ、partial SA；支持 `rb2/bwtsw/is` 三种算法 | `bwt_lite`, `is.c`, `ksort`, `kthread` |
| **IO 与工具** | `bwaseqio.c`, `kseq.h`, `utils.c`, `kstring.c`, `kthread.c` | FASTA/Q 流式读取、压缩读写、线程池、日志、随机数 | `zlib`, `pthread` |
| **BWA-backtrack** | `bwase.c/h`, `bwtaln.c`, `bwtgap.c`, `bwape.c` | `aln/samse/sampe` 主逻辑，短读 seed 扩展、配对策略、BWT 搜索 | `bwt`, `bntseq`, `ksw`, `kbtree`, `khash` |
| **BWA-SW** | `bwasw2_*.c`, `bwtsw2_main.c` | 长读局部比对、链合并、banded DP | `ksw`, `bwt`, `bntseq` |
| **BWA-MEM** | `bwamem*.c`, `fastmap.c` | MEM seeding、链化、配对、局部 SW、SAM 输出 | `ksw`, `bwt`, `bntseq`, `kvec` |
| **配套工具** | `bamlite.*`, `rle.*`, `rope.*`, `pemerge.c`, `xa2multi.pl` | BAM 精简实现、波形压缩、rope 数据结构、pair merge、XA 标签工具 | `zlib`, `kstring`, `kvec` |

子命令与依赖关系概览：

1. `bwa index`: 依赖 `bntseq` 读入参考 → `bwtindex/bwt_gen` 生成 BWT/FMD → 输出 `.bwt/.sa/.ann/.amb/.pac`。
2. `bwa aln`: 使用 `bwtaln` 执行 BWT 精确/近似匹配 → `bwase` 评分与剪枝；输出 `.sai`。
3. `bwa samse/sampe`: 读取 `.sai` 与 FASTQ，借助 `bwase/bwape` 生成 SAM，处理单端/双端逻辑、插入片段估计。
4. `bwa mem`: `fastmap` 进行 MEM seeding → `bwamem` 链化、DP 扩展、配对、SAM 生成；依赖 `ksw` 进行局部比对。
5. `bwa bwasw`: 复用 `bwtsw2_*` 模块进行长序列 Smith-Waterman。
6. 共享工具：`bseq_read`（批量 FASTQ）、`bwa_fill_scmat`（打分矩阵）、`bwa_gen_cigar2`（生成 CIGAR）、`bwt` 与 `bntseq` 基础设施。

Rust 版模块映射建议：
- `cli`：包装 clap→兼容原 `bwa` 参数，桥接到 Rust 模块。
- `seqio`：封装 FASTA/Q（含批处理、paired chunk）。
- `index`：细分 `pac`（2bit 序列）、`sa`（SA-IS/倍增）、`bwt`、`fmd`、`ann/amb` 序列化。
- `align::backtrack`：对应 `aln/samse/sampe`，含 BWT 搜索、错配容忍、配对策略。
- `align::mem`：MEM seeding、链化、KSW 扩展、SAM 生成。
- `align::sw`：覆盖 `bwasw` 的 SW/banded DP。
- `util`：`ksw`（SIMD/DP）、`thread`（任务池）、`sam`（写入）、`config`（参数预设）。

阶段 0 下一动作：
1. 画出索引文件格式（`.bwt/.sa/.pac/.ann/.amb`）字段布局，准备 Rust 读写结构。
2. 梳理 `bseq_read` 批处理协议，决定 Rust 版本的 chunk & pipeline API。
3. 列出 `bwa mem` 参数→Rust 配置映射表，便于阶段 3 实现时直接落地。

#### 0.1 索引文件格式与 Rust 读写结构

| 文件 | 关键字段 | BWA 源实现要点 | Rust 侧结构 / 计划 |
| --- | --- | --- | --- |
| `.pac` | 2-bit 压缩序列；顺向 + 反向互补；尾部长度字节 | `bns_fasta2bntseq` 写入时追加反向互补、在尾部写入 `len % 4` 与必要的 padding @bwa-0.7.19/bntseq.c#232-333 | `index::pac::PacWriter` 负责顺序写入；`PacRecord { total_len, forward_len, buf }` 暴露 `into_mmap()` 并在 `Drop` 时写回尾字节；读取端暴露 `PacView<'a>`，提供 `get_base(idx, rev)`。 |
| `.ann` | 头 `(l_pac, n_seqs, seed)`；逐条 `(gi, name, anno, offset, len, n_ambs)` | `bns_dump/bns_restore` 以文本存储 contig 元信息，并可由 `.alt` 标记 ALT contig @bwa-0.7.19/bntseq.c#65-137,@bwa-0.7.19/bntseq.c#168-211 | `AnnRecord` mirror 结构，借助 `serde` 读写；提供 `AnnTable` 支持按 contig 检索与 ALT 标注。 |
| `.amb` | 头 `(l_pac, n_seqs, n_holes)`；记录 `(offset, len, amb_char)` | `amb` 文件记录连续 N 区域，offset/len 均落在 forward pac @bwa-0.7.19/bntseq.c#84-155 | `AmbRecord` 使用紧凑二进制写法（offset/len little-endian，char 为 UTF-8），读取后转为 `AmbSpan` 迭代器，供 BWT 构建回填随机碱基使用。 |
| `.bwt` | `primary`、`L2[4]`、压缩 BWT 内容 + OCC 表（每 0x80 bases 存 4×`bwtint_t`） | `bwt_dump_bwt` 顺序写入 meta 与 payload；`bwt_restore_bwt` 读取并生成计数表；`bwt_bwtupdate_core` 负责在磁盘上注入 OCC @bwa-0.7.19/bwt.c#385-470,@bwa-0.7.19/bwtindex.c#148-207 | `BwtFile { primary, l2: [u64;5], occ_interval, data }`；写入阶段拆为 `BwtRawBuilder`（无 OCC）与 `BwtOccBuilder`；读取使用 `memmap2` + `bytemuck` 提供只读视图。 |
| `.sa` | `primary`、`L2`（跳过）、`sa_intv`、`seq_len`、采样数组 | `bwt_dump_sa` 写入 `primary/L2/sa_intv/seq_len/SAs[1..]`；恢复时校验 primary/seq_len @bwa-0.7.19/bwt.c#396-441 | `SaFile { primary, seq_len, intv, samples: Vec<u64> }`；实现 `SaIndex::lookup(k)`，根据 `intv` + `bwt_B0` 回溯真实 SA。 |

> 实现步骤：
> 1. 新增 `index/formats/{pac,ann,amb,bwt,sa}.rs` 分别封装读写与校验。
> 2. 引入 `IndexBundle` 聚合器（持有 `PacView`, `AnnTable`, `AmbTable`, `Bwt`, `SaIndex`），统一暴露 `load(prefix)`/`build_from_fasta()`。
> 3. 定义 `IndexSerializer` trait，支持 “兼容 BWA 前缀文件” 与 “Rust 原生 *.fm 快照” 双后端。

#### 0.2 `bseq_read` 批处理协议 → Rust pipeline

**原协议**：`bseq_read(chunk_size,n_,ks1,ks2)` 基于累计碱基数截断批次，并保证 paired-end 模式下读数成对（读取第二 FASTQ 失败时告警）；`kseq2bseq1` 拷贝 name/comment/seq/qual，`id` 记录插入顺序；上游 `mem_process_seqs`/`bsw2_aln`/`pemerge` 均按 “读取→处理→释放” 三步流水执行 @bwa-0.7.19/bwa.c#70-113,@bwa-0.7.19/fastmap.c#64-121,@bwa-0.7.19/bwtsw2_aux.c#727-775,@bwa-0.7.19/pemerge.c#217-290。

**Rust 设计**：
1. `seqio::reader::FastxStream`：抽象 gzip/plain FASTA/Q 流，提供 `next_record()`，可包装 needletail 或手写 `kseq` FFI。
2. `seqio::chunker::ChunkConfig { chunk_bases, copy_comment, paired }` 负责累计碱基数与 `id` 分配；实现 `Chunker::next_batch(&mut self) -> Option<ReadBatch>`。
3. `ReadBatch` 结构包含 `reads: Vec<Read>` 与 `layout: BatchLayout`（`SingleEnd`/`InterleavedPE`），PE 模式保证 `reads.len()` 为偶数。
4. Pipeline：
   - `producer` 线程串行读取，借助 `crossbeam_channel` 将 `ReadBatch` 推送给 worker。
   - `worker`（Rayon 线程池）调用 `align::mem::process_batch(&MemConfig, &IndexBundle, ReadBatch)`；当 `copy_comment = false` 时在 reader 阶段即释放 comment，以免在 worker 再 free。
   - `consumer` 收集对齐结果（SAM/BAM 字符串），顺序写出并负责释放 `ReadBatch` 内存，mirror C 版 `step 2` 清理逻辑。
5. 附加保障：
   - `InterleaveGuard` 检查双端输入长度差异并打印 warn（对应原 `fprintf` 告警）。
   - `BatchStats` 记录 `n_reads/n_pairs/total_bases`，供日志与自适应 chunk 调整，默认使用 `mem_opt_t::chunk_size` (10M bp)。

#### 0.3 `bwa mem` 参数 → Rust 配置映射

`mem_opt_t` 定义了打分、链化、配对、输出、标志位等完整参数；`mem_opt_init()` 给出默认值 @bwa-0.7.19/bwamem.h#40-208,@bwa-0.7.19/bwamem.c#74-110。需要在 Rust 中提供一一映射的 `MemConfig` 并保持 CLI 兼容。

| CLI 选项 | `mem_opt_t` 字段/标志 | 说明 | Rust 规划 |
| --- | --- | --- | --- |
| `-k INT` | `min_seed_len` | MEM 最小长度 | `MemConfig::seeding.min_len`，默认 19。 |
| `-w INT` | `w` | DP 带宽 | `alignment.band_width`。 |
| `-d INT` | `zdrop` | Z-drop | `alignment.z_drop`。 |
| `-r FLOAT` | `split_factor` | MEM 拆分比例 | `seeding.split_factor`。 |
| `-c INT` | `split_width` | occ 阈值 | `seeding.split_width`。 |
| `-A/-B/-O/-E` | `a,b,o_del,e_del,o_ins,e_ins` | 打分矩阵 | `Scoring` 结构 + `fill_scmat()`。 |
| `-L INT[,INT]` | `pen_clip5/pen_clip3` | 端剪惩罚 | `alignment.clip.{five,three}`。 |
| `-U INT` | `pen_unpaired` | 单端惩罚 | `pairing.pen_unpaired`。 |
| `-x STR` | 预设 | 对多字段批量覆写 | `MemPreset` 枚举。 |
| `-T INT` | `T` | 输出阈值 | `output.min_score`. |
| `-P` | `flag|=MEM_F_PRIMARY5` | 优先 5' | `flags.primary5`. |
| `-p` | `flag|=MEM_F_PE` | 开启 PE | `flags.paired`. |
| `-a` | `flag|=MEM_F_ALL` | 输出所有 | `flags.emit_all`. |
| `-M` | `flag|=MEM_F_NO_RESCUE` | 禁用 mate rescue | `flags.no_rescue`. |
| `-S` | `flag|=MEM_F_NO_MULTI` | 禁用次要比对 | `flags.no_secondary`. |
| `-C` | `flag|=MEM_F_REF_HDR` | 输出 @SQ | `output.emit_sq`. |
| `-1` | `flag|=MEM_F_SMARTPE` | 智能区分 SE/PE | `flags.smart_pe`. |
| `-W INT` | `max_chain_gap` | 链距阈值 | `chaining.max_gap`. |
| `-m INT` | `max_occ` | 种子最大 Occ | `seeding.max_occ`. |
| `-S INT` | `max_matesw` | mate SW 次数 | `pairing.max_mate_sw`. |
| `-g INT` | `max_ins` | 估计插入上限 | `pairing.max_ins_for_stat`. |
| `-N INT` | `max_XA_hits` | XA 数量 | `output.max_xa`. |

**配置结构**：

```rust
pub struct MemConfig {
    pub scoring: Scoring,
    pub seeding: Seeding,
    pub chaining: Chaining,
    pub alignment: Alignment,
    pub pairing: Pairing,
    pub output: Output,
    pub flags: MemFlags,
    pub presets: Option<MemPreset>,
}
```

落地步骤：
1. 在 `align/mem/config.rs` 定义 `MemConfig` 及 `impl Default`（调用 `MemConfig::from_legacy(mem_opt_init())`）。
2. CLI (`cli::mem`) 通过 `clap` 声明全部参数 → `MemCliArgs::into_config()`。
3. `MemConfig::to_legacy()` 将值映射回 `mem_opt_t`，便于后续直接调用移植后的 MEM 内核。

### 阶段 1：索引管线
1. **FASTA 处理**
   - [ ] 移植 `bntseq` 功能：序列信息、contig 注释、AMB 区域记录。
   - [ ] 二进制 `.pac` 表示与 `$` 分隔策略。
2. **BWT/SA/FMD**
   - [ ] 引入 SA-IS（或借助现有 crate）以加速 SA 构建。
   - [ ] 构建 BWT（支持 2-bit 压缩），实现部分 SA 采样。
   - [ ] 构建 FMD（双向索引）供 `aln/mem` 共用。
3. **序列化**
   - [ ] 兼容 `.bwt/.sa/.ann/.amb/.pac` 前缀文件。
   - [ ] 支持 Rust 自身 `*.fm` 快照，互相转换。

#### 1.1 FASTA→PAC/ANN/AMB 构建流水

- **输入侧**：
  - 复用 `seqio::reader::FastxStream` 逐条拉取 FASTA；支持 `stdin`/gzip。
  - 提供 `IndexBuildConfig { for_only: bool, alt_list: Option<Path>, seed: u32 }` 对应 `bns_fasta2bntseq(fp, prefix, for_only)` 的第三参与 `.alt` 行为。
- **`bntseq` 映射**：
  - 定义 `index::meta::{BntAnn, BntAmb, BntSeq}` mirror C 端 `bntann1_t/bntamb1_t/bntseq_t` 结构。
  - 在 Rust 中通过 `Vec<BntAnn>`/`Vec<BntAmb>` 累积 contig 与 N 区域，并在结束时调用 `write_ann(prefix)`、`write_amb(prefix)` 输出。
- **`pac` 写入**：
  - 使用前文的 `PacWriter` 将 2-bit 压缩序列落盘；当 `for_only=false` 时追加反向互补段，长度为 `l_pac*2`，保持与 C 版一致。
  - 统一由 `PacWriter::finish()` 负责写尾部 `ct` 字节与 padding，避免多处散落逻辑。

#### 1.2 BWT/SA/FMD 构建策略

- **SA 构建**：
  - 抽象 `trait SaBuilder { fn build(pac: &PacView) -> SaArray; }`，默认实现使用纯 Rust SA-IS 或现有 MIT/Apache crate；预留 `algo_type` 参数对应 C 版 `bwt_bwtgen2/bwt_pac2bwt`。
  - `SaArray` 仅在内存中存在，用于后续 BWT 构建，不直接序列化。
- **BWT 构建**：
  - 在 `index::bwt::build_bwt_from_sa` 中实现：从 SA 顺序生成原始 BWT 字符串，再交给 `BwtRawBuilder` 压缩为 2-bit + block 布局。
  - 引入 `BwtOccBuilder::with_interval(OCC_INTERVAL)` 注入 OCC 表，之后持久化到 `.bwt` 文件中。
- **FMD 构建**：
  - 依赖 pac 中 forward+RC 拼接特点，通过在同一个 `Bwt` 上构造 `FmdIndex` 视图，而不单独生成第二份索引。
  - `FmdIndex` 提供 `forward()`/`backward()` 操作，内部只持有 `Arc<Bwt>` 与 `l_pac` 元数据。

#### 1.3 索引 CLI 与并行策略

- `bwa-rust index` 子命令：
  - CLI 参数映射到 `IndexBuildConfig`（算法类型、block size、for-only 开关等），保持与 C 版 `bwa index` 兼容。
  - 流水线：FASTA 解析 → PAC/ANN/AMB 写出 → BWT 构建+OCC 注入 → SA 采样 → 写 `.sa`。
- 并行：
  - 初版以单线程保证 correctness；后续在 PAC→BWT 与 SA 构建阶段引入 chunk 级并行（参考 `block_size` 切分）。
  - 暴露 `--threads` 参数，但在早期版本中仅用于 BWT/SA 构建阶段。

### 阶段 2：BWA-backtrack 系列
1. `aln`
   - [ ] seed 扩展、错配罚、gap 动态规划。
   - [ ] 支持 `-n -o -e -k -l` 参数。
2. `samse/sampe`
   - [ ] SAM 生成、配对策略、insert size 估计。
   - [ ] 线程化读写 pipeline。
3. `bwa bwasw`
   - [ ] 局部长读比对、链选、banded DP。

#### 2.1 `aln` 流水与 Rust 模块划分

- `align::backtrack::aln`：
  - 入口接收 `ReadBatch` 与 `IndexBundle`，内部依赖 `bwt`/`bns`/`pac`；对标 `bwtaln.c + bwase.c`。
  - 设计 `AlnConfig`（打分、错配容忍、最大错配比例等）与 CLI (`bwa-rust aln`) 映射表。
- 算法阶段：
  - **seeding**：使用 `Bwt` 执行近似匹配（容忍少量错配），得到候选 SA 区间。
  - **extension**：在候选区间周围应用带惩罚的 DP（gap open/extend），生成局部比对得分。
  - **过滤与排序**：
    - 根据得分、错配数、比对长度进行剪枝；
    - 控制多重比对数量，保留 top-N 命中。
  - 输出为简化的中间结构（如 `BacktrackHit`），供 `samse/sampe` 阶段消费。

#### 2.2 `samse/sampe` 与配对策略

- `align::backtrack::sam`：
  - `samse`：将单端 hits 转换为 SAM 记录，考虑软剪切、标志位、MAPQ 估计；
  - `sampe`：在成对 reads 的 hits 之间执行模板配对：估计 insert size 分布、过滤不合理的 orientation/距离。
- 插入片段估计：
  - 收集一批高质量成对比对，计算平均长度与方差；
  - 将估计结果缓存到 `PairStats` 中，重复使用，避免每批次重算。
- Pipeline：延续 `seqio` 的 `producer/worker/consumer`：
  - `aln` 阶段只产生 hits，不直接输出 SAM；
  - `samse/sampe` 阶段负责最终 SAM 生成与写出，便于后续切换 BAM writer。

#### 2.3 `bwasw` / banded DP 设计

- `align::sw` 模块：
  - 对标 `bwasw2_*`，处理长读局部比对；
  - 内部复用 `ksw` DP 核心（可先移植标量版本，再用 SIMD 优化）。
- 设计要点：
  - 抽象 `SwTask`，包含参考窗口与读序列；
  - 提供 `SwAligner::align(task) -> SwResult`，内含 CIGAR 与得分；
  - 后续可被 MEM 与 backtrack 共用，避免重复实现 DP。

### 阶段 3：BWA-MEM
1. **FASTMAP & MEM seeding**：重写 `bwamem.c` 中的 MEM 检测、链合并。
2. **链到比对**：实现 `bwa_gen_cigar2` 等 KSW 扩展，支持 `-x` 预设。
3. **配对处理**：模板 insert size、二次比对、supplementary/secondary flag。
4. **SIMD/多线程**：绑定 `rayon` 或 `crossbeam`，可选 `packed_simd`/`std::simd`。

#### 3.1 MEM seeding & 链化

- `align::mem::seed`：
  - 重写 `fastmap.c + bwamem.c` 的 MEM 探测流程，基于 `FmdIndex` 实现 `smem` 迭代器；
  - 利用前面定义的 `MemConfig::seeding`（`min_seed_len/split_width/max_occ` 等）控制 seed 生成。
- `align::mem::chain`：
  - 实现 seed 链化算法，按坐标和方向将 MEM 组合成候选链；
  - 使用 `MemConfig::chaining.max_gap/min_chain_weight` 控制链合并与剪枝；
  - 输出 `MemChain` 列表，供后续 DP 扩展使用。

#### 3.2 链到 DP 比对与 CIGAR 生成

- `align::mem::extend`：
  - 对每个 `MemChain`，调用 `ksw` 实现的 banded SW 在链间空隙和两端做局部比对；
  - 控制 band 宽度、Z-drop（`MemConfig::alignment.w/z_drop`）。
- CIGAR 生成：
  - 移植/复用 `bwa_gen_cigar2` 逻辑，根据 DP 路径与参考坐标生成标准 CIGAR；
  - 添加 NM 计算与 clipping 处理，向 `mem_aln_t` 填充必要字段。

#### 3.3 配对处理与标志位

- `align::mem::pairing`：
  - 收集 interleaved reads 的对齐结果，根据 `MemConfig::pairing` 与 `MemConfig::output` 决定 primary/secondary/supplementary；
  - 估计 insert size 分布（`mem_pestat` 对应实现），筛掉异常 pair；
  - 设置 SAM flag（proper pair, first/second in pair, supplementary 等）。

#### 3.4 SIMD 与多线程策略

- 多线程：
  - 沿用 `ReadBatch` 级任务划分，每个 worker 线程独立运行 MEM 全流水，避免细粒度锁；
  - `MemConfig::flags` 中的线程数由 CLI 控制，默认 1。
- SIMD：
  - 初期实现纯标量 DP 核心，保证结果与 C 版一致；
  - 后续提供可选的 `feature = "simd"`，以 `std::simd` 或 `packed_simd` 提升 DP 性能。

### 阶段 4：工具与整合
- [ ] `fastmap`, `pemerge`, `xa2multi` 等附属工具。
- [ ] CLI 兼容层（解析 `bwa` 风格参数，支持 `@RG` 注入）。
- [ ] 完整文档与示例脚本。

#### 4.1 附属工具在 Rust 中的定位

- `fastmap`：
  - 对标 C 版 `fastmap.c`，复用 `align::mem` 的 seeding/链化管线，仅输出候选命中（不做完整比对）。
  - Rust 侧提供二进制 `bwa-rust-fastmap` 或 `bwa-rust fastmap` 子命令，输出可供下游 MEM/其他工具消费的中间格式（如 JSON lines / TSV）。
- `pemerge`：
  - 映射到 `tools::pemerge` 模块，使用 `seqio::reader`/`ReadBatch`，实现 read merge 逻辑与错误统计，参考 `pemerge.c`。
  - 输出合并后 FASTQ 或 SAM，并记录错误分布到 stderr（便于可视化）。
- `xa2multi`：
  - 设计 `tools::xa2multi`，负责将 XA 标签展开为多条 SAM/BAM 记录；
  - 在 Rust 中可直接处理 BAM 流（借助 `rust-htslib` 或自研 SAM/BAM writer）。

#### 4.2 CLI 兼容层设计

- 顶层二进制 `bwa-rust`：
  - 使用 `clap` 构建与 C 版相近的子命令与参数集合（`index/aln/samse/sampe/mem/bwasw/fastmap` 等）；
  - 入口统一解析 RG/PG、线程数、预设 (`-x`) 等，再分派到内部模块。
- `@RG` 注入：
  - 在 CLI 层将 `-R` 或配置文件中的 RG line 注入到 `util::sam::HeaderConfig`；
  - `sam` 输出模块负责将 RG 写入 header，并在每条记录中正确设置 `RG` tag。
- 兼容行为：
  - 优先保证常用子命令的参数名和默认值与 BWA 一致；
  - 对新增的 Rust-only 功能（如 JSON stats）使用长参数名（`--json-stats` 等），不影响原语义。

#### 4.3 文档与示例脚本

- 文档结构：
  - `docs/usage.md`：面向使用者的 CLI 手册，给出与 BWA 的兼容性说明。
  - `docs/internals/*.md`：按模块（`index/align::mem/align::backtrack/seqio`）说明内部数据结构和算法要点。
- 示例脚本：
  - `examples/` 下提供：
    - `run-index.sh`：从 FASTA 构建索引；
    - `run-mem.sh`：MEM 比对单端/双端 reads；
    - `compare-bwa.sh`：调用原版 BWA 和 `bwa-rust`，对比 SAM 差异。

---

### 阶段 5：验证
- [ ] 使用官方测试 FASTA/FASTQ 对比 SAM（diff+容忍字段）。
- [ ] 性能基线（单线程 vs 多线程）。
- [ ] 发布 CI：cargo test + Golden tests + benchmark。

#### 5.1 正确性验证策略

- **Golden tests**：
  - 在 `tests/golden/` 放置小规模参考 + reads + 预期 SAM；
  - 提供 `golden::compare_sam(a, b, Tolerance)` 工具，允许在 MAPQ、次要 tags 上有可控偏差；
  - 针对 `index/aln/mem/bwasw` 各阶段都准备典型 case（小基因组、短读、长读、含 N 区域等）。
- **与原 BWA 对比**：
  - 脚本层：调用原 `bwa` 和 `bwa-rust`，生成两份 SAM/BAM；
  - 比较时关注：
    - 主比对位置、CIGAR、FLAG、MAPQ；
    - 允许某些 tags（如 `MC`, `SA`, 自定义 tags）存在差异，只统计差异比例。

#### 5.2 性能基线与基准测试

- **单线程基线**：
  - 对 `index`、`mem`、`aln` 在单线程模式下与原 BWA 对比总耗时和峰值内存；
  - 所有重大改动需保证不显著回退（可以在 `docs/perf.md` 中记录结果）。
- **多线程扩展**：
  - 随着 MEM/Backtrack 并行化完善，引入多线程基准（如 4/8/16 线程），记录 speedup；
  - 对 pipeline 拥塞点（I/O、DP、写出）进行简单 profiling。

#### 5.3 CI 规划

- 基础 CI：
  - `cargo fmt --check` / `cargo clippy -- -D warnings`；
  - `cargo test --workspace`；
  - Golden tests（含与原 BWA 的输出 diff）在 CI 中运行小规模数据集。
- 扩展 CI：
  - nightly/weekly job 跑中等规模数据（人类 chr1 等），输出性能报告和 correctness summary 到 artifact 中。

---

## 关键技术决策
1. **语言特性**：尽量安全实现；性能热点允许 `unsafe`（SIMD、内存映射）。
2. **数据兼容**：优先生成与 BWA 完全兼容的索引文件，以便互操作。
3. **并行策略**：索引阶段使用 chunked 任务；对齐阶段沿用 task pool。
4. **依赖约束**：不引入 GPL 污染第三方库；自研或 MIT/Apache 兼容。

## 风险与缓解
- **实现规模大**：拆分阶段、保持可运行里程碑。
- **性能差异**：优先 correctness，再在关键路径（BWT、KSW）进行优化。
- **测试数据**：复用原仓库 `example.c` 等脚本，必要时生成模拟 reads。

## 下一步
1. 细化阶段 0 调研结果，输出模块依赖图。
2. 开始在 Rust 项目中搭建 `cli` 子命令骨架（`aln/mem` 空实现）。
3. 规划索引文件读写格式并 stub API。

## 模块依赖图与依赖关系（草案）

### 模块层次

- 最底层：
  - `util`：通用工具（`ksw`, `thread`, `sam`, `config`），不依赖上层模块。
  - `seqio`：FASTX 读取与批处理，仅依赖标准库和少量第三方 crate。
- 中间层：
  - `index`：`pac/ann/amb/bwt/sa/fmd` 索引读写与构建，依赖 `util`，不依赖 `align`。
- 上层：
  - `align::backtrack` / `align::mem` / `align::sw`：依赖 `index`、`seqio`、`util`。
- 顶层：
  - `cli` & 工具二进制：依赖所有下层模块，但不被任何模块依赖。

### 依赖关系示意（简化）

```text
        +-----------+        +-----------+
        |  cli/bin  |        |  tools/*  |
        +-----+-----+        +-----+-----+
              |                    |
              v                    v
        +-----------+        +-----------+
        |   align   |<-------+   seqio   |
        +-----+-----+        +-----------+
              |
              v
        +-----------+
        |   index   |
        +-----+-----+
              |
              v
        +-----------+
        |   util    |
        +-----------+
```

约束原则：
- `util` 不依赖任何项目内部模块；
- `index` 不依赖 `align`/`cli`；
- `align` 不依赖 `cli`，只通过配置结构接收参数；
- `cli` 不引入核心算法逻辑，仅做参数解析和模块调度。
