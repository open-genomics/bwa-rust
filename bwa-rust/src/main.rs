use anyhow::Result;
use clap::{Parser, Subcommand};

use bwa_rust::align;
use bwa_rust::index;

#[derive(Parser, Debug)]
#[command(
    name = "bwa-rust",
    author,
    version,
    about = "Rust implementation inspired by BWA",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build index of the reference (BWT/FM will be added later)
    Index {
        /// Reference FASTA file
        reference: String,
        /// Output prefix for index files (not used yet)
        #[arg(short, long, default_value = "ref")]
        output: String,
    },
    /// Align reads (FASTQ) using an FM index (exact match MVP)
    Align {
        /// Path to FM index (.fm)
        #[arg(short = 'i', long = "index")]
        index: String,
        /// Reads FASTQ file
        reads: String,
        /// Output SAM path (stdout if omitted)
        #[arg(short, long)]
        out: Option<String>,
        #[arg(long = "match", default_value_t = 2)]
        match_score: i32,
        #[arg(long = "mismatch", default_value_t = 1)]
        mismatch_penalty: i32,
        #[arg(long = "gap-open", default_value_t = 2)]
        gap_open: i32,
        #[arg(long = "gap-ext", default_value_t = 1)]
        gap_extend: i32,
        #[arg(long = "band-width", default_value_t = 16)]
        band_width: usize,
        #[arg(long = "score-threshold", default_value_t = 20)]
        score_threshold: i32,
        #[arg(short = 't', long = "threads", default_value_t = 1)]
        threads: usize,
    },
    /// BWA-MEM style alignment: build index from FASTA and align FASTQ in one step
    Mem {
        /// Reference FASTA file
        reference: String,
        /// Reads FASTQ file
        reads: String,
        /// Output SAM path (stdout if omitted)
        #[arg(short, long)]
        out: Option<String>,
        /// Match score
        #[arg(short = 'A', long = "match", default_value_t = 1)]
        match_score: i32,
        /// Mismatch penalty
        #[arg(short = 'B', long = "mismatch", default_value_t = 4)]
        mismatch_penalty: i32,
        /// Gap open penalty
        #[arg(short = 'O', long = "gap-open", default_value_t = 6)]
        gap_open: i32,
        /// Gap extension penalty
        #[arg(short = 'E', long = "gap-ext", default_value_t = 1)]
        gap_extend: i32,
        /// Band width for banded SW
        #[arg(short = 'w', long = "band-width", default_value_t = 100)]
        band_width: usize,
        /// Minimum alignment score to output (BWA default: 30, lowered for short reads)
        #[arg(short = 'T', long = "score-threshold", default_value_t = 10)]
        score_threshold: i32,
        /// Number of threads
        #[arg(short = 't', long = "threads", default_value_t = 1)]
        threads: usize,
    },
}

fn build_align_opt(
    match_score: i32,
    mismatch_penalty: i32,
    gap_open: i32,
    gap_extend: i32,
    band_width: usize,
    score_threshold: i32,
    threads: usize,
) -> align::AlignOpt {
    align::AlignOpt {
        match_score,
        mismatch_penalty,
        gap_open,
        gap_extend,
        band_width,
        score_threshold,
        min_seed_len: 19,
        threads,
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Index { reference, output } => run_index(&reference, &output),
        Commands::Align {
            index,
            reads,
            out,
            match_score,
            mismatch_penalty,
            gap_open,
            gap_extend,
            band_width,
            score_threshold,
            threads,
        } => {
            let opt = build_align_opt(
                match_score,
                mismatch_penalty,
                gap_open,
                gap_extend,
                band_width,
                score_threshold,
                threads,
            );
            run_align(&index, &reads, out.as_deref(), opt)
        }
        Commands::Mem {
            reference,
            reads,
            out,
            match_score,
            mismatch_penalty,
            gap_open,
            gap_extend,
            band_width,
            score_threshold,
            threads,
        } => {
            let opt = build_align_opt(
                match_score,
                mismatch_penalty,
                gap_open,
                gap_extend,
                band_width,
                score_threshold,
                threads,
            );
            run_mem(&reference, &reads, out.as_deref(), opt)
        }
    }
}

fn run_index(reference: &str, output: &str) -> Result<()> {
    let mut result = index::builder::build_fm_from_fasta(reference, 512)?;

    println!("reference: {}", reference);
    println!("sequences: {}", result.n_seqs);
    println!("total_len: {}", result.total_len);

    result.fm.set_meta(index::fm::IndexMeta {
        reference_file: Some(reference.to_string()),
        build_args: Some(std::env::args().collect::<Vec<_>>().join(" ")),
        build_timestamp: Some(chrono::Utc::now().to_rfc3339()),
    });

    let out_path = format!("{}.fm", output);
    result
        .fm
        .save_to_file(&out_path)
        .map_err(|e| anyhow::anyhow!("cannot write index to '{}': {}", out_path, e))?;
    println!("FM index saved: {}", out_path);
    Ok(())
}

fn run_align(index_path: &str, reads_path: &str, out_path: Option<&str>, opt: align::AlignOpt) -> Result<()> {
    align::align_fastq_with_opt(index_path, reads_path, out_path, opt)
}

fn run_mem(reference: &str, reads_path: &str, out_path: Option<&str>, opt: align::AlignOpt) -> Result<()> {
    eprintln!("[bwa-rust mem] Loading reference: {}", reference);

    let result = index::builder::build_fm_from_fasta(reference, 512)?;

    eprintln!(
        "[bwa-rust mem] {} sequences, {} bp total",
        result.n_seqs, result.total_len
    );
    eprintln!("[bwa-rust mem] FM index built");

    let fm = std::sync::Arc::new(result.fm);

    eprintln!("[bwa-rust mem] Aligning reads from: {}", reads_path);
    align::align_fastq_with_fm_opt(fm, reads_path, out_path, opt)
}
