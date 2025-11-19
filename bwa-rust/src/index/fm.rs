use anyhow::Result;
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Serialize, Deserialize)]
pub struct FMIndex {
    pub sigma: u8,
    pub block: u32,
    /// C[i] = 文本中字母 < i 的累计数量
    pub c: Vec<u32>,
    /// BWT 序列（与 SA 同长度）
    pub bwt: Vec<u8>,
    /// Occ 采样（按块存储，行优先展平）：occ_samples[block_id * sigma + c]
    pub occ_samples: Vec<u32>,
    /// 完整 SA（MVP，可换稀疏）
    pub sa: Vec<u32>,
    /// contig 元信息（名称、长度、起始偏移）
    pub contigs: Vec<Contig>,
    /// 原始文本（数值化字母表，包含 contig 间的 0 分隔符）
    pub text: Vec<u8>,
}

impl FMIndex {
    pub fn build(text: Vec<u8>, bwt: Vec<u8>, sa: Vec<u32>, contigs: Vec<Contig>, sigma: u8, block: usize) -> Self {
        let n = bwt.len();
        let sigma_us = sigma as usize;
        // 计算 C 表
        let mut freq = vec![0u32; sigma_us];
        for &ch in &bwt {
            let ci = ch as usize;
            if ci < sigma_us { freq[ci] += 1; }
        }
        let mut c = vec![0u32; sigma_us];
        let mut acc = 0u32;
        for i in 0..sigma_us {
            c[i] = acc;
            acc += freq[i];
        }

        // 采样 Occ
        let block_u = block as u32;
        let num_blocks = if n == 0 { 0 } else { (n + block - 1) / block };
        let mut occ_samples = vec![0u32; num_blocks * sigma_us];
        let mut running = vec![0u32; sigma_us];
        for bi in 0..num_blocks {
            // 记录到块起始位置的累计
            for a in 0..sigma_us {
                occ_samples[bi * sigma_us + a] = running[a];
            }
            // 扫描本块内容，更新 running
            let start = bi * block;
            let end = ((bi + 1) * block).min(n);
            for &ch in &bwt[start..end] {
                let ci = ch as usize;
                if ci < sigma_us { running[ci] += 1; }
            }
        }

        Self { sigma, block: block_u, c, bwt, occ_samples, sa, contigs, text }
    }

    #[inline]
    pub fn occ(&self, c: u8, pos: usize) -> u32 {
        // 返回 BWT[0..pos) 中 c 的出现次数
        if pos == 0 { return 0; }
        let sigma_us = self.sigma as usize;
        let block = self.block as usize;
        let bi = (pos - 1) / block; // 所在块编号
        let base = self.occ_samples[bi * sigma_us + c as usize];
        let start = bi * block;
        let mut add = 0u32;
        for &ch in &self.bwt[start..pos] {
            if ch == c { add += 1; }
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
        if self.bwt.is_empty() { return None; }
        let mut l = 0usize;
        let mut r = self.bwt.len();
        for &a in pat.iter().rev() {
            let (nl, nr) = self.rank_range(a, l, r);
            if nl >= nr { return None; }
            l = nl; r = nr;
        }
        Some((l, r))
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let mut f = std::fs::File::create(path)?;
        bincode::serialize_into(&mut f, self)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self> {
        let f = std::fs::File::open(path)?;
        let idx: Self = bincode::deserialize_from(f)?;
        Ok(idx)
    }

    /// 取出 SA 区间对应的文本位置（MVP：直接从完整 SA 返回）。
    pub fn sa_interval_positions(&self, l: usize, r: usize) -> &[u32] {
        &self.sa[l..r]
    }

    /// 将文本位置映射到 (contig_index, contig_offset)。若落在分隔符($)位置，则返回 None。
    pub fn map_text_pos(&self, pos: u32) -> Option<(usize, u32)> {
        if self.contigs.is_empty() { return None; }
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
