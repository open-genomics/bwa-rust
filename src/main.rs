// jemalloc：替换默认分配器，补回 musl malloc 性能，多线程 rayon 场景显著提升
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

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
    /// Build an FM index from a reference FASTA
    Index {
        /// Reference FASTA file
        reference: String,
        /// Output prefix for the generated .fm index
        #[arg(short, long, default_value = "ref")]
        output: String,
    },
    /// Align reads in FASTQ against an existing FM index
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
        #[arg(long = "clip-penalty", default_value_t = 1)]
        clip_penalty: i32,
        #[arg(long = "band-width", default_value_t = 16)]
        band_width: usize,
        #[arg(long = "score-threshold", default_value_t = 20)]
        score_threshold: i32,
        /// Minimum seed length
        #[arg(short = 'k', long = "min-seed-len", default_value_t = 19)]
        min_seed_len: usize,
        /// Z-drop threshold for alignment extension
        #[arg(short = 'd', long = "z-drop", default_value_t = 100)]
        zdrop: i32,
        /// Preset configuration (pacbio, ont2d)
        #[arg(short = 'x', long = "preset")]
        preset: Option<String>,
        /// Number of threads
        #[arg(short = 't', long = "threads", value_parser = parse_threads, default_value_t = 1)]
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
        /// Soft-clipping penalty used during candidate ranking
        #[arg(long = "clip-penalty", default_value_t = 1)]
        clip_penalty: i32,
        /// Band width for banded SW
        #[arg(short = 'w', long = "band-width", default_value_t = 100)]
        band_width: usize,
        /// Minimum alignment score to output (BWA default: 30, lowered for short reads)
        #[arg(short = 'T', long = "score-threshold", default_value_t = 10)]
        score_threshold: i32,
        /// Minimum seed length
        #[arg(short = 'k', long = "min-seed-len", default_value_t = 19)]
        min_seed_len: usize,
        /// Z-drop threshold for alignment extension
        #[arg(short = 'd', long = "z-drop", default_value_t = 100)]
        zdrop: i32,
        /// Preset configuration (pacbio, ont2d)
        #[arg(short = 'x', long = "preset")]
        preset: Option<String>,
        /// Number of threads
        #[arg(short = 't', long = "threads", value_parser = parse_threads, default_value_t = 1)]
        threads: usize,
    },
}

fn parse_threads(s: &str) -> std::result::Result<usize, String> {
    let threads: usize = s.parse().map_err(|_| "threads must be a positive integer".to_string())?;
    if threads == 0 {
        return Err("threads must be >= 1".to_string());
    }
    Ok(threads)
}

/// Apply preset configuration to alignment options
fn apply_preset(opt: &mut align::AlignOpt, preset: &str) {
    match preset {
        "pacbio" | "pacbio-hifi" => {
            opt.min_seed_len = 17;
            opt.band_width = 200;
            opt.score_threshold = 10;
        }
        "ont2d" | "ont" => {
            opt.min_seed_len = 14;
            opt.band_width = 100;
            opt.score_threshold = 10;
        }
        _ => {
            eprintln!(
                "[bwa-rust] Warning: unknown preset '{}', using default parameters",
                preset
            );
        }
    }
}

fn build_align_opt(
    match_score: i32,
    mismatch_penalty: i32,
    gap_open: i32,
    gap_extend: i32,
    clip_penalty: i32,
    band_width: usize,
    score_threshold: i32,
    min_seed_len: usize,
    zdrop: i32,
    threads: usize,
    preset: Option<&str>,
) -> align::AlignOpt {
    let mut opt = align::AlignOpt {
        match_score,
        mismatch_penalty,
        gap_open,
        gap_extend,
        clip_penalty,
        band_width,
        score_threshold,
        min_seed_len,
        threads,
        zdrop,
        ..align::AlignOpt::default()
    };

    if let Some(p) = preset {
        apply_preset(&mut opt, p);
    }

    if let Err(e) = opt.validate() {
        eprintln!("Error: invalid alignment parameters: {}", e);
        std::process::exit(1);
    }
    opt
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
            clip_penalty,
            band_width,
            score_threshold,
            min_seed_len,
            zdrop,
            preset,
            threads,
        } => {
            let opt = build_align_opt(
                match_score,
                mismatch_penalty,
                gap_open,
                gap_extend,
                clip_penalty,
                band_width,
                score_threshold,
                min_seed_len,
                zdrop,
                threads,
                preset.as_deref(),
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
            clip_penalty,
            band_width,
            score_threshold,
            min_seed_len,
            zdrop,
            preset,
            threads,
        } => {
            let opt = build_align_opt(
                match_score,
                mismatch_penalty,
                gap_open,
                gap_extend,
                clip_penalty,
                band_width,
                score_threshold,
                min_seed_len,
                zdrop,
                threads,
                preset.as_deref(),
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
