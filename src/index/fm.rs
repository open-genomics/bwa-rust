use std::io::{Read, Write};
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::error::BwaError;

const FM_MAGIC: u64 = 0x424D_4146_4D5F_5253; // "BWAFM_RS"
const FM_VERSION: u32 = 3;

/// `.fm` 文件头：magic/version 作为独立前缀写在 bincode 载荷之前，
/// 使格式错误（坏 magic、坏版本）无需反序列化即可被识别。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct FileHeader {
    magic: u64,
    version: u32,
    /// 载荷（bincode 序列化的 `FmBody`）的 FNV-1a 64 位校验和，
    /// 用于检测位翻转等静默损坏。
    checksum: u64,
}

fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct IndexMeta {
    pub reference_file: Option<String>,
    pub build_args: Option<String>,
    pub build_timestamp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Contig {
    pub name: String,
    pub len: u32,
    pub offset: u32,
}

/// 朴素 FM 索引实现：
/// - 支持任意有限字母表，字母以 [0..sigma) 进行编码（0 预留为 $）。
/// - 采用定长分块的 Occ 采样（块内顺扫补偿），便于后续替换为压缩结构。
/// - 保存完整 SA（MVP），方便从区间获得位置；后续可替换为稀疏采样。
///
/// # 内存占用说明
///
/// `text` 字段保存原始文本，占用 O(n) 空间。这是必要的权衡：
/// - 比对时需要参考序列进行 Smith-Waterman 扩展
/// - 从 BWT 重建文本的开销远大于保存原始文本
/// - 未来可考虑压缩存储或按需加载
///
/// # 不变量
///
/// - `bwt`/`sa` 与 `text` 长度一致
/// - contig 偏移连续且不越界（`validate` 在加载时校验）
/// - 内部字段私有，构造必须经过 [`FMIndex::build`]；直接操作内部字段可能破坏上述不变量
#[derive(Debug, Serialize, Deserialize)]
pub struct FMIndex {
    magic: u64,
    version: u32,
    sigma: u8,
    block: u32,
    /// C\[i\] = 文本中字母 < i 的累计数量
    c: Vec<u32>,
    /// BWT 序列（与 SA 同长度）
    bwt: Vec<u8>,
    /// Occ 采样（按块存储，行优先展平）：occ_samples[block_id * sigma + c]
    occ_samples: Vec<u32>,
    /// SA（完整或稀疏采样）
    sa: Vec<u32>,
    /// 稀疏 SA 采样间隔（0 表示完整 SA）
    sa_sample_rate: u32,
    /// contig 元信息（名称、长度、起始偏移）
    contigs: Vec<Contig>,
    /// 原始文本（数值化字母表，包含 contig 间的 0 分隔符）
    ///
    /// # 用途
    /// 比对时需要恢复参考序列进行 SW 扩展。虽然占用 O(n) 空间，
    /// 但避免了从 BWT 重建的 O(n) 时间开销。
    text: Vec<u8>,
    /// 可选的构建元数据
    meta: Option<IndexMeta>,
}

impl FMIndex {
    pub fn build(
        text: Vec<u8>,
        bwt: Vec<u8>,
        sa: Vec<u32>,
        contigs: Vec<Contig>,
        sigma: u8,
        block: usize,
    ) -> Result<Self> {
        if block == 0 {
            return Err(anyhow!("block size must be greater than zero"));
        }
        if sigma == 0 {
            return Err(anyhow!("sigma must be greater than zero"));
        }
        if bwt.len() != text.len() {
            return Err(anyhow!("BWT/text length mismatch"));
        }
        if sa.len() != text.len() {
            return Err(anyhow!("SA/text length mismatch"));
        }
        // 计算 C 表
        let mut freq = vec![0u32; sigma as usize];
        for &ch in &bwt {
            let ci = ch as usize;
            if ci < sigma as usize {
                freq[ci] += 1;
            }
        }
        let mut c = vec![0u32; sigma as usize];
        let mut acc = 0u32;
        for i in 0..sigma as usize {
            c[i] = acc;
            acc += freq[i];
        }

        // 采样 Occ
        let block_u = block as u32;
        let num_blocks = if bwt.is_empty() { 0 } else { bwt.len().div_ceil(block) };
        let mut occ_samples = vec![0u32; num_blocks * sigma as usize];
        let mut running = vec![0u32; sigma as usize];
        for bi in 0..num_blocks {
            // 记录到块起始位置的累计
            for a in 0..sigma as usize {
                occ_samples[bi * sigma as usize + a] = running[a];
            }
            // 扫描本块内容，更新 running
            let start = bi * block;
            let end = ((bi + 1) * block).min(bwt.len());
            for &ch in &bwt[start..end] {
                let ci = ch as usize;
                if ci < sigma as usize {
                    running[ci] += 1;
                }
            }
        }

        Ok(Self {
            magic: FM_MAGIC,
            version: FM_VERSION,
            sigma,
            block: block_u,
            c,
            bwt,
            occ_samples,
            sa,
            sa_sample_rate: 0,
            contigs,
            text,
            meta: None,
        })
    }

    /// 构建使用稀疏 SA 采样的 FM 索引
    pub fn build_sparse(
        text: Vec<u8>,
        bwt: Vec<u8>,
        sa: Vec<u32>,
        contigs: Vec<Contig>,
        sigma: u8,
        block: usize,
        sa_rate: u32,
    ) -> Result<Self> {
        let mut fm = Self::build(text, bwt, sa, contigs, sigma, block)?;
        if sa_rate > 1 {
            fm.sparsify_sa(sa_rate);
        }
        Ok(fm)
    }

    /// 将完整 SA 转换为稀疏采样
    fn sparsify_sa(&mut self, rate: u32) {
        let n = self.sa.len();
        let mut sparse = Vec::with_capacity(n / rate as usize + 1);
        for i in (0..n).step_by(rate as usize) {
            sparse.push(self.sa[i]);
        }
        self.sa = sparse;
        self.sa_sample_rate = rate;
    }

    /// 通过 LF-mapping 从稀疏 SA 恢复任意位置的 SA 值
    pub fn sa_value(&self, mut idx: usize) -> u32 {
        if self.sa_sample_rate <= 1 {
            return self.sa[idx];
        }
        let rate = self.sa_sample_rate as usize;
        let n = self.bwt.len() as u32;
        let mut steps = 0u32;
        loop {
            if idx % rate == 0 {
                return (self.sa[idx / rate] + steps) % n;
            }
            // LF-mapping: idx = C[BWT[idx]] + Occ(BWT[idx], idx)
            let ch = self.bwt[idx];
            idx = self.c[ch as usize] as usize + self.occ(ch, idx) as usize;
            steps += 1;
        }
    }

    pub fn set_meta(&mut self, meta: IndexMeta) {
        self.meta = Some(meta);
    }

    pub fn sigma(&self) -> u8 {
        self.sigma
    }

    pub fn block_size(&self) -> u32 {
        self.block
    }

    pub fn c_table(&self) -> &[u32] {
        &self.c
    }

    pub fn bwt(&self) -> &[u8] {
        &self.bwt
    }

    pub fn occ_samples(&self) -> &[u32] {
        &self.occ_samples
    }

    pub fn sa(&self) -> &[u32] {
        &self.sa
    }

    pub fn sa_sample_rate(&self) -> u32 {
        self.sa_sample_rate
    }

    pub fn contigs(&self) -> &[Contig] {
        &self.contigs
    }

    pub fn text(&self) -> &[u8] {
        &self.text
    }

    pub fn meta(&self) -> Option<&IndexMeta> {
        self.meta.as_ref()
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    /// 解码并返回指定 contig 的完整参考序列（内部字母表 → ASCII）。
    ///
    /// 返回 `None` 当 contig 索引越界或区间超出文本范围（正常情况下不会发生，
    /// 因为加载时已校验；此处防御性检查用于库消费者手工构造的场景）。
    pub fn contig_sequence(&self, ci: usize) -> Option<Vec<u8>> {
        let contig = self.contigs.get(ci)?;
        let start = contig.offset as usize;
        let end = start.checked_add(contig.len as usize)?;
        if end > self.text.len() {
            return None;
        }
        Some(
            self.text[start..end]
                .iter()
                .map(|&code| crate::util::dna::from_alphabet(code))
                .collect(),
        )
    }

    fn validate(&self) -> Result<()> {
        if self.sigma == 0 {
            return Err(anyhow!("invalid FM index file: sigma must be greater than zero"));
        }
        if self.block == 0 {
            return Err(anyhow!("invalid FM index file: block size must be greater than zero"));
        }
        if self.bwt.len() != self.text.len() {
            return Err(anyhow!("invalid FM index file: BWT/text length mismatch"));
        }
        if self.c.len() != self.sigma as usize {
            return Err(anyhow!("invalid FM index file: C table length does not match sigma"));
        }
        let expected_blocks = if self.bwt.is_empty() {
            0
        } else {
            self.bwt.len().div_ceil(self.block as usize)
        };
        let expected_occ_len = expected_blocks * self.sigma as usize;
        if self.occ_samples.len() != expected_occ_len {
            return Err(anyhow!(
                "invalid FM index file: occ_samples length mismatch (expected {}, got {})",
                expected_occ_len,
                self.occ_samples.len()
            ));
        }
        if self.sa_sample_rate <= 1 {
            if self.sa.len() != self.bwt.len() {
                return Err(anyhow!("invalid FM index file: SA length mismatch"));
            }
        } else {
            let rate = self.sa_sample_rate as usize;
            let expected_sa_len = self.bwt.len().div_ceil(rate);
            if self.sa.len() != expected_sa_len {
                return Err(anyhow!(
                    "invalid FM index file: sparse SA length mismatch (expected {}, got {})",
                    expected_sa_len,
                    self.sa.len()
                ));
            }
        }
        for (i, &ch) in self.bwt.iter().enumerate() {
            if ch as usize >= self.sigma as usize {
                return Err(anyhow!("invalid FM index file: BWT symbol out of range at {}", i));
            }
        }
        for (i, &pos) in self.sa.iter().enumerate() {
            if pos as usize >= self.text.len() {
                return Err(anyhow!("invalid FM index file: SA position out of range at {}", i));
            }
        }
        let mut expected_offset = 0u32;
        for contig in &self.contigs {
            if contig.offset != expected_offset {
                return Err(anyhow!("invalid FM index file: contig offsets are not contiguous"));
            }
            let end = contig
                .offset
                .checked_add(contig.len)
                .ok_or_else(|| anyhow!("invalid FM index file: contig range overflow"))?;
            if end as usize > self.text.len() {
                return Err(anyhow!("invalid FM index file: contig range exceeds text length"));
            }
            expected_offset = end.saturating_add(1);
        }
        Ok(())
    }

    #[inline]
    pub fn occ(&self, c: u8, pos: usize) -> u32 {
        // 返回 BWT[0..pos) 中 c 的出现次数
        if pos == 0 {
            return 0;
        }
        let sigma_us = self.sigma as usize;
        let block = self.block as usize;
        let bi = (pos - 1) / block; // 所在块编号
        let base = self.occ_samples[bi * sigma_us + c as usize];
        let start = bi * block;
        let mut add = 0u32;
        for &ch in &self.bwt[start..pos] {
            if ch == c {
                add += 1;
            }
        }
        base + add
    }

    #[inline]
    pub fn rank_range(&self, c: u8, l: usize, r: usize) -> (usize, usize) {
        // 返回在区间 [l, r) 上扩展字符 c 后的新区间
        let c0 = self.c[c as usize] as usize;
        let nl = c0 + self.occ(c, l) as usize;
        let nr = c0 + self.occ(c, r) as usize;
        (nl, nr)
    }

    /// 反向搜索精确匹配，pat 已经是编码后的字母表（不应包含 0）
    pub fn backward_search(&self, pat: &[u8]) -> Option<(usize, usize)> {
        if self.bwt.is_empty() {
            return None;
        }
        let mut l = 0usize;
        let mut r = self.bwt.len();
        for &a in pat.iter().rev() {
            let (nl, nr) = self.rank_range(a, l, r);
            if nl >= nr {
                return None;
            }
            l = nl;
            r = nr;
        }
        Some((l, r))
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let body = bincode::serialize(self).map_err(|e| anyhow!("cannot serialize FM index: {}", e))?;
        let header = FileHeader {
            magic: FM_MAGIC,
            version: FM_VERSION,
            checksum: fnv1a64(&body),
        };
        let mut header_bytes = bincode::serialize(&header).map_err(|e| anyhow!("cannot serialize FM header: {}", e))?;
        let mut f = std::fs::File::create(path)?;
        f.write_all(&header_bytes)?;
        header_bytes.clear();
        f.write_all(&body)?;
        f.flush()?;
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let mut f = std::fs::File::open(path.as_ref())?;
        let header: FileHeader = bincode::deserialize_from(&mut f)
            .map_err(|_| BwaError::IndexFormat("not a bwa-rust FM index (missing or corrupt header)".into()))?;
        if header.magic != FM_MAGIC {
            return Err(BwaError::IndexFormat(format!(
                "bad magic number (expected 0x{:016X}, got 0x{:016X})",
                FM_MAGIC, header.magic
            ))
            .into());
        }
        if header.version != FM_VERSION {
            return Err(BwaError::IndexFormat(format!(
                "unsupported FM index version: expected {}, got {}",
                FM_VERSION, header.version
            ))
            .into());
        }
        let mut body = Vec::new();
        f.read_to_end(&mut body)?;
        let actual = fnv1a64(&body);
        if actual != header.checksum {
            return Err(BwaError::IndexFormat(format!(
                "checksum mismatch: file is corrupt (expected {:016X}, got {:016X})",
                header.checksum, actual
            ))
            .into());
        }
        let idx: Self = bincode::deserialize(&body)
            .map_err(|e| BwaError::IndexFormat(format!("cannot deserialize FM index body: {}", e)))?;
        idx.validate()?;
        Ok(idx)
    }

    /// 取出 SA 区间对应的文本位置
    pub fn sa_interval_positions(&self, l: usize, r: usize) -> Vec<u32> {
        if self.sa_sample_rate <= 1 {
            self.sa[l..r].to_vec()
        } else {
            (l..r).map(|i| self.sa_value(i)).collect()
        }
    }

    pub fn for_each_sa_interval_position<F>(&self, l: usize, r: usize, mut f: F)
    where
        F: FnMut(u32),
    {
        if self.sa_sample_rate <= 1 {
            for &pos in &self.sa[l..r] {
                f(pos);
            }
        } else {
            for i in l..r {
                f(self.sa_value(i));
            }
        }
    }

    /// 将文本位置映射到 (contig_index, contig_offset)。若落在分隔符($)位置，则返回 None。
    pub fn map_text_pos(&self, pos: u32) -> Option<(usize, u32)> {
        if self.contigs.is_empty() {
            return None;
        }
        let mut lo = 0usize;
        let mut hi = self.contigs.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            let c = &self.contigs[mid];
            if pos < c.offset {
                hi = mid;
            } else if pos >= c.offset + c.len {
                lo = mid + 1;
            } else {
                return Some((mid, pos - c.offset));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{bwt, sa};

    fn build_toy_fm(text_bytes: &[u8]) -> FMIndex {
        let mut text: Vec<u8> = text_bytes.to_vec();
        let len = text.len() as u32;
        let contigs = vec![Contig {
            name: "seq1".to_string(),
            len,
            offset: 0,
        }];
        text.push(0); // sentinel
        let sa_arr = sa::build_sa(&text);
        let bwt_arr = bwt::build_bwt(&text, &sa_arr);
        FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4).unwrap()
    }

    #[test]
    fn fm_build_basic_fields() {
        let fm = build_toy_fm(&[1, 2, 3, 4]); // ACGT
        assert_eq!(fm.magic, FM_MAGIC);
        assert_eq!(fm.version, FM_VERSION);
        assert_eq!(fm.sigma, 6);
        assert_eq!(fm.contigs.len(), 1);
        assert_eq!(fm.contigs[0].name, "seq1");
        assert_eq!(fm.contigs[0].len, 4);
        assert_eq!(fm.sa.len(), 5); // text len = 5 (ACGT$)
    }

    #[test]
    fn fm_backward_search_finds_pattern() {
        let fm = build_toy_fm(&[1, 2, 3, 4, 1, 2]); // ACGTAC
                                                    // search for "AC" = [1,2]
        let res = fm.backward_search(&[1, 2]);
        assert!(res.is_some());
        let (l, r) = res.unwrap();
        assert!(r > l);
        assert_eq!(r - l, 2); // "AC" appears twice
    }

    #[test]
    fn fm_backward_search_not_found() {
        let fm = build_toy_fm(&[1, 2, 3, 4]); // ACGT
                                              // search for "TT" = [4,4] — should not exist
        let res = fm.backward_search(&[4, 4]);
        assert!(res.is_none());
    }

    #[test]
    fn fm_save_load_roundtrip() {
        let fm = build_toy_fm(&[1, 2, 3, 4, 1, 2, 3]);
        let tmp = std::env::temp_dir().join("bwa_rust_test_fm_roundtrip.fm");
        let path = tmp.to_str().unwrap();
        fm.save_to_file(path).unwrap();
        let loaded = FMIndex::load_from_file(path).unwrap();
        assert_eq!(loaded.magic, fm.magic);
        assert_eq!(loaded.version, fm.version);
        assert_eq!(loaded.sigma, fm.sigma);
        assert_eq!(loaded.block, fm.block);
        assert_eq!(loaded.c, fm.c);
        assert_eq!(loaded.bwt, fm.bwt);
        assert_eq!(loaded.sa, fm.sa);
        assert_eq!(loaded.text, fm.text);
        assert_eq!(loaded.contigs.len(), fm.contigs.len());
        assert_eq!(loaded.contigs[0].name, fm.contigs[0].name);
        assert_eq!(loaded.contigs[0].len, fm.contigs[0].len);
        assert_eq!(loaded.contigs[0].offset, fm.contigs[0].offset);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn fm_map_text_pos_basic() {
        // Two contigs: [0..3) and [4..7), separator at pos 3
        let text = vec![1u8, 2, 3, 0, 1, 3, 4, 0];
        let contigs = vec![
            Contig {
                name: "c1".to_string(),
                len: 3,
                offset: 0,
            },
            Contig {
                name: "c2".to_string(),
                len: 3,
                offset: 4,
            },
        ];
        let sa_arr = sa::build_sa(&text);
        let bwt_arr = bwt::build_bwt(&text, &sa_arr);
        let fm = FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4).unwrap();

        assert_eq!(fm.map_text_pos(0), Some((0, 0)));
        assert_eq!(fm.map_text_pos(2), Some((0, 2)));
        assert_eq!(fm.map_text_pos(3), None); // separator
        assert_eq!(fm.map_text_pos(4), Some((1, 0)));
        assert_eq!(fm.map_text_pos(6), Some((1, 2)));
        assert_eq!(fm.map_text_pos(7), None); // separator
        assert_eq!(fm.map_text_pos(100), None);
    }

    #[test]
    fn fm_occ_correctness() {
        let fm = build_toy_fm(&[1, 2, 1, 2, 3]); // ACACG$
                                                 // Verify occ counts are consistent: occ(c, n) should equal total frequency of c in BWT
        let n = fm.bwt.len();
        for c in 0..fm.sigma {
            let total = fm.occ(c, n);
            let manual: u32 = fm.bwt.iter().filter(|&&b| b == c).count() as u32;
            assert_eq!(total, manual, "occ mismatch for c={}", c);
        }
    }

    #[test]
    fn fm_sa_interval_positions_returns_all() {
        let fm = build_toy_fm(&[1, 2, 3, 1, 2, 3]); // ACGACG
                                                    // "ACG" appears twice
        let res = fm.backward_search(&[1, 2, 3]).unwrap();
        let positions = fm.sa_interval_positions(res.0, res.1);
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn fm_sparse_sa_roundtrip() {
        let mut text: Vec<u8> = vec![1, 2, 3, 4, 1, 2, 3, 4, 1, 2];
        let len = text.len() as u32;
        let contigs = vec![Contig {
            name: "s1".to_string(),
            len,
            offset: 0,
        }];
        text.push(0);
        let sa_arr = sa::build_sa(&text);
        let bwt_arr = bwt::build_bwt(&text, &sa_arr);
        let full_sa = sa_arr.clone();
        let fm = FMIndex::build_sparse(text.clone(), bwt_arr, sa_arr, contigs, 6, 4, 4).unwrap();
        assert_eq!(fm.sa_sample_rate, 4);
        // Verify sa_value recovers correct positions via LF-mapping
        for (i, &sa_value) in full_sa.iter().enumerate() {
            assert_eq!(fm.sa_value(i), sa_value, "sa_value mismatch at i={}", i);
        }
    }

    #[test]
    fn fm_sparse_sa_backward_search() {
        let mut text: Vec<u8> = vec![1, 2, 3, 4, 1, 2, 3];
        let len = text.len() as u32;
        let contigs = vec![Contig {
            name: "s1".to_string(),
            len,
            offset: 0,
        }];
        text.push(0);
        let sa_arr = sa::build_sa(&text);
        let bwt_arr = bwt::build_bwt(&text, &sa_arr);
        let fm = FMIndex::build_sparse(text, bwt_arr, sa_arr, contigs, 6, 4, 2).unwrap();
        let res = fm.backward_search(&[1, 2, 3]);
        assert!(res.is_some());
        let (l, r) = res.unwrap();
        let positions = fm.sa_interval_positions(l, r);
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn fm_rank_range_monotonic() {
        let fm = build_toy_fm(&[1, 2, 3, 4, 1, 2, 3, 4]);
        let n = fm.bwt.len();
        for c in 1..fm.sigma {
            let mut prev_l = 0;
            for pos in 0..=n {
                let (nl, _nr) = fm.rank_range(c, 0, pos);
                assert!(nl >= prev_l, "rank_range not monotonic for c={}, pos={}", c, pos);
                prev_l = nl;
            }
        }
    }

    #[test]
    fn fm_backward_search_single_char() {
        let fm = build_toy_fm(&[1, 2, 3, 4, 1]); // ACGTA
        for c in 1u8..=4 {
            let res = fm.backward_search(&[c]);
            assert!(res.is_some(), "single char {} not found", c);
        }
        // N (5) not in text
        assert!(fm.backward_search(&[5]).is_none());
    }

    #[test]
    fn fm_backward_search_full_text() {
        let text_bytes = vec![1u8, 2, 3, 4];
        let fm = build_toy_fm(&text_bytes);
        let res = fm.backward_search(&text_bytes);
        assert!(res.is_some());
        let (l, r) = res.unwrap();
        assert_eq!(r - l, 1);
    }

    #[test]
    fn fm_map_text_pos_single_contig() {
        let fm = build_toy_fm(&[1, 2, 3, 4, 5]);
        assert_eq!(fm.map_text_pos(0), Some((0, 0)));
        assert_eq!(fm.map_text_pos(4), Some((0, 4)));
        assert_eq!(fm.map_text_pos(5), None); // sentinel
    }

    #[test]
    fn fm_occ_boundary_values() {
        let fm = build_toy_fm(&[1, 1, 1, 2, 2, 3]);
        assert_eq!(fm.occ(1, 0), 0);
        assert_eq!(fm.occ(1, 1), fm.bwt[..1].iter().filter(|&&b| b == 1).count() as u32);
        let n = fm.bwt.len();
        let total_a = fm.bwt.iter().filter(|&&b| b == 1).count() as u32;
        assert_eq!(fm.occ(1, n), total_a);
    }

    #[test]
    fn fm_save_load_with_meta() {
        let mut fm = build_toy_fm(&[1, 2, 3, 4]);
        fm.set_meta(IndexMeta {
            reference_file: Some("test.fa".to_string()),
            build_args: Some("bwa-rust index test.fa".to_string()),
            build_timestamp: Some("2025-01-01T00:00:00Z".to_string()),
        });
        let tmp = std::env::temp_dir().join("bwa_rust_test_fm_meta.fm");
        let path = tmp.to_str().unwrap();
        fm.save_to_file(path).unwrap();
        let loaded = FMIndex::load_from_file(path).unwrap();
        let meta = loaded.meta.unwrap();
        assert_eq!(meta.reference_file.as_deref(), Some("test.fa"));
        assert_eq!(meta.build_args.as_deref(), Some("bwa-rust index test.fa"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn fm_build_rejects_zero_block_size() {
        let text = vec![1u8, 2, 3, 4, 0];
        let sa_arr = sa::build_sa(&text);
        let bwt_arr = bwt::build_bwt(&text, &sa_arr);
        let contigs = vec![Contig {
            name: "seq1".to_string(),
            len: 4,
            offset: 0,
        }];
        assert!(FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 0).is_err());
    }

    #[test]
    fn fm_load_rejects_zero_block_size() {
        let mut fm = build_toy_fm(&[1, 2, 3, 4]);
        fm.block = 0;
        let tmp = std::env::temp_dir().join("bwa_rust_test_fm_invalid_block.fm");
        let path = tmp.to_str().unwrap();
        fm.save_to_file(path).unwrap();
        let err = FMIndex::load_from_file(path).unwrap_err();
        assert!(err.to_string().contains("block size"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn fm_load_rejects_invalid_occ_samples_len() {
        let mut fm = build_toy_fm(&[1, 2, 3, 4]);
        fm.occ_samples.pop();
        let tmp = std::env::temp_dir().join("bwa_rust_test_fm_invalid_occ.fm");
        let path = tmp.to_str().unwrap();
        fm.save_to_file(path).unwrap();
        let err = FMIndex::load_from_file(path).unwrap_err();
        assert!(err.to_string().contains("occ_samples"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn fm_load_rejects_truncated_file() {
        let fm = build_toy_fm(&[1, 2, 3, 4]);
        let tmp = std::env::temp_dir().join("bwa_rust_test_fm_truncated.fm");
        let path = tmp.to_str().unwrap();
        fm.save_to_file(path).unwrap();
        let data = std::fs::read(path).unwrap();
        std::fs::write(path, &data[..data.len() / 2]).unwrap();
        let err = FMIndex::load_from_file(path).unwrap_err();
        assert!(err.to_string().contains("checksum") || err.to_string().contains("deserialize"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn fm_load_rejects_random_bytes() {
        let tmp = std::env::temp_dir().join("bwa_rust_test_fm_random.fm");
        let path = tmp.to_str().unwrap();
        std::fs::write(
            path,
            b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f",
        )
        .unwrap();
        let err = FMIndex::load_from_file(path).unwrap_err();
        assert!(err.to_string().contains("header") || err.to_string().contains("magic"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn fm_load_rejects_flipped_bit_in_payload() {
        let fm = build_toy_fm(&[1, 2, 3, 4]);
        let tmp = std::env::temp_dir().join("bwa_rust_test_fm_bitflip.fm");
        let path = tmp.to_str().unwrap();
        fm.save_to_file(path).unwrap();
        let mut data = std::fs::read(path).unwrap();
        // 翻转载荷中部的一个字节（跳过 16 字节 bincode 头 + 24 字节 FileHeader 之后）
        let payload_start = 16 + 24;
        if data.len() > payload_start + 8 {
            let flip_at = payload_start + 8;
            data[flip_at] ^= 0x01;
            std::fs::write(path, &data).unwrap();
            let err = FMIndex::load_from_file(path).unwrap_err();
            assert!(err.to_string().contains("checksum"), "unexpected error: {}", err);
        }
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn fm_contig_sequence_roundtrip() {
        let fm = build_toy_fm(&[1, 2, 3, 4]);
        assert_eq!(fm.contig_sequence(0).as_deref(), Some(&b"ACGT"[..]));
        assert!(fm.contig_sequence(5).is_none());
    }
}
