/// 构建后缀数组（基于倍增法，O(n log n) 排序）。
/// 输入为数值化的文本（如 0:$,1:A,2:C,3:G,4:T,5:N）。
/// 允许文本中包含多个 0 作为不同 contig 的分隔符。
pub fn build_sa(text: &[u8]) -> Vec<u32> {
    let n = text.len();
    if n == 0 {
        return Vec::new();
    }
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rank: Vec<i32> = text.iter().map(|&b| b as i32).collect();
    let mut tmp: Vec<i32> = vec![0; n];

    let mut k = 1usize;
    while k < n {
        sa.sort_unstable_by(|&i, &j| {
            let r1 = rank[i];
            let r2 = rank[j];
            if r1 != r2 {
                return r1.cmp(&r2);
            }
            let r1n = if i + k < n { rank[i + k] } else { -1 };
            let r2n = if j + k < n { rank[j + k] } else { -1 };
            r1n.cmp(&r2n)
        });

        tmp[sa[0]] = 0;
        for i in 1..n {
            let a = sa[i - 1];
            let b = sa[i];
            let prev = (rank[a], if a + k < n { rank[a + k] } else { -1 });
            let curr = (rank[b], if b + k < n { rank[b + k] } else { -1 });
            tmp[b] = tmp[a] + if curr != prev { 1 } else { 0 };
        }

        // 复制回 rank
        rank.copy_from_slice(&tmp);
        if rank[sa[n - 1]] as usize == n - 1 {
            break;
        }
        k <<= 1;
    }

    sa.into_iter().map(|x| x as u32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn naive_sa(text: &[u8]) -> Vec<u32> {
        let n = text.len();
        let mut suffixes: Vec<(usize, &[u8])> = (0..n).map(|i| (i, &text[i..])).collect();
        suffixes.sort_by(|a, b| a.1.cmp(b.1));
        suffixes.into_iter().map(|(i, _)| i as u32).collect()
    }

    fn make_text(len: usize) -> Vec<u8> {
        let mut x: u32 = 1_234_567;
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let val = (x % 6) as u8;
            v.push(val);
        }
        v
    }

    #[test]
    fn sa_basic() {
        // 文本：A C G T $  -> 1 2 3 4 0
        let text = [1u8, 2, 3, 4, 0];
        let sa = build_sa(&text);
        // 期望：后缀按字典序：$, A$, C$, G$, T$
        assert_eq!(sa, vec![4, 0, 1, 2, 3]);
    }

    #[test]
    fn sa_matches_naive_on_small_random_texts() {
        for len in 1..=20 {
            let text = make_text(len);
            let sa_fast = build_sa(&text);
            let sa_naive = naive_sa(&text);
            assert_eq!(sa_fast, sa_naive, "mismatch on len={}", len);
        }
    }

    #[test]
    fn sa_handles_multiple_separators() {
        // 文本：A C $ G $  -> 1 2 0 3 0
        let text = [1u8, 2, 0, 3, 0];
        let sa = build_sa(&text);
        let expected = naive_sa(&text);
        assert_eq!(sa, expected);
    }
}
