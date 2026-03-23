// competitor_spy_cli — entry point only.
//
// No domain logic lives here. This binary parses CLI arguments and
// delegates entirely to the domain and adapter crates.
// Implemented in T-016.

// competitor-spy CLI entry point — T-016
// Thin argument-parsing wrapper; all logic lives in competitor_spy_cli::runner.

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use competitor_spy_cli::runner::{AdapterUrls, credential_store_path, run_with_urls};
use competitor_spy_credentials::store::CredentialStore;
use competitor_spy_telemetry::init::init_telemetry;

/// Valid adapter IDs that accept credentials.
const CREDENTIAL_ADAPTERS: &[&str] = &["yelp", "google_places"];

#[derive(Parser, Debug)]
#[command(
    name = "competitor-spy",
    version,
    about = "Discover and rank competitors within a geographic radius"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Industry or business category to search for
    #[arg(long)]
    industry: Option<String>,

    /// Human-readable location string (e.g. "London, UK")
    #[arg(long)]
    location: Option<String>,

    /// Search radius in km: 5, 10, 20, 25, or 50
    #[arg(long)]
    radius: Option<u32>,

    /// Directory to write the PDF report (default: current directory)
    #[arg(long, default_value = ".")]
    output_dir: PathBuf,

    /// Skip PDF output; only render to terminal
    #[arg(long, default_value_t = false)]
    no_pdf: bool,

    /// Show detailed card view: editorial summary, price level, and reviews (default: on)
    #[arg(long, default_value_t = true)]
    detail: bool,

    /// Log verbosity: trace, debug, info, warn, error
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Seed for request pacing (overrides CSPY_PACING_SEED env var)
    #[arg(long)]
    pacing_seed: Option<u64>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Store or delete an API key for an adapter (yelp, google_places)
    Credentials {
        #[command(subcommand)]
        action: CredentialAction,
    },
}

#[derive(Subcommand, Debug)]
enum CredentialAction {
    /// Encrypt and save an API key. Reads the key from stdin (no echo).
    Set {
        /// Adapter name: yelp or google_places
        adapter: String,
    },
    /// Remove a stored API key
    Delete {
        /// Adapter name: yelp or google_places
        adapter: String,
    },
    /// List which adapters currently have a stored key
    List,
}

#[tokio::main]
async fn main() {
    std::process::exit(run().await);
}

async fn run() -> i32 {
    let cli = Cli::parse();

    if let Some(Commands::Credentials { action }) = cli.command {
        return run_credentials(action);
    }

    // Search mode — --industry, --location, --radius are all required.
    let (industry, location, radius) = match (cli.industry, cli.location, cli.radius) {
        (Some(i), Some(l), Some(r)) => (i, l, r),
        _ => {
            eprintln!("error: --industry, --location, and --radius are required for a search.");
            eprintln!("       Run with --help for usage, or use 'credentials' subcommand to manage API keys.");
            return 1;
        }
    };

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
        &industry,
        &location,
        radius,
        &cli.output_dir,
        cli.no_pdf,
        cli.detail,
        AdapterUrls::production(),
        HashMap::new(),
    )
    .await
}

// ---------------------------------------------------------------------------
// Credential management (no telemetry, no async needed)
// ---------------------------------------------------------------------------

fn run_credentials(action: CredentialAction) -> i32 {
    let passphrase = match std::env::var("CSPY_CREDENTIAL_PASSPHRASE") {
        Ok(p) if !p.is_empty() => p,
        _ => {
            eprintln!("error: CSPY_CREDENTIAL_PASSPHRASE environment variable is not set.");
            eprintln!("       Set it first: $env:CSPY_CREDENTIAL_PASSPHRASE = \"your-passphrase\"");
            return 1;
        }
    };

    let cred_path = credential_store_path();

    // Ensure parent directory exists before opening/creating the store.
    if let Some(parent) = cred_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("error: cannot create credential store directory: {e}");
            return 1;
        }
    }

    let mut store = match CredentialStore::open(cred_path, passphrase) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to open credential store: {e}");
            return 1;
        }
    };

    match action {
        CredentialAction::Set { adapter } => {
            if !CREDENTIAL_ADAPTERS.contains(&adapter.as_str()) {
                eprintln!(
                    "error: unknown adapter '{}'. Valid values: {}",
                    adapter,
                    CREDENTIAL_ADAPTERS.join(", ")
                );
                return 1;
            }
            eprint!("Enter API key for {}: ", adapter);
            let _ = io::stderr().flush();
            let key = read_secret_from_stdin();
            if key.trim().is_empty() {
                eprintln!("error: API key must not be empty.");
                return 1;
            }
            match store.store(&adapter, key.trim()) {
                Ok(()) => {
                    eprintln!("Credential for '{}' stored successfully.", adapter);
                    0
                }
                Err(e) => {
                    eprintln!("error: failed to store credential: {e}");
                    1
                }
            }
        }
        CredentialAction::Delete { adapter } => {
            match store.delete(&adapter) {
                Ok(true) => {
                    eprintln!("Credential for '{}' deleted.", adapter);
                    0
                }
                Ok(false) => {
                    eprintln!("No credential found for '{}'.", adapter);
                    0
                }
                Err(e) => {
                    eprintln!("error: failed to delete credential: {e}");
                    1
                }
            }
        }
        CredentialAction::List => {
            eprintln!("Stored credentials:");
            for adapter in CREDENTIAL_ADAPTERS {
                let status = if store.contains(adapter) { "SET" } else { "not set" };
                eprintln!("  {:<20} {}", adapter, status);
            }
            0
        }
    }
}

/// Reads a line from stdin. The key should be piped in or typed directly;
/// it is never passed as a CLI argument (which would expose it in process lists).
fn read_secret_from_stdin() -> String {
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    line
}
