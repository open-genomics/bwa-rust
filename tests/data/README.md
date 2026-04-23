# 测试数据

此目录用于存放真实数据测试所需的参考基因组和测序数据。

## 获取测试数据

### 大肠杆菌基因组 (~4.6 Mbp)

```bash
# 从 NCBI 下载
wget -O e_coli.fa.gz ftp://ftp.ncbi.nlm.nih.gov/genomes/all/GCF/000/005/845/GCF_000005845.2_ASM584v2/GCF_000005845.2_ASM584v2_genomic.fna.gz
gunzip e_coli.fa.gz

# 或使用 Ensembl
wget -O e_coli.fa.gz ftp://ftp.ensemblgenomes.org/pub/bacteria/release-57/fasta/bacteria_0_collection/escherichia_coli_str_k_12_substr_mg1655/dna/Escherichia_coli_str_k_12_substr_mg1655.ASM584v1.dna.toplevel.fa.gz
gunzip e_coli.fa.gz
```

### 生成模拟 reads

```bash
# 使用 ART 或 wgsim 生成模拟 reads
# 示例：生成 10000 条 150bp paired-end reads
wgsim -N 10000 -1 150 -2 150 e_coli.fa reads_1.fq reads_2.fq

# 或使用 ART
art_illumina -ss HS25 -i e_coli.fa -p -l 150 -f 10 -o reads_
```

## 运行真实数据测试

```bash
# 启用 real-data feature 运行测试
cargo test --features real-data --test real_data

# 运行性能基准测试
cargo test --features real-data --test real_data -- --ignored --nocapture
```

## 数据格式要求

- 参考基因组：FASTA 格式（`.fa` 或 `.fasta`）
- 测序数据：FASTQ 格式（`.fq` 或 `.fastq`）
- 支持 gzip 压缩（`.gz`）

## 注意事项

- 大肠杆菌基因组索引构建约需 10-30 秒（取决于硬件）
- 真实数据测试默认忽略，需显式启用 `--ignored` 参数
- 测试会与 BWA 输出对比，需确保系统已安装 BWA
