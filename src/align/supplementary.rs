//! Supplementary alignment detection (SA:Z tag generation).
//!
//! BWA-MEM detects chimeric reads by finding multiple high-scoring chains that don't overlap
//! on the query. Non-overlapping alignments are reported as supplementary alignments with
//! the SA:Z tag.

use super::AlignCandidate;

/// Check if two alignments are non-overlapping on the query.
///
/// Two alignments are non-overlapping if their query ranges don't intersect.
pub fn are_non_overlapping(a: &AlignCandidate, b: &AlignCandidate) -> bool {
    a.query_end <= b.query_start || b.query_end <= a.query_start
}

/// Classification of alignment type for SA:Z tag generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentType {
    /// Primary alignment (best scoring, non-overlapping)
    Primary,
    /// Secondary alignment (overlapping alternative, lower score)
    Secondary,
    /// Supplementary alignment (non-overlapping chimeric)
    Supplementary,
}

/// Classify alignments into primary, secondary, and supplementary.
///
/// The algorithm:
/// 1. The first alignment (highest score) is always primary
/// 2. Subsequent alignments that don't overlap with primary are supplementary
/// 3. Subsequent alignments that overlap with primary are secondary
///
/// Returns a vector of (index, alignment_type) pairs.
pub fn classify_alignments(candidates: &[AlignCandidate]) -> Vec<(usize, AlignmentType)> {
    if candidates.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(candidates.len());

    // First alignment is always primary
    result.push((0, AlignmentType::Primary));

    if candidates.len() == 1 {
        return result;
    }

    // Track the query ranges covered by primary and supplementary alignments
    let mut covered_ranges: Vec<(usize, usize)> = vec![(candidates[0].query_start, candidates[0].query_end)];

    for (idx, cand) in candidates.iter().enumerate().skip(1) {
        // Check if this alignment overlaps with any primary/supplementary alignment
        let overlaps = covered_ranges
            .iter()
            .any(|(start, end)| !(cand.query_end <= *start || cand.query_start >= *end));

        if overlaps {
            result.push((idx, AlignmentType::Secondary));
        } else {
            result.push((idx, AlignmentType::Supplementary));
            covered_ranges.push((cand.query_start, cand.query_end));
        }
    }

    result
}

