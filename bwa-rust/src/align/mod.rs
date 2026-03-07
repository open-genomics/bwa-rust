pub mod seed;
pub mod chain;
pub mod sw;
pub mod extend;
pub mod mapq;
pub mod candidate;
pub mod pipeline;

pub use sw::{SwParams, SwResult, banded_sw};
pub use seed::{MemSeed, AlnReg, find_smem_seeds, find_mem_seeds};
pub use chain::{Chain, best_chain, build_chains, filter_chains};
pub use extend::{ChainAlignResult, chain_to_alignment, chain_to_alignment_buf};
pub use mapq::compute_mapq;
pub use candidate::{AlignCandidate, collect_candidates, dedup_candidates};
pub use pipeline::{align_fastq_with_opt, align_fastq_with_fm_opt};

#[derive(Clone, Copy, Debug)]
pub struct AlignOpt {
    pub match_score: i32,
    pub mismatch_penalty: i32,
    pub gap_open: i32,
    pub gap_extend: i32,
    pub band_width: usize,
    pub score_threshold: i32,
    pub min_seed_len: usize,
    pub threads: usize,
}

impl Default for AlignOpt {
    fn default() -> Self {
        Self {
            match_score: 2,
            mismatch_penalty: 1,
            gap_open: 2,
            gap_extend: 1,
            band_width: 16,
            score_threshold: 20,
            min_seed_len: 19,
            threads: 1,
        }
    }
}
