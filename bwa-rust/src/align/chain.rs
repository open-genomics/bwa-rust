use super::seed::MemSeed;

/// 种子链结构
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain {
    pub contig: usize,
    pub seeds: Vec<MemSeed>,
    pub score: u32,
}

/// 从种子集合中构建最佳链（DP 方法）
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

        if best_i
            .map(|bi| dp[t] > dp[bi])
            .unwrap_or(true)
        {
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
    let seeds_vec: Vec<MemSeed> = chain_idxs.into_iter().map(|i| seeds[i].clone()).collect();
    let score = dp[best_t];

    Some(Chain {
        contig,
        seeds: seeds_vec,
        score,
    })
}

/// 构建所有可能的链（返回多条链，按得分排序）
pub fn build_chains(seeds: &[MemSeed], max_gap: usize) -> Vec<Chain> {
    if seeds.is_empty() {
        return Vec::new();
    }

    // 按 contig 分组
    let mut by_contig: std::collections::HashMap<usize, Vec<MemSeed>> =
        std::collections::HashMap::new();
    for s in seeds {
        by_contig
            .entry(s.contig)
            .or_default()
            .push(s.clone());
    }

    let mut chains = Vec::new();
    for (_, contig_seeds) in &by_contig {
        // 提取多条链（贪心剥离）
        let mut remaining = contig_seeds.clone();
        for _ in 0..5 {
            if remaining.is_empty() {
                break;
            }
            if let Some(chain) = best_chain(&remaining, max_gap) {
                // 从 remaining 中移除已用种子
                let used: std::collections::HashSet<(usize, usize, u32, u32)> = chain
                    .seeds
                    .iter()
                    .map(|s| (s.qb, s.qe, s.rb, s.re))
                    .collect();
                remaining.retain(|s| !used.contains(&(s.qb, s.qe, s.rb, s.re)));
                chains.push(chain);
            } else {
                break;
            }
        }
    }

    chains.sort_by(|a, b| b.score.cmp(&a.score));
    chains
}

/// 链过滤：去除弱链和冗余链
/// 类似 BWA 的 mem_chain_flt
pub fn filter_chains(chains: &mut Vec<Chain>, min_score_ratio: f64) {
    if chains.is_empty() {
        return;
    }

    let best_score = chains[0].score;
    let threshold = (best_score as f64 * min_score_ratio) as u32;

    // 按得分过滤
    chains.retain(|c| c.score >= threshold);

    // 去除 read 覆盖高度重叠的链
    let mut keep = vec![true; chains.len()];
    for i in 0..chains.len() {
        if !keep[i] {
            continue;
        }
        let ci = &chains[i];
        let (qi_min, qi_max) = chain_query_range(ci);

        for j in (i + 1)..chains.len() {
            if !keep[j] {
                continue;
            }
            let cj = &chains[j];
            let (qj_min, qj_max) = chain_query_range(cj);

            // 计算 read 坐标上的重叠
            let overlap_start = qi_min.max(qj_min);
            let overlap_end = qi_max.min(qj_max);
            if overlap_end > overlap_start {
                let overlap_len = overlap_end - overlap_start;
                let shorter_len = (qi_max - qi_min).min(qj_max - qj_min);
                if shorter_len > 0 && overlap_len as f64 / shorter_len as f64 > 0.8 {
                    keep[j] = false;
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn best_chain_simple_diagonal() {
        let seeds = vec![
            MemSeed { contig: 0, qb: 0, qe: 4, rb: 0, re: 4 },
            MemSeed { contig: 0, qb: 4, qe: 8, rb: 4, re: 8 },
        ];
        let chain = best_chain(&seeds, 10).expect("chain");
        assert_eq!(chain.contig, 0);
        assert_eq!(chain.seeds.len(), 2);
        assert_eq!(chain.score, 8);
    }

    #[test]
    fn best_chain_avoids_overlapping_and_far_gaps() {
        let seeds = vec![
            MemSeed { contig: 0, qb: 0, qe: 4, rb: 0, re: 4 },
            MemSeed { contig: 0, qb: 3, qe: 6, rb: 3, re: 6 },
            MemSeed { contig: 0, qb: 20, qe: 24, rb: 20, re: 24 },
            MemSeed { contig: 0, qb: 4, qe: 8, rb: 4, re: 8 },
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
            MemSeed { contig: 0, qb: 0, qe: 4, rb: 0, re: 4 },
            MemSeed { contig: 0, qb: 4, qe: 8, rb: 4, re: 8 },
            MemSeed { contig: 0, qb: 0, qe: 4, rb: 100, re: 104 },
            MemSeed { contig: 0, qb: 4, qe: 8, rb: 104, re: 108 },
        ];
        let chains = build_chains(&seeds, 10);
        assert!(chains.len() >= 2);
    }

    #[test]
    fn filter_chains_removes_weak() {
        let mut chains = vec![
            Chain { contig: 0, seeds: vec![MemSeed { contig: 0, qb: 0, qe: 20, rb: 0, re: 20 }], score: 20 },
            Chain { contig: 0, seeds: vec![MemSeed { contig: 0, qb: 0, qe: 3, rb: 100, re: 103 }], score: 3 },
        ];
        filter_chains(&mut chains, 0.5);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].score, 20);
    }
}
