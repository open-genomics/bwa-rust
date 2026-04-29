pub mod candidate;
pub mod chain;
pub mod extend;
pub mod insert_size;
pub mod mapq;
pub mod pipeline;
pub mod seed;
pub mod supplementary;
pub mod sw;

pub use candidate::{collect_candidates, dedup_candidates, AlignCandidate};
pub use chain::{best_chain, build_chains, build_chains_with_limit, filter_chains, Chain};
pub use extend::{chain_to_alignment, chain_to_alignment_buf, chain_to_alignment_with_zdrop, ChainAlignResult};
pub use mapq::compute_mapq;
pub use pipeline::{align_fastq_with_fm_opt, align_fastq_with_opt};
pub use seed::{find_mem_seeds, find_smem_seeds, find_smem_seeds_with_max_occ, AlnReg, MemSeed};
pub use supplementary::{are_non_overlapping, classify_alignments, generate_sa_tag, AlignmentType};
pub use sw::{banded_sw, SwParams, SwResult};

/// Re-export DEFAULT_MAX_OCC from seed module
pub use seed::DEFAULT_MAX_OCC;

/// Re-export DEFAULT_MAX_CHAINS_PER_CONTIG from chain module
pub use chain::DEFAULT_MAX_CHAINS_PER_CONTIG;

/// Default maximum alignments output per read
pub const DEFAULT_MAX_ALIGNMENTS_PER_READ: usize = 5;

/// Default Z-drop threshold for alignment extension
pub const DEFAULT_ZDROP: i32 = 100;

/// Default maximum insert size for paired-end alignment
pub const DEFAULT_MAX_INSERT: usize = 500;

/// Default minimum insert size for paired-end alignment
pub const DEFAULT_MIN_INSERT: usize = 0;

/// Options for paired-end alignment.
#[derive(Clone, Copy, Debug)]
pub struct PairingOpt {
    /// Minimum insert size (distance between read pairs)
    pub min_insert: usize,
    /// Maximum insert size
    pub max_insert: usize,
    /// Enable mate rescue for unmapped mates
    pub mate_rescue: bool,
    /// Penalty for unpaired alignments
    pub pen_unpaired: i32,
}

impl Default for PairingOpt {
    fn default() -> Self {
        Self {
            min_insert: DEFAULT_MIN_INSERT,
            max_insert: DEFAULT_MAX_INSERT,
            mate_rescue: true,
            pen_unpaired: 17,
        }
    }
}

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
}
