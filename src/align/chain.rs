use std::collections::{HashMap, HashSet};

use super::seed::MemSeed;

/// 每个 contig 最多贪心剥离的链数
pub const DEFAULT_MAX_CHAINS_PER_CONTIG: usize = 5;

/// 种子链结构
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain {
    pub contig: usize,
    pub seeds: Vec<MemSeed>,
    pub score: u32,
}

/// 用 DP 方法从种子集合中找到得分最高的单条链。
///
/// 按 `(contig, qb, rb)` 排序后做链式 DP，不允许跨 contig 或 query/ref 上有重叠，
/// gap（query 侧或 ref 侧）超过 `max_gap` 的种子对不能链接。
/// 返回 `None` 当且仅当 `seeds` 为空。
pub fn best_chain(seeds: &[MemSeed], max_gap: usize) -> Option<Chain> {
    if seeds.is_empty() {
        return None;
    }

    let mut idxs: Vec<usize> = (0..seeds.len()).collect();
    idxs.sort_by_key(|&i| {
        let s = &seeds[i];
        (s.contig, s.qb, s.rb)
    });

    let n = idxs.len();
    let mut dp: Vec<u32> = vec![0; n];
    let mut prev: Vec<Option<usize>> = vec![None; n];
    let mut best_i: Option<usize> = None;

    for (t, &i) in idxs.iter().enumerate() {
        let si = &seeds[i];
        let len_i = (si.qe - si.qb) as u32;
        dp[t] = len_i;

        for (u, &j) in idxs[..t].iter().enumerate() {
            let sj = &seeds[j];
            if sj.contig != si.contig {
                continue;
            }
            if sj.qe > si.qb {
                continue;
            }
            if sj.re > si.rb {
                continue;
            }
            let gap_q = si.qb - sj.qe;
            let gap_r = (si.rb - sj.re) as usize;
            if gap_q > max_gap || gap_r > max_gap {
                continue;
            }
            let cand = dp[u] + len_i;
            if cand > dp[t] {
                dp[t] = cand;
                prev[t] = Some(u);
            }
        }

        if best_i.map(|bi| dp[t] > dp[bi]).unwrap_or(true) {
            best_i = Some(t);
        }
    }

    let best_t = best_i?;
    let mut chain_idxs: Vec<usize> = Vec::new();
    let mut cur = Some(best_t);
    while let Some(t) = cur {
        chain_idxs.push(idxs[t]);
        cur = prev[t];
    }
    chain_idxs.reverse();

    let contig = seeds[chain_idxs[0]].contig;
    let seeds_vec: Vec<MemSeed> = chain_idxs.into_iter().map(|i| seeds[i]).collect();
    let score = dp[best_t];

    Some(Chain {
        contig,
        seeds: seeds_vec,
        score,
    })
}

/// 构建所有可能的链（返回多条链，按得分排序）
/// 对种子集合按 contig 分组，每组内贪心剥离出最多 `max_chains_per_contig` 条链，
/// 全部链按得分降序、contig 升序、参考区间和 query 区间确定性排序后返回。
pub fn build_chains(seeds: &[MemSeed], max_gap: usize) -> Vec<Chain> {
    build_chains_with_limit(seeds, max_gap, DEFAULT_MAX_CHAINS_PER_CONTIG)
}

/// 同 [`build_chains`]，但可指定每个 contig 的最大链数。
pub fn build_chains_with_limit(seeds: &[MemSeed], max_gap: usize, max_chains_per_contig: usize) -> Vec<Chain> {
    if seeds.is_empty() {
        return Vec::new();
    }

    // 按 contig 分组
    let mut by_contig: HashMap<usize, Vec<MemSeed>> = HashMap::new();
    for s in seeds {
        by_contig.entry(s.contig).or_default().push(*s);
    }

    let mut contig_groups: Vec<(usize, Vec<MemSeed>)> = by_contig.into_iter().collect();
    contig_groups.sort_by_key(|(contig_id, _)| *contig_id);

    let mut chains = Vec::new();
    for (_contig_id, contig_seeds) in contig_groups {
        // 提取多条链（贪心剥离）
        let mut remaining = contig_seeds;
        for _ in 0..max_chains_per_contig {
            if remaining.is_empty() {
                break;
            }
            if let Some(chain) = best_chain(&remaining, max_gap) {
                // 从 remaining 中移除已用种子
                let used: HashSet<(usize, usize, u32, u32)> =
                    chain.seeds.iter().map(|s| (s.qb, s.qe, s.rb, s.re)).collect();
                remaining.retain(|s| !used.contains(&(s.qb, s.qe, s.rb, s.re)));
                chains.push(chain);
            } else {
                break;
            }
        }
    }

    sort_chains_deterministically(&mut chains);
    chains
}

