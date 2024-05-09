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
    /// Compares two populations
    Compare(CompareArgs),
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
    #[arg(long)]
    threshold: Option<f32>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Compare(args) => {
            clonehort::perform_comparison(
                &args.samples,
                &args.reference,
                &args.target,
                args.threshold,
            )?;
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
