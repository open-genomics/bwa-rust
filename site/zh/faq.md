# 常见问题

## bwa-rust 能直接读取 BWA 索引吗？

不能。bwa-rust 使用自己的单文件 `.fm` 索引格式，需要用 `bwa-rust index` 构建。

## 输出和 BWA 完全一致吗？

不保证。bwa-rust 采用 BWA-MEM 风格的种子、链和延伸思想，但索引格式、MAPQ、启发式和部分 tie-break 都是项目自己的实现。

## 支持配对端吗？

当前稳定 CLI 只支持单端 FASTQ。仓库中有配对端 reader 和 insert-size 基础设施，但不能把它视为已交付能力。

## 支持 BAM 或 CRAM 吗？

不支持。当前输出格式是 SAM。

## 为什么保留完整参考文本在索引中？

比对延伸阶段需要参考序列片段。当前版本选择保存编码文本，换取实现清晰和读取简单；未来可以评估压缩或按需加载。

## 默认参数在哪里看？

`src/align/mod.rs` 的 `AlignOpt::default()` 是唯一真值。`align` 和 `mem` 的普通默认值应与它保持一致。

## 为什么文档站只有中文？

当前 Pages 是中文公共门户，英文入口保留在 `README.md` 和 docs.rs。未维护的英文站点不会被伪装成已存在。
