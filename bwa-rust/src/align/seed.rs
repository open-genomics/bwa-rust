use crate::index::fm::FMIndex;

/// 对齐区域结构，类似 BWA 的 mem_alnreg_t
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlnReg {
    /// read 上的区间 [qb, qe)
    pub qb: usize,
    pub qe: usize,
    /// 参考上的区间 [rb, re)
    pub rb: u32,
    pub re: u32,
    /// contig 索引
    pub contig: usize,
    /// 对齐得分
    pub score: i32,
    /// 次优得分
    pub sub_score: i32,
    /// CIGAR
    pub cigar: String,
    /// edit distance
    pub nm: u32,
    /// 是否反向互补
    pub is_rev: bool,
}

/// MEM 种子
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemSeed {
    pub contig: usize,
    pub qb: usize,
    pub qe: usize,
    pub rb: u32,
    pub re: u32,
}

/// SMEM 搜索：对 read 的每个位置，找到以该位置为起点的最长精确匹配（MEM）。
/// 使用 FM 索引的 backward_search 逐步延伸。
/// 之后过滤被包含的种子，仅保留超级最大精确匹配（SMEM）。
pub fn find_smem_seeds(
    fm: &FMIndex,
    query_alpha: &[u8],
    min_len: usize,
) -> Vec<MemSeed> {
    let n = query_alpha.len();
    if min_len == 0 || n == 0 || min_len > n {
        return Vec::new();
    }

    // 第一步：为每个起始位置找到最长精确匹配
    let mut raw_mems: Vec<(usize, usize, usize, usize)> = Vec::new(); // (qb, qe, sa_l, sa_r)

    for qb in 0..n {
        if qb + min_len > n {
            break;
        }

        let mut best_len = 0usize;
        let mut best_l = 0usize;
        let mut best_r = 0usize;

        // 逐步增加长度，使用 backward_search
        let mut l = 0usize;
        let mut r = fm.bwt.len();
        // 从 qb+len-1 向 qb 逆向扩展（backward search 的自然方向）
        // 但我们需要按正序查找子串 query[qb..qb+len]
        // backward_search 已经内部反转，所以直接调用即可
        let mut len = min_len;
        while qb + len <= n {
            let pat = &query_alpha[qb..qb + len];
            match fm.backward_search(pat) {
                Some((sl, sr)) if sl < sr => {
                    best_len = len;
                    best_l = sl;
                    best_r = sr;
                    len += 1;
                }
                _ => break,
            }
        }

        if best_len >= min_len {
            raw_mems.push((qb, qb + best_len, best_l, best_r));
        }
    }

    // 第二步：过滤被包含的 MEM，保留 SMEM
    filter_contained(&mut raw_mems);

    // 第三步：将区间展开为具体种子
    let mut seeds = Vec::new();
    for (qb, qe, l, r) in &raw_mems {
        for sa_pos in fm.sa_interval_positions(*l, *r) {
            if let Some((ci, off)) = fm.map_text_pos(sa_pos) {
                let seed_len = (qe - qb) as u32;
                let contig_len = fm.contigs[ci].len as u32;
                if off + seed_len <= contig_len {
                    seeds.push(MemSeed {
                        contig: ci,
                        qb: *qb,
                        qe: *qe,
                        rb: off,
                        re: off + seed_len,
                    });
                }
            }
        }
    }

    dedup_seeds(&mut seeds);
    seeds
}

/// 过滤被其他区间完全包含的 MEM
fn filter_contained(mems: &mut Vec<(usize, usize, usize, usize)>) {
    if mems.len() <= 1 {
        return;
    }
    // 按长度降序排列
    mems.sort_by(|a, b| {
        let len_a = a.1 - a.0;
        let len_b = b.1 - b.0;
        len_b.cmp(&len_a)
    });

    let mut keep = vec![true; mems.len()];
    for i in 0..mems.len() {
        if !keep[i] {
            continue;
        }
        for j in (i + 1)..mems.len() {
            if !keep[j] {
                continue;
            }
            // 如果 j 被 i 完全包含
            if mems[i].0 <= mems[j].0 && mems[i].1 >= mems[j].1 {
                keep[j] = false;
            }
        }
    }

    let mut idx = 0;
    mems.retain(|_| {
        let k = keep[idx];
        idx += 1;
        k
    });
}

fn dedup_seeds(seeds: &mut Vec<MemSeed>) {
    seeds.sort_by(|a, b| {
        a.contig
            .cmp(&b.contig)
            .then(a.qb.cmp(&b.qb))
            .then(a.qe.cmp(&b.qe))
            .then(a.rb.cmp(&b.rb))
            .then(a.re.cmp(&b.re))
    });
    seeds.dedup();
}

/// 向后兼容的 MEM 种子查找（保留原有接口）
pub fn find_mem_seeds(
    fm: &FMIndex,
    query_alpha: &[u8],
    min_len: usize,
) -> Vec<MemSeed> {
    find_smem_seeds(fm, query_alpha, min_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{bwt, sa};
    use crate::index::fm::{Contig, FMIndex};
    use crate::util::dna;

    fn build_test_fm(seq: &[u8]) -> FMIndex {
        let norm = dna::normalize_seq(seq);
        let mut text: Vec<u8> = Vec::new();
        for &b in &norm {
            text.push(dna::to_alphabet(b));
        }
        let len = text.len() as u32;
        let contigs = vec![Contig {
            name: "chr1".to_string(),
            len,
            offset: 0,
        }];
        text.push(0);
        let sa_arr = sa::build_sa(&text);
        let bwt_arr = bwt::build_bwt(&text, &sa_arr);
        FMIndex::build(text, bwt_arr, sa_arr, contigs, dna::SIGMA as u8, 4)
    }

    #[test]
    fn smem_seeds_basic() {
        let fm = build_test_fm(b"ACGTACGT");
        let read = b"CGTA";
        let norm = dna::normalize_seq(read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_smem_seeds(&fm, &alpha, 2);
        assert!(
            seeds.iter().any(|s| s.contig == 0 && s.qb == 0 && s.qe == 4)
        );
    }

    #[test]
    fn smem_seeds_respect_min_len() {
        let fm = build_test_fm(b"ACGTACGT");
        let read = b"CGTA";
        let norm = dna::normalize_seq(read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_smem_seeds(&fm, &alpha, 5);
        assert!(seeds.is_empty());
    }

    #[test]
    fn smem_finds_longer_match() {
        let fm = build_test_fm(b"ACGTACGTACGTACGTACGTACGTACGT");
        let read = b"ACGTACGTACGT";
        let norm = dna::normalize_seq(read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_smem_seeds(&fm, &alpha, 4);
        assert!(!seeds.is_empty());
        // Should find the full-length match
        assert!(seeds.iter().any(|s| s.qe - s.qb >= 12));
    }
}
