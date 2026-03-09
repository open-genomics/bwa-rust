use crate::index::fm::FMIndex;

/// 对齐区域结构，类似 BWA 的 mem_alnreg_t。
///
/// 当前版本（v0.1.0）的 pipeline 使用 `candidate::AlignCandidate` 作为内部候选表示。
/// `AlnReg` 保留为公开 API 类型，供未来配对端（PE）对齐和库模式调用使用。
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

/// SMEM 搜索：对 read 的每个位置，找到包含该位置的最长精确匹配（MEM）。
/// 使用增量式左扩展（incremental left-extension）：固定右端点，逐字符向左扩展 SA 区间，
/// 每步仅需一次 `rank_range` 调用（O(1)），相比逐长度重新 backward_search（O(L)）显著更快。
/// 之后过滤被包含的种子，仅保留超级最大精确匹配（SMEM）。
pub fn find_smem_seeds(fm: &FMIndex, query_alpha: &[u8], min_len: usize) -> Vec<MemSeed> {
    let n = query_alpha.len();
    if min_len == 0 || n == 0 || min_len > n {
        return Vec::new();
    }

    let bwt_len = fm.bwt.len();
    let mut raw_mems: Vec<(usize, usize, usize, usize)> = Vec::new(); // (qb, qe, sa_l, sa_r)

    // 第一步：对每个右端点 qe，通过增量左扩展找到最长精确匹配。
    // 从单字符 query[qe-1] 开始，逐步向左调用 rank_range 扩展 SA 区间，
    // 直到区间为空或到达 query 左端。
    for qe in 1..=n {
        let (mut l, mut r) = fm.rank_range(query_alpha[qe - 1], 0, bwt_len);
        if l >= r {
            continue;
        }

        let mut best_qb = qe - 1;
        let mut best_l = l;
        let mut best_r = r;

        // 增量左扩展：每步 O(1)
        for qb in (0..qe.saturating_sub(1)).rev() {
            let (nl, nr) = fm.rank_range(query_alpha[qb], l, r);
            if nl >= nr {
                break;
            }
            l = nl;
            r = nr;
            best_qb = qb;
            best_l = l;
            best_r = r;
        }

        let match_len = qe - best_qb;
        if match_len >= min_len {
            raw_mems.push((best_qb, qe, best_l, best_r));
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
                let contig_len = fm.contigs[ci].len;
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
pub fn find_mem_seeds(fm: &FMIndex, query_alpha: &[u8], min_len: usize) -> Vec<MemSeed> {
    find_smem_seeds(fm, query_alpha, min_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::build_test_fm;
    use crate::util::dna;

    #[test]
    fn smem_seeds_basic() {
        let fm = build_test_fm(b"ACGTACGT");
        let read = b"CGTA";
        let norm = dna::normalize_seq(read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_smem_seeds(&fm, &alpha, 2);
        assert!(seeds.iter().any(|s| s.contig == 0 && s.qb == 0 && s.qe == 4));
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

    #[test]
    fn smem_empty_query() {
        let fm = build_test_fm(b"ACGTACGT");
        let seeds = find_smem_seeds(&fm, &[], 2);
        assert!(seeds.is_empty());
    }

    #[test]
    fn smem_min_len_zero() {
        let fm = build_test_fm(b"ACGTACGT");
        let alpha: Vec<u8> = b"ACGT".iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_smem_seeds(&fm, &alpha, 0);
        assert!(seeds.is_empty());
    }

    #[test]
    fn smem_min_len_exceeds_query() {
        let fm = build_test_fm(b"ACGTACGT");
        let alpha: Vec<u8> = b"AC".iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_smem_seeds(&fm, &alpha, 5);
        assert!(seeds.is_empty());
    }

    #[test]
    fn smem_no_match_in_reference() {
        let fm = build_test_fm(b"AAAAAAAAAAAAAAAA");
        let alpha: Vec<u8> = b"CCCC".iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_smem_seeds(&fm, &alpha, 2);
        assert!(seeds.is_empty());
    }

    #[test]
    fn smem_seeds_have_valid_coordinates() {
        let fm = build_test_fm(b"ACGTACGTACGTACGTACGT");
        let read = b"CGTACGT";
        let norm = dna::normalize_seq(read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_smem_seeds(&fm, &alpha, 3);
        for s in &seeds {
            assert!(s.qe > s.qb, "seed query end <= begin");
            assert!(s.re > s.rb, "seed ref end <= begin");
            assert_eq!(s.qe - s.qb, (s.re - s.rb) as usize, "seed length mismatch");
            assert!(s.re <= fm.contigs[s.contig].len, "seed exceeds contig");
        }
    }

    #[test]
    fn smem_dedup_removes_exact_duplicates() {
        let fm = build_test_fm(b"ACGTACGTACGT");
        let read = b"ACGT";
        let norm = dna::normalize_seq(read);
        let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds = find_smem_seeds(&fm, &alpha, 2);
        // Check no duplicates
        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                assert!(seeds[i] != seeds[j], "duplicate seed found at {} and {}", i, j);
            }
        }
    }

    #[test]
    fn find_mem_seeds_same_as_smem() {
        let fm = build_test_fm(b"ACGTACGTACGT");
        let alpha: Vec<u8> = b"CGTA".iter().map(|&b| dna::to_alphabet(b)).collect();
        let seeds1 = find_smem_seeds(&fm, &alpha, 2);
        let seeds2 = find_mem_seeds(&fm, &alpha, 2);
        assert_eq!(seeds1, seeds2);
    }
}
