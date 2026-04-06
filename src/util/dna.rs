/// 字母表大小：`{0:$, 1:A, 2:C, 3:G, 4:T/U, 5:N}`
pub const SIGMA: usize = 6;

/// 将 ASCII 碱基字节编码为内部字母表索引（0–5）。
///
/// - `0`（sentinel `$`）→ 0
/// - `A`/`a` → 1，`C`/`c` → 2，`G`/`g` → 3，`T`/`t`/`U`/`u` → 4，`N`/`n` → 5
/// - 其余未知字符映射到 5（等同 `N`）
#[inline]
#[must_use]
pub fn to_alphabet(b: u8) -> u8 {
    if b == 0 {
        return 0;
    }
    match b.to_ascii_uppercase() {
        b'A' => 1,
        b'C' => 2,
        b'G' => 3,
        b'T' | b'U' => 4,
        b'N' => 5,
        _ => 5, // map others to N
    }
}

/// 将内部字母表索引解码回大写 ASCII 碱基字节。
///
/// 0 → 0（sentinel），1 → `A`，2 → `C`，3 → `G`，4 → `T`，5/其他 → `N`
#[inline]
#[must_use]
pub fn from_alphabet(a: u8) -> u8 {
    match a {
        0 => 0,
        1 => b'A',
        2 => b'C',
        3 => b'G',
        4 => b'T',
        5 => b'N',
        _ => b'N',
    }
}

/// 将原始碱基序列归一化为大写 `{A, C, G, T, N}`。
///
/// - 小写转大写
/// - `U`/`u` → `T`
/// - 其余未知字符 → `N`
#[must_use]
pub fn normalize_seq(seq: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(seq.len());
    for &b in seq {
        let up = b.to_ascii_uppercase();
        let nb = match up {
            b'A' | b'C' | b'G' | b'T' | b'N' => up,
            b'U' => b'T',
            _ => b'N',
        };
        out.push(nb);
    }
    out
}

/// 返回单个碱基的互补碱基（大小写均支持）。未知字符返回 `N`。
#[inline]
#[must_use]
pub fn complement(base: u8) -> u8 {
    match base.to_ascii_uppercase() {
        b'A' => b'T',
        b'C' => b'G',
        b'G' => b'C',
        b'T' | b'U' => b'A',
        _ => b'N',
    }
}

/// 返回序列的反向互补（reverse complement）。长度不变。
#[must_use]
pub fn revcomp(seq: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(seq.len());
    for &b in seq.iter().rev() {
        out.push(complement(b));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_seq_basic() {
        let input = b"acgtuXnN";
        let out = normalize_seq(input);
        assert_eq!(out, b"ACGTTNNN");
    }

    #[test]
    fn to_from_alphabet_roundtrip() {
        assert_eq!(to_alphabet(0), 0);
        assert_eq!(to_alphabet(b'A'), 1);
        assert_eq!(to_alphabet(b'a'), 1);
        assert_eq!(to_alphabet(b'C'), 2);
        assert_eq!(to_alphabet(b'c'), 2);
        assert_eq!(to_alphabet(b'G'), 3);
        assert_eq!(to_alphabet(b'T'), 4);
        assert_eq!(to_alphabet(b'U'), 4);
        assert_eq!(to_alphabet(b'N'), 5);
        assert_eq!(to_alphabet(b'x'), 5);

        assert_eq!(from_alphabet(0), 0);
        assert_eq!(from_alphabet(1), b'A');
        assert_eq!(from_alphabet(2), b'C');
        assert_eq!(from_alphabet(3), b'G');
        assert_eq!(from_alphabet(4), b'T');
        assert_eq!(from_alphabet(5), b'N');
        assert_eq!(from_alphabet(100), b'N');
    }

    #[test]
    fn complement_and_revcomp() {
        assert_eq!(complement(b'A'), b'T');
        assert_eq!(complement(b'a'), b'T');
        assert_eq!(complement(b'C'), b'G');
        assert_eq!(complement(b'G'), b'C');
        assert_eq!(complement(b'T'), b'A');
        assert_eq!(complement(b'U'), b'A');
        assert_eq!(complement(b'N'), b'N');
        assert_eq!(complement(b'x'), b'N');

        let seq = b"ACGTN";
        let rc = revcomp(seq);
        assert_eq!(rc, b"NACGT");
        let back = revcomp(&rc);
        assert_eq!(back, seq);
    }

    #[test]
    fn revcomp_roundtrip_various() {
        let seqs: &[&[u8]] = &[b"A", b"AAAA", b"ACGTACGT", b"NNNN", b"TGCA", b"ACGTNNNNACGT"];
        for &s in seqs {
            let norm = normalize_seq(s);
            let rc = revcomp(&norm);
            let back = revcomp(&rc);
            assert_eq!(back, norm, "revcomp roundtrip failed for {:?}", std::str::from_utf8(s));
        }
    }

    #[test]
    fn normalize_seq_maps_unknown_to_n() {
        let input = b"AcRYSWKMBDHV.";
        let out = normalize_seq(input);
        // A, c->C, rest are non-ACGTN -> N
        assert_eq!(out[0], b'A');
        assert_eq!(out[1], b'C');
        for &b in &out[2..] {
            assert_eq!(b, b'N');
        }
    }

    #[test]
    fn to_from_alphabet_complete_mapping() {
        // Verify the full mapping table
        assert_eq!(to_alphabet(b'$'), 5); // unknown -> N
        assert_eq!(from_alphabet(0), 0); // sentinel
        for code in 0..=5u8 {
            let base = from_alphabet(code);
            if code == 0 {
                assert_eq!(base, 0);
            } else {
                let back = to_alphabet(base);
                assert_eq!(back, code, "roundtrip failed for code={}", code);
            }
        }
    }
}
