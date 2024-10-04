use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Prepares vcf files for training and analysis
    Prepare(PrepareArgs),
    /// Compares two populations
    Compare(CompareArgs),
}

#[derive(Args)]
struct PrepareArgs {
    /// Path to vcf or bcf file
    #[arg(short, long, required = true)]
    vcf: String,
}

#[derive(Args)]
struct CompareArgs {
    /// Newline-separated file of sample names to compare
    #[arg(short, long, required = true)]
    samples: String,
    /// Path and prefix of reference
    #[arg(short, long, required = true)]
    reference: String,
    /// Path and prefix of target
    #[arg(short, long, required = true)]
    target: String,
    /// Posterior probability threshold for the inclusion of a locus in the comparison
    /// If included, fb.tsv file must be available for reference
    #[arg(long)]
    threshold: Option<f32>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Prepare(args) => {
            clonehort::process_variant_input(&args.vcf)?;
        }
        Commands::Compare(args) => {
            let (samples, n_shared_by_col, n_total_by_col) = clonehort::perform_comparison(
                &args.samples,
                &args.reference,
                &args.target,
                args.threshold,
            )?;
            clonehort::display_comparison(samples, n_shared_by_col, n_total_by_col)?;
        }
    }

    // println!(
    //     "path: {}",
    //     args.path.into_os_string().into_string().unwrap()
    // );
    // let path = String::from(
    //     "/media/storage/1000_genomes/GRCh38/variants/chr20/mxl.chr20.chunk1.GRCh38.vcf.gz",
    // );

    //let path = String::from("/media/storage/1000_genomes/GRCh38/variants/chr20/mxl.chr20.GRCh38.vcf.gz");

    // clonehort::read(&path);

    Ok(())
}
