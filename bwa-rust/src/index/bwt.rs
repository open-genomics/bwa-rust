/// 根据后缀数组构建 BWT。
/// text 为数值化字母表（0..SIGMA），sa 为后缀数组位置。
pub fn build_bwt(text: &[u8], sa: &[u32]) -> Vec<u8> {
    let n = text.len();
    if n == 0 {
        return Vec::new();
    }
    let mut bwt = Vec::with_capacity(n);
    for &p in sa {
        let i = p as usize;
        let prev = if i == 0 { text[n - 1] } else { text[i - 1] };
        bwt.push(prev);
    }
    bwt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bwt_single_contig() {
        // 文本：A C G T $  -> 1 2 3 4 0
        let text = [1u8, 2, 3, 4, 0];
        // 对应的 SA：$, A$, C$, G$, T$
        let sa = [4u32, 0, 1, 2, 3];
        let bwt = build_bwt(&text, &sa);
        // BWT：T $ A C G  -> 4 0 1 2 3
        assert_eq!(bwt, vec![4u8, 0, 1, 2, 3]);
    }

    #[test]
    fn bwt_multi_contig_with_separators() {
        // 文本：A C G $ A G $  -> 1 2 3 0 1 3 0
        let text = [1u8, 2, 3, 0, 1, 3, 0];
        // 后缀按字典序对应的 SA = [6, 3, 0, 4, 1, 5, 2]
        let sa = [6u32, 3, 0, 4, 1, 5, 2];
        let bwt = build_bwt(&text, &sa);
        // 对应的 BWT = [G, G, $, $, A, A, C] -> [3, 3, 0, 0, 1, 1, 2]
        assert_eq!(bwt, vec![3u8, 3, 0, 0, 1, 1, 2]);
    }
}
