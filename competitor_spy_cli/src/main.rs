// competitor_spy_cli — entry point only.
//
// No domain logic lives here. This binary parses CLI arguments and
// delegates entirely to the domain and adapter crates.
// Implemented in T-016.

// competitor-spy CLI entry point — T-016
// Thin argument-parsing wrapper; all logic lives in competitor_spy_cli::runner.

use std::collections::HashMap;
use std::path::PathBuf;

use clap::Parser;

use competitor_spy_cli::runner::{AdapterUrls, run_with_urls};
use competitor_spy_telemetry::init::init_telemetry;

#[derive(Parser, Debug)]
#[command(
    name = "competitor-spy",
    version,
    about = "Discover and rank competitors within a geographic radius"
)]
struct Cli {
    /// Industry or business category to search for
    #[arg(long)]
    industry: String,

    /// Human-readable location string (e.g. "London, UK")
    #[arg(long)]
    location: String,

    /// Search radius in km: 5, 10, 20, 25, or 50
    #[arg(long)]
    radius: u32,

    /// Directory to write the PDF report (default: current directory)
    #[arg(long, default_value = ".")]
    output_dir: PathBuf,

    /// Skip PDF output; only render to terminal
    #[arg(long, default_value_t = false)]
    no_pdf: bool,

    /// Log verbosity: trace, debug, info, warn, error
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Seed for request pacing (overrides CSPY_PACING_SEED env var)
    #[arg(long)]
    pacing_seed: Option<u64>,
}

#[tokio::main]
async fn main() {
    std::process::exit(run().await);
}

async fn run() -> i32 {
    let cli = Cli::parse();

    // Pacing seed: CLI flag > env var (informational for now; passed to registry in future)
    let _pacing_seed: Option<u64> = cli.pacing_seed.or_else(|| {
        std::env::var("CSPY_PACING_SEED")
            .ok()
            .and_then(|s| s.parse().ok())
    });

    let _guard = match init_telemetry(&cli.log_level) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("error: failed to initialise telemetry: {e}");
            return 1;
        }
    };

    run_with_urls(
        &cli.industry,
        &cli.location,
        cli.radius,
        &cli.output_dir,
        cli.no_pdf,
        AdapterUrls::production(),
        HashMap::new(),
    )
    .await
}
