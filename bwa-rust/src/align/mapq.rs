/// BWA 风格的 MAPQ 计算
/// 参考 BWA mem_approx_mapq_se: mapq = MEM_MAPQ_COEF * (1 - sub/best) * ln(best)
/// MEM_MAPQ_COEF = 30, MEM_MAPQ_MAX = 60
pub fn compute_mapq(best_score: i32, second_best_score: i32) -> u8 {
    const MAPQ_COEF: f64 = 30.0;
    const MAPQ_MAX: u8 = 60;

    if best_score <= 0 {
        return 0;
    }

    let best = best_score as f64;

    if second_best_score <= 0 {
        // 唯一比对：q = coef * ln(best)，上限 MAPQ_MAX
        let q = (MAPQ_COEF * best.ln()).round() as i32;
        return (q.clamp(0, MAPQ_MAX as i32)) as u8;
    }

    let sub = second_best_score as f64;
    let ratio = sub / best;
    // q = coef * (1 - sub/best) * ln(best)
    let q = (MAPQ_COEF * (1.0 - ratio) * best.ln()).round() as i32;
    (q.clamp(0, MAPQ_MAX as i32)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapq_model() {
        // 唯一比对：q = 30 * ln(best)，上限 60
        assert!(compute_mapq(50, 0) > 50);
        assert!(compute_mapq(100, 0) == 60);
        // 有次优：q = 30 * (1 - sub/best) * ln(best)
        assert!(compute_mapq(50, 25) > 0);
        // 相同分数 -> 0
        assert_eq!(compute_mapq(10, 10), 0);
        assert_eq!(compute_mapq(100, 100), 0);
        // 无效分数
        assert_eq!(compute_mapq(0, 0), 0);
        assert_eq!(compute_mapq(-5, 0), 0);
        // 唯一比对且分数较高
        assert!(compute_mapq(30, 0) > 30);
    }

    #[test]
    fn mapq_monotonically_decreases_with_better_secondary() {
        // As second best score approaches best, MAPQ should decrease
        let q1 = compute_mapq(100, 0);
        let q2 = compute_mapq(100, 50);
        let q3 = compute_mapq(100, 90);
        assert!(q1 >= q2);
        assert!(q2 >= q3);
    }

    #[test]
    fn mapq_is_zero_for_equal_scores() {
        for score in [1, 10, 50, 100] {
            assert_eq!(compute_mapq(score, score), 0);
        }
    }
}