/// 过滤弱链和冗余链（类似 BWA 的 `mem_chain_flt`）。
///
/// 首先移除得分低于最佳链 `min_score_ratio` 倍的链；
/// 然后在同一 contig 上，若两条链的 query 区间重叠率 > 80% 且 ref 区间重叠率 > 80%，
/// 保留得分更高的链（即先出现的），丢弃另一条。
///
/// # 重叠阈值说明
///
/// `0.8` (80%) 重叠阈值来自 BWA 的经验值：
/// - 两条链如果在 query 和 reference 上都有 >80% 重叠，很可能是同一比对的不同表示
/// - 该阈值平衡了去重效果和保留真实多比对位点的能力
/// - 过低会误删真实的多比对；过高会保留冗余候选
pub fn filter_chains(chains: &mut Vec<Chain>, min_score_ratio: f64) {
    if chains.is_empty() {
        return;
    }

    let best_score = chains.iter().map(|chain| chain.score).max().unwrap_or(0);
    let threshold = (best_score as f64 * min_score_ratio) as u32;

    // 按得分过滤
    chains.retain(|c| c.score >= threshold);

    let ranges: Vec<ChainRanges> = chains.iter().map(ChainRanges::from_chain).collect();

    // 仅在同一 contig 且参考区间也高度重叠时，才视为冗余链
    // OVERLAP_THRESHOLD = 0.8: 两条链在 query 和 ref 上重叠都超过 80% 视为冗余
    const OVERLAP_THRESHOLD: f64 = 0.8;
    let mut keep = vec![true; chains.len()];
    for i in 0..chains.len() {
        if !keep[i] {
            continue;
        }
        let ci = &chains[i];
        let ri = &ranges[i];

        for j in (i + 1)..chains.len() {
            if !keep[j] {
                continue;
            }
            let cj = &chains[j];
            if ci.contig != cj.contig {
                continue;
            }
            let rj = &ranges[j];

            if overlap_ratio(ri.qb as u64, ri.qe as u64, rj.qb as u64, rj.qe as u64) > OVERLAP_THRESHOLD
                && overlap_ratio(ri.rb as u64, ri.re as u64, rj.rb as u64, rj.re as u64) > OVERLAP_THRESHOLD
            {
                keep[j] = false;
            }
        }
    }

    let mut idx = 0;
    chains.retain(|_| {
        let k = keep[idx];
        idx += 1;
        k
    });
}

fn chain_query_range(chain: &Chain) -> (usize, usize) {
    let min = chain.seeds.iter().map(|s| s.qb).min().unwrap_or(0);
    let max = chain.seeds.iter().map(|s| s.qe).max().unwrap_or(0);
    (min, max)
}

fn chain_ref_range(chain: &Chain) -> (u32, u32) {
    let min = chain.seeds.iter().map(|s| s.rb).min().unwrap_or(0);
    let max = chain.seeds.iter().map(|s| s.re).max().unwrap_or(0);
    (min, max)
}

#[derive(Clone, Copy)]
struct ChainRanges {
    qb: usize,
    qe: usize,
    rb: u32,
    re: u32,
}

impl ChainRanges {
    fn from_chain(chain: &Chain) -> Self {
        let (qb, qe) = chain_query_range(chain);
        let (rb, re) = chain_ref_range(chain);
        Self { qb, qe, rb, re }
    }
}

fn sort_chains_deterministically(chains: &mut Vec<Chain>) {
    let mut decorated: Vec<(ChainRanges, Chain)> =
        chains.drain(..).map(|chain| (ChainRanges::from_chain(&chain), chain)).collect();
    decorated.sort_by(|(ra, a), (rb, b)| {
        b.score
            .cmp(&a.score)
            .then(a.contig.cmp(&b.contig))
            .then((ra.rb, ra.re).cmp(&(rb.rb, rb.re)))
            .then((ra.qb, ra.qe).cmp(&(rb.qb, rb.qe)))
    });
    chains.extend(decorated.into_iter().map(|(_, chain)| chain));
}

