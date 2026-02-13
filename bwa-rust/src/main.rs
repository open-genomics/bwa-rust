use anyhow::Result;
use clap::{Parser, Subcommand};

mod io;
mod index;
mod util;
mod align;

#[derive(Parser, Debug)]
#[command(name = "bwa-rust", author, version, about = "Rust implementation inspired by BWA", arg_required_else_help = true)]
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
            let opt = align::AlignOpt {
                match_score,
                mismatch_penalty,
                gap_open,
                gap_extend,
                band_width,
                score_threshold,
                threads,
            };
            run_align(&index, &reads, out.as_deref(), opt)
        }
    }
}

fn run_index(reference: &str, output: &str) -> Result<()> {
    let fh = std::fs::File::open(reference)
        .map_err(|e| anyhow::anyhow!("cannot open reference FASTA '{}': {}", reference, e))?;
    let buf = std::io::BufReader::new(fh);
    let mut reader = io::fasta::FastaReader::new(buf);

    let mut n_seqs = 0usize;
    let mut total_len = 0usize;
    let mut text: Vec<u8> = Vec::new();
    let mut contigs: Vec<index::fm::Contig> = Vec::new();

    while let Some(rec) = reader.next_record()? {
        n_seqs += 1;
        total_len += rec.seq.len();
        let norm = util::dna::normalize_seq(&rec.seq);
        let start = text.len() as u32;
        for b in norm {
            text.push(util::dna::to_alphabet(b));
        }
        let len_u32 = (text.len() as u32).saturating_sub(start);
        contigs.push(index::fm::Contig { name: rec.id, len: len_u32, offset: start });
        // sentinel between contigs
        text.push(0);
    }

    if n_seqs == 0 {
        anyhow::bail!("FASTA file '{}' contains no sequences", reference);
    }
    if total_len == 0 {
        anyhow::bail!("FASTA file '{}' contains only empty sequences", reference);
    }

    println!("reference: {}", reference);
    println!("sequences: {}", n_seqs);
    println!("total_len: {}", total_len);

    // Build SA -> BWT -> FM
    let sa = index::sa::build_sa(&text);
    let bwt = index::bwt::build_bwt(&text, &sa);
    let mut fm = index::fm::FMIndex::build(text, bwt, sa, contigs, util::dna::SIGMA as u8, 512);
    fm.set_meta(index::fm::IndexMeta {
        reference_file: Some(reference.to_string()),
        build_args: Some(std::env::args().collect::<Vec<_>>().join(" ")),
        build_timestamp: Some(chrono::Utc::now().to_rfc3339()),
    });

    let out_path = format!("{}.fm", output);
    fm.save_to_file(&out_path)
        .map_err(|e| anyhow::anyhow!("cannot write index to '{}': {}", out_path, e))?;
    println!("FM index saved: {}", out_path);
    Ok(())
}

fn run_align(
    index_path: &str,
    reads_path: &str,
    out_path: Option<&str>,
    opt: align::AlignOpt,
) -> Result<()> {
    align::align_fastq_with_opt(index_path, reads_path, out_path, opt)
}
