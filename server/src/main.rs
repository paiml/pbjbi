use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "pbjbi")]
#[command(about = "Pragmatic Business Intelligence with Deterministic Processing", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run statistical analysis
    Analyze {
        /// Input data file
        #[arg(short, long)]
        input: String,

        /// Output format
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Start MCP server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Run quality checks
    Quality {
        /// Data to check
        #[arg(short, long)]
        data: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pbjbi=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { input, format } => {
            tracing::info!("Analyzing data from {} with format {}", input, format);
            println!("Analysis feature will be implemented in next release");
        }
        Commands::Serve { port } => {
            tracing::info!("Starting MCP server on port {}", port);
            println!("MCP server feature will be implemented in next release");
        }
        Commands::Quality { data } => {
            tracing::info!("Running quality checks on {}", data);
            println!("Quality checks feature will be implemented in next release");
        }
    }

    Ok(())
}
