pub mod candidate;
pub mod chain;
pub mod extend;
pub mod mapq;
pub mod pipeline;
pub mod seed;
pub mod supplementary;
pub mod sw;

pub use candidate::{collect_candidates, dedup_candidates, AlignCandidate, RefSeqCache};
pub use chain::{
    best_chain, build_chains, build_chains_with_limit, filter_chains, Chain, DEFAULT_MAX_CHAINS_PER_CONTIG,
};
pub use extend::{chain_to_alignment, chain_to_alignment_with_buf};
pub use mapq::compute_mapq;
pub use pipeline::{align_fastq_with_fm, align_fastq_with_fm_opt, align_fastq_with_opt};
pub use seed::{find_smem_seeds, find_smem_seeds_with_max_occ, MemSeed, DEFAULT_MAX_OCC};
pub use supplementary::{are_non_overlapping, classify_alignments, generate_sa_tag, AlignmentType};
pub use sw::{banded_sw, semiglobal_align, SwParams, SwResult};

/// Default maximum alignments output per read
pub const DEFAULT_MAX_ALIGNMENTS_PER_READ: usize = 5;

/// Default Z-drop threshold for alignment extension
pub const DEFAULT_ZDROP: i32 = 100;

#[derive(Clone, Copy, Debug)]
pub struct AlignOpt {
    pub match_score: i32,
    pub mismatch_penalty: i32,
    pub gap_open: i32,
    pub gap_extend: i32,
    pub clip_penalty: i32,
    pub band_width: usize,
    pub score_threshold: i32,
    pub min_seed_len: usize,
    pub threads: usize,
    /// Maximum chains to extract per contig (greedy peeling)
    pub max_chains_per_contig: usize,
    /// Maximum alignments to output per read
    pub max_alignments_per_read: usize,
    /// Maximum occurrences for a MEM seed (skip highly repetitive seeds)
    pub max_occ: usize,
    /// Z-drop threshold for alignment extension termination
    pub zdrop: i32,
}

impl Default for AlignOpt {
    fn default() -> Self {
        Self {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            clip_penalty: 1,
            band_width: 16,
            score_threshold: 20,
            min_seed_len: 19,
            threads: 1,
            max_chains_per_contig: DEFAULT_MAX_CHAINS_PER_CONTIG,
            max_alignments_per_read: DEFAULT_MAX_ALIGNMENTS_PER_READ,
            max_occ: DEFAULT_MAX_OCC,
            zdrop: DEFAULT_ZDROP,
        }
    }
}

impl AlignOpt {
    /// Validate alignment options, returning an error if invalid
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.band_width == 0 {
            return Err("band_width must be greater than 0");
        }
        if self.match_score < 0 {
            return Err("match_score must be non-negative");
        }
        if self.mismatch_penalty < 0 {
            return Err("mismatch_penalty must be non-negative");
        }
        if self.gap_open < 0 {
            return Err("gap_open must be non-negative");
        }
        if self.gap_extend < 0 {
            return Err("gap_extend must be non-negative");
        }
        if self.clip_penalty < 0 {
            return Err("clip_penalty must be non-negative");
        }
        if self.threads == 0 {
            return Err("threads must be greater than 0");
        }
        if self.max_chains_per_contig == 0 {
            return Err("max_chains_per_contig must be greater than 0");
        }
        if self.max_alignments_per_read == 0 {
            return Err("max_alignments_per_read must be greater than 0");
        }
        if self.min_seed_len == 0 {
            return Err("min_seed_len must be greater than 0");
        }
        if self.score_threshold <= 0 {
            return Err("score_threshold must be greater than 0");
        }
        if self.zdrop <= 0 {
            return Err("zdrop must be greater than 0");
        }
        if self.max_occ == 0 {
            return Err("max_occ must be greater than 0");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_opt_default_is_valid() {
        let opt = AlignOpt::default();
        assert!(opt.validate().is_ok());
    }

    #[test]
    fn align_opt_rejects_zero_band_width() {
        let opt = AlignOpt {
            band_width: 0,
            ..AlignOpt::default()
        };
        assert!(opt.validate().is_err());
    }

    #[test]
    fn align_opt_rejects_negative_match_score() {
        let opt = AlignOpt {
            match_score: -1,
            ..AlignOpt::default()
        };
        assert!(opt.validate().is_err());
    }

    #[test]
    fn align_opt_rejects_negative_gap_open() {
        let opt = AlignOpt {
            gap_open: -1,
            ..AlignOpt::default()
        };
        assert!(opt.validate().is_err());
    }

    #[test]
    fn align_opt_rejects_zero_threads() {
        let opt = AlignOpt {
            threads: 0,
            ..AlignOpt::default()
        };
        assert!(opt.validate().is_err());
    }

    #[test]
    fn align_opt_rejects_zero_max_chains() {
        let opt = AlignOpt {
            max_chains_per_contig: 0,
            ..AlignOpt::default()
        };
        assert!(opt.validate().is_err());
    }

    #[test]
    fn align_opt_rejects_zero_max_alignments() {
        let opt = AlignOpt {
            max_alignments_per_read: 0,
            ..AlignOpt::default()
        };
        assert!(opt.validate().is_err());
    }

    #[test]
    fn align_opt_rejects_zero_min_seed_len() {
        let opt = AlignOpt {
            min_seed_len: 0,
            ..AlignOpt::default()
        };
        assert!(opt.validate().is_err());
    }

    #[test]
    fn align_opt_rejects_nonpositive_zdrop() {
        let opt = AlignOpt {
            zdrop: 0,
            ..AlignOpt::default()
        };
        assert!(opt.validate().is_err());
    }

    #[test]
    fn align_opt_rejects_nonpositive_score_threshold() {
        let opt = AlignOpt {
            score_threshold: 0,
            ..AlignOpt::default()
        };
        assert!(opt.validate().is_err());
    }

    #[test]
    fn align_opt_rejects_zero_max_occ() {
        let opt = AlignOpt {
            max_occ: 0,
            ..AlignOpt::default()
        };
        assert!(opt.validate().is_err());
    }
}