fn overlap_ratio(a_start: u64, a_end: u64, b_start: u64, b_end: u64) -> f64 {
    let overlap_start = a_start.max(b_start);
    let overlap_end = a_end.min(b_end);
    if overlap_end <= overlap_start {
        return 0.0;
    }
    let overlap_len = overlap_end - overlap_start;
    let shorter_len = (a_end - a_start).min(b_end - b_start);
    if shorter_len == 0 {
        0.0
    } else {
        overlap_len as f64 / shorter_len as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn best_chain_simple_diagonal() {
        let seeds = vec![
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 0,
                re: 4,
            },
            MemSeed {
                contig: 0,
                qb: 4,
                qe: 8,
                rb: 4,
                re: 8,
            },
        ];
        let chain = best_chain(&seeds, 10).expect("chain");
        assert_eq!(chain.contig, 0);
        assert_eq!(chain.seeds.len(), 2);
        assert_eq!(chain.score, 8);
    }

    #[test]
    fn best_chain_avoids_overlapping_and_far_gaps() {
        let seeds = vec![
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 0,
                re: 4,
            },
            MemSeed {
                contig: 0,
                qb: 3,
                qe: 6,
                rb: 3,
                re: 6,
            },
            MemSeed {
                contig: 0,
                qb: 20,
                qe: 24,
                rb: 20,
                re: 24,
            },
            MemSeed {
                contig: 0,
                qb: 4,
                qe: 8,
                rb: 4,
                re: 8,
            },
        ];
        let chain = best_chain(&seeds, 10).expect("chain");
        assert_eq!(chain.seeds.len(), 2);
        assert_eq!(chain.seeds[0].qb, 0);
        assert_eq!(chain.seeds[1].qb, 4);
        assert_eq!(chain.score, 8);
    }

    #[test]
    fn build_chains_multi() {
        let seeds = vec![
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 0,
                re: 4,
            },
            MemSeed {
                contig: 0,
                qb: 4,
                qe: 8,
                rb: 4,
                re: 8,
            },
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 100,
                re: 104,
            },
            MemSeed {
                contig: 0,
                qb: 4,
                qe: 8,
                rb: 104,
                re: 108,
            },
        ];
        let chains = build_chains(&seeds, 10);
        assert!(chains.len() >= 2);
    }

    #[test]
    fn filter_chains_removes_weak() {
        let mut chains = vec![
            Chain {
                contig: 0,
                seeds: vec![MemSeed {
                    contig: 0,
                    qb: 0,
                    qe: 20,
                    rb: 0,
                    re: 20,
                }],
                score: 20,
            },
            Chain {
                contig: 0,
                seeds: vec![MemSeed {
                    contig: 0,
                    qb: 0,
                    qe: 3,
                    rb: 100,
                    re: 103,
                }],
                score: 3,
            },
        ];
        filter_chains(&mut chains, 0.5);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].score, 20);
    }

    #[test]
    fn best_chain_empty_seeds() {
        assert!(best_chain(&[], 10).is_none());
    }

    #[test]
    fn best_chain_single_seed() {
        let seeds = vec![MemSeed {
            contig: 0,
            qb: 5,
            qe: 10,
            rb: 100,
            re: 105,
        }];
        let chain = best_chain(&seeds, 10).unwrap();
        assert_eq!(chain.seeds.len(), 1);
        assert_eq!(chain.score, 5);
        assert_eq!(chain.contig, 0);
    }

    #[test]
    fn best_chain_three_collinear_seeds() {
        let seeds = vec![
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 5,
                rb: 0,
                re: 5,
            },
            MemSeed {
                contig: 0,
                qb: 5,
                qe: 10,
                rb: 5,
                re: 10,
            },
            MemSeed {
                contig: 0,
                qb: 10,
                qe: 15,
                rb: 10,
                re: 15,
            },
        ];
        let chain = best_chain(&seeds, 10).unwrap();
        assert_eq!(chain.seeds.len(), 3);
        assert_eq!(chain.score, 15);
    }

    #[test]
    fn best_chain_different_contigs() {
        let seeds = vec![
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 5,
                rb: 0,
                re: 5,
            },
            MemSeed {
                contig: 1,
                qb: 5,
                qe: 10,
                rb: 5,
                re: 10,
            },
        ];
        let chain = best_chain(&seeds, 10).unwrap();
        // Seeds on different contigs cannot chain together
        assert_eq!(chain.seeds.len(), 1);
        assert_eq!(chain.score, 5);
    }

    #[test]
    fn build_chains_empty() {
        let chains = build_chains(&[], 10);
        assert!(chains.is_empty());
    }

    #[test]
    fn build_chains_sorted_by_score() {
        let seeds = vec![
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 10,
                rb: 0,
                re: 10,
            },
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 3,
                rb: 100,
                re: 103,
            },
        ];
        let chains = build_chains(&seeds, 10);
        assert!(!chains.is_empty());
        for i in 1..chains.len() {
            assert!(chains[i].score <= chains[i - 1].score);
        }
    }

    #[test]
    fn filter_chains_empty() {
        let mut chains = Vec::new();
        filter_chains(&mut chains, 0.5);
        assert!(chains.is_empty());
    }

    #[test]
    fn filter_chains_keeps_non_overlapping() {
        let mut chains = vec![
            Chain {
                contig: 0,
                seeds: vec![MemSeed {
                    contig: 0,
                    qb: 0,
                    qe: 10,
                    rb: 0,
                    re: 10,
                }],
                score: 10,
            },
            Chain {
                contig: 0,
                seeds: vec![MemSeed {
                    contig: 0,
                    qb: 20,
                    qe: 30,
                    rb: 20,
                    re: 30,
                }],
                score: 10,
            },
        ];
        filter_chains(&mut chains, 0.5);
        assert_eq!(chains.len(), 2);
    }

    #[test]
    fn filter_chains_keeps_overlapping_multimappers_on_same_contig() {
        let mut chains = vec![
            Chain {
                contig: 0,
                seeds: vec![MemSeed {
                    contig: 0,
                    qb: 0,
                    qe: 12,
                    rb: 10,
                    re: 22,
                }],
                score: 12,
            },
            Chain {
                contig: 0,
                seeds: vec![MemSeed {
                    contig: 0,
                    qb: 0,
                    qe: 12,
                    rb: 110,
                    re: 122,
                }],
                score: 12,
            },
        ];
        filter_chains(&mut chains, 0.5);
        assert_eq!(chains.len(), 2);
    }

    #[test]
    fn build_chains_uses_deterministic_tie_break_order() {
        let seeds = vec![
            MemSeed {
                contig: 1,
                qb: 0,
                qe: 4,
                rb: 100,
                re: 104,
            },
            MemSeed {
                contig: 1,
                qb: 4,
                qe: 8,
                rb: 104,
                re: 108,
            },
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 0,
                re: 4,
            },
            MemSeed {
                contig: 0,
                qb: 4,
                qe: 8,
                rb: 4,
                re: 8,
            },
        ];
        let chains = build_chains(&seeds, 10);
        assert!(chains.len() >= 2);
        assert_eq!(chains[0].contig, 0);
        assert_eq!(chains[1].contig, 1);
    }

    #[test]
    fn best_chain_gap_too_large() {
        let seeds = vec![
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 5,
                rb: 0,
                re: 5,
            },
            MemSeed {
                contig: 0,
                qb: 100,
                qe: 105,
                rb: 100,
                re: 105,
            },
        ];
        // max_gap = 10, gap between seeds = 95
        let chain = best_chain(&seeds, 10).unwrap();
        assert_eq!(chain.seeds.len(), 1); // can't chain across large gap
    }

    #[test]
    fn build_chains_with_limit_respects_limit() {
        let seeds = vec![
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 0,
                re: 4,
            },
            MemSeed {
                contig: 0,
                qb: 4,
                qe: 8,
                rb: 4,
                re: 8,
            },
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 100,
                re: 104,
            },
            MemSeed {
                contig: 0,
                qb: 4,
                qe: 8,
                rb: 104,
                re: 108,
            },
            MemSeed {
                contig: 0,
                qb: 0,
                qe: 4,
                rb: 200,
                re: 204,
            },
            MemSeed {
                contig: 0,
                qb: 4,
                qe: 8,
                rb: 204,
                re: 208,
            },
        ];
        // With limit 1, only one chain per contig
        let chains = build_chains_with_limit(&seeds, 10, 1);
        assert_eq!(chains.len(), 1);

        // With limit 2, up to two chains
        let chains = build_chains_with_limit(&seeds, 10, 2);
        assert!(chains.len() <= 2);

        // With default limit, should get more
        let chains_default = build_chains(&seeds, 10);
        assert!(chains_default.len() >= chains.len());
    }
}
