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
        /// Minimum seed length
        #[arg(short = 'k', long = "min-seed-len", default_value_t = align::AlignOpt::default().min_seed_len)]
        min_seed_len: usize,
        /// Number of threads
        #[arg(short = 't', long = "threads", value_parser = parse_threads, default_value_t = align::AlignOpt::default().threads)]
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
        /// Minimum seed length
        #[arg(short = 'k', long = "min-seed-len", default_value_t = align::AlignOpt::default().min_seed_len)]
        min_seed_len: usize,
        /// Number of threads
        #[arg(short = 't', long = "threads", value_parser = parse_threads, default_value_t = align::AlignOpt::default().threads)]
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

fn build_align_opt(min_seed_len: usize, threads: usize) -> align::AlignOpt {
    let opt = align::AlignOpt {
        min_seed_len,
        threads,
        ..align::AlignOpt::default()
    };
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
            min_seed_len,
            threads,
        } => {
            let opt = build_align_opt(min_seed_len, threads);
            run_align(&index, &reads, out.as_deref(), opt)
        }
        Commands::Mem {
            reference,
            reads,
            out,
            min_seed_len,
            threads,
        } => {
            let opt = build_align_opt(min_seed_len, threads);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_defaults_match_align_opt_default() {
        let cli = Cli::try_parse_from(["bwa-rust", "align", "-i", "ref.fm", "reads.fq"]).unwrap();
        let defaults = align::AlignOpt::default();

        let Commands::Align {
            min_seed_len, threads, ..
        } = cli.command
        else {
            panic!("expected align command");
        };

        assert_eq!(min_seed_len, defaults.min_seed_len);
        assert_eq!(threads, defaults.threads);
    }

    #[test]
    fn mem_defaults_match_align_opt_default() {
        let cli = Cli::try_parse_from(["bwa-rust", "mem", "ref.fa", "reads.fq"]).unwrap();
        let defaults = align::AlignOpt::default();

        let Commands::Mem {
            min_seed_len, threads, ..
        } = cli.command
        else {
            panic!("expected mem command");
        };

        assert_eq!(min_seed_len, defaults.min_seed_len);
        assert_eq!(threads, defaults.threads);
    }
}
