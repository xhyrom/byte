mod cli;
mod compression;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source: Option<PathBuf>,
    #[arg(short, long)]
    destination: Option<PathBuf>,
    #[arg(short, long, default_value_t = 80.)]
    quality: f32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    cli::run(
        args.source.unwrap(),
        args.destination.unwrap(),
        args.quality,
    )
    .await?;

    Ok(())
}
