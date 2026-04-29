# 快速开始

仓库自带 toy 数据，可用于确认 CLI 和输出格式。

## 构建索引

```bash
cargo run --release -- index data/toy.fa -o toy
```

输出：

```text
toy.fm
```

## 比对 reads

```bash
cargo run --release -- align -i toy.fm data/toy_reads.fq -o toy.sam
```

或一步执行：

```bash
cargo run --release -- mem data/toy.fa data/toy_reads.fq -t 4 -o toy.sam
```

## 查看 SAM

```bash
grep -v '^@' toy.sam
```

输出记录包含标准 SAM 字段，以及 `AS:i`、`XS:i`、`NM:i`、可用时的 `MD:Z` 和 `SA:Z` 标签。

## 清理临时文件

```bash
rm -f toy.fm toy.sam
```

## 调参示例

```bash
bwa-rust align -i toy.fm data/toy_reads.fq \
  --min-seed-len 15 \
  --max-occ 200 \
  --band-width 32 \
  --z-drop 80 \
  -o tuned.sam
```

所有默认值以 `AlignOpt::default()` 为准；`mem` 与 `align` 的普通默认值一致。
