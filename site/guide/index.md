# 使用指南

本指南只覆盖已交付的单端流程：FASTA 参考序列、FASTQ reads、`.fm` 索引和 SAM 输出。

## 标准路径

```bash
bwa-rust index reference.fa -o ref
bwa-rust align -i ref.fm reads.fq -o output.sam
```

也可以用 `mem` 在内存中构建索引并立即比对：

```bash
bwa-rust mem reference.fa reads.fq -t 4 -o output.sam
```

## 参数真值

CLI 默认值与 `src/align/mod.rs` 的 `AlignOpt::default()` 保持一致。常用参数：

| 参数 | 默认值 | CLI |
|------|--------|-----|
| match score | `2` | `--match` / `-A` |
| mismatch penalty | `1` | `--mismatch` / `-B` |
| gap open | `2` | `--gap-open` / `-O` |
| gap extend | `1` | `--gap-ext` / `-E` |
| band width | `16` | `--band-width` / `-w` |
| score threshold | `20` | `--score-threshold` / `-T` |
| min seed length | `19` | `--min-seed-len` / `-k` |
| z-drop | `100` | `--z-drop` / `-d` |

## 下一步

- [安装](/guide/installation)
- [快速开始](/guide/quickstart)
- [比对流水线](/architecture/pipeline)