/// Generate SA:Z tag content for an alignment.
///
/// The SA:Z tag format is: "rname,pos,strand,CIGAR,mapQ,NM;"
/// Each entry ends with a semicolon.
///
/// For a primary alignment, SA:Z lists all supplementaries.
/// For a supplementary alignment, SA:Z lists the primary + all other supplementaries.
pub fn generate_sa_tag(
    current_idx: usize,
    candidates: &[AlignCandidate],
    classification: &[(usize, AlignmentType)],
) -> String {
    let mut entries: Vec<String> = Vec::new();

    for &(idx, align_type) in classification {
        // Skip the current alignment and secondary alignments
        if idx == current_idx || align_type == AlignmentType::Secondary {
            continue;
        }

        let cand = &candidates[idx];
        let strand = if cand.is_rev { '-' } else { '+' };
        // MAPQ is computed elsewhere, use 0 as placeholder for now
        // In a full implementation, we'd need to pass the MAPQ values
        let mapq = 0;

        entries.push(format!(
            "{},{},{},{},{},{};",
            cand.rname, cand.pos1, strand, cand.cigar, mapq, cand.nm
        ));
    }

    entries.concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(
        score: i32,
        query_start: usize,
        query_end: usize,
        is_rev: bool,
        rname: &str,
        pos1: u32,
    ) -> AlignCandidate {
        AlignCandidate {
            score,
            sort_score: score,
            is_rev,
            rname: rname.to_string(),
            pos1,
            cigar: "20M".to_string(),
            nm: 0,
            contig_idx: 0,
            ref_seq: Vec::new(),
            query_seq: Vec::new(),
            query_start,
            query_end,
        }
    }

    #[test]
    fn test_are_non_overlapping_disjoint() {
        let a = make_candidate(50, 0, 20, false, "chr1", 100);
        let b = make_candidate(40, 25, 45, false, "chr1", 200);
        assert!(are_non_overlapping(&a, &b));
        assert!(are_non_overlapping(&b, &a));
    }

    #[test]
    fn test_are_non_overlapping_adjacent() {
        // Adjacent ranges (end == start) are non-overlapping
        let a = make_candidate(50, 0, 20, false, "chr1", 100);
        let b = make_candidate(40, 20, 40, false, "chr1", 200);
        assert!(are_non_overlapping(&a, &b));
    }

    #[test]
    fn test_are_non_overlapping_overlapping() {
        let a = make_candidate(50, 0, 25, false, "chr1", 100);
        let b = make_candidate(40, 20, 40, false, "chr1", 200);
        assert!(!are_non_overlapping(&a, &b));
    }

    #[test]
    fn test_are_non_overlapping_contained() {
        let a = make_candidate(50, 0, 50, false, "chr1", 100);
        let b = make_candidate(40, 10, 30, false, "chr1", 200);
        assert!(!are_non_overlapping(&a, &b));
    }

    #[test]
    fn test_classify_single_alignment() {
        let candidates = vec![make_candidate(50, 0, 20, false, "chr1", 100)];
        let classification = classify_alignments(&candidates);
        assert_eq!(classification, vec![(0, AlignmentType::Primary)]);
    }

    #[test]
    fn test_classify_two_non_overlapping() {
        let candidates = vec![
            make_candidate(50, 0, 20, false, "chr1", 100),
            make_candidate(45, 25, 45, false, "chr1", 200),
        ];
        let classification = classify_alignments(&candidates);
        assert_eq!(
            classification,
            vec![(0, AlignmentType::Primary), (1, AlignmentType::Supplementary),]
        );
    }

    #[test]
    fn test_classify_two_overlapping() {
        let candidates = vec![
            make_candidate(50, 0, 25, false, "chr1", 100),
            make_candidate(45, 20, 40, false, "chr1", 200),
        ];
        let classification = classify_alignments(&candidates);
        assert_eq!(
            classification,
            vec![(0, AlignmentType::Primary), (1, AlignmentType::Secondary),]
        );
    }

    #[test]
    fn test_classify_mixed() {
        // Primary at 0-20, Secondary at 15-35 (overlaps), Supplementary at 40-60 (non-overlapping)
        let candidates = vec![
            make_candidate(50, 0, 20, false, "chr1", 100),
            make_candidate(40, 15, 35, false, "chr1", 200),
            make_candidate(35, 40, 60, false, "chr1", 300),
        ];
        let classification = classify_alignments(&candidates);
        assert_eq!(
            classification,
            vec![
                (0, AlignmentType::Primary),
                (1, AlignmentType::Secondary),
                (2, AlignmentType::Supplementary),
            ]
        );
    }

    #[test]
    fn test_classify_multiple_supplementaries() {
        // Primary at 0-20, Supp1 at 30-50, Supp2 at 60-80
        let candidates = vec![
            make_candidate(50, 0, 20, false, "chr1", 100),
            make_candidate(45, 30, 50, false, "chr1", 200),
            make_candidate(40, 60, 80, false, "chr1", 300),
        ];
        let classification = classify_alignments(&candidates);
        assert_eq!(
            classification,
            vec![
                (0, AlignmentType::Primary),
                (1, AlignmentType::Supplementary),
                (2, AlignmentType::Supplementary),
            ]
        );
    }

    #[test]
    fn test_generate_sa_tag_primary() {
        let candidates = vec![
            make_candidate(50, 0, 20, false, "chr1", 100),
            make_candidate(45, 30, 50, true, "chr2", 200),
        ];
        let classification = vec![(0, AlignmentType::Primary), (1, AlignmentType::Supplementary)];

        // SA:Z for primary should list the supplementary
        let sa = generate_sa_tag(0, &candidates, &classification);
        assert!(sa.contains("chr2"));
        assert!(sa.contains(','));
        assert!(sa.ends_with(';'));
    }

    #[test]
    fn test_generate_sa_tag_supplementary() {
        let candidates = vec![
            make_candidate(50, 0, 20, false, "chr1", 100),
            make_candidate(45, 30, 50, true, "chr2", 200),
        ];
        let classification = vec![(0, AlignmentType::Primary), (1, AlignmentType::Supplementary)];

        // SA:Z for supplementary should list the primary
        let sa = generate_sa_tag(1, &candidates, &classification);
        assert!(sa.contains("chr1"));
        assert!(sa.ends_with(';'));
    }

    #[test]
    fn test_generate_sa_tag_empty() {
        let candidates = vec![make_candidate(50, 0, 20, false, "chr1", 100)];
        let classification = vec![(0, AlignmentType::Primary)];

        // SA:Z for single alignment should be empty
        let sa = generate_sa_tag(0, &candidates, &classification);
        assert!(sa.is_empty());
    }
}
