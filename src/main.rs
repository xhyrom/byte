mod compression;
mod ui;

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

    #[arg(long, default_value_t = false, help = "Launch Terminal UI")]
    tui: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    let (source, destination, quality) =
        if args.tui || args.source.is_none() || args.destination.is_none() {
            let input = ui::tui::run_tui()?;
            (input.source, input.destination, input.quality)
        } else {
            (
                args.source.unwrap(),
                args.destination.unwrap(),
                args.quality,
            )
        };

    ui::cli::run(source, destination, quality).await?;
    Ok(())
}
