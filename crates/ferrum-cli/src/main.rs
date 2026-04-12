//! Ferrum CLI - main entry point.

use clap::{Parser, Subcommand};
use ferrum_core::config::FerrumConfig;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ferrum", version, about = "Ferrum - Rust Trading Agent Harness")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "ferrum.toml")]
    config: PathBuf,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },

    /// Start an MCP server
    Mcp {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8081)]
        port: u16,
    },

    /// Run a trading agent
    Run {
        /// Path to agent.md file
        #[arg(short, long)]
        agent: PathBuf,

        /// Run in paper trading mode
        #[arg(long, default_value_t = false)]
        paper: bool,
    },

    /// Start Telegram bot
    Telegram {
        /// Telegram bot token
        #[arg(long)]
        token: String,
    },

    /// List available agents
    List {
        /// Directory containing agent definitions
        #[arg(short, long, default_value = "agents")]
        dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| cli.log_level.clone().into())
        )
        .with_target(true)
        .with_thread_ids(true)
        .init();

    match cli.command {
        Some(Commands::Serve { port }) => {
            tracing::info!("Starting Ferrum API server on port {}", port);
            let server = ferrum_api::ApiServer::new(port);
            server.run().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Some(Commands::Mcp { port }) => {
            tracing::info!("Starting Ferrum MCP server on port {}", port);
            let server = ferrum_mcp::McpServer::new(port);
            server.run().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Some(Commands::Run { agent, paper }) => {
            tracing::info!("Running agent from {:?} (paper={})", agent, paper);
            let content = std::fs::read_to_string(&agent)?;
            let definition = ferrum_agent::AgentDefinitionParser::parse(&content)?;
            tracing::info!("Agent: {}", definition.name);
            tracing::info!("Pair: {}", definition.config.trading_pair);
            tracing::info!("Tick interval: {}s", definition.config.tick_interval_secs);
            tracing::info!("Limits: max_loss={}, max_position={}",
                definition.limits.max_daily_loss_quote,
                definition.limits.max_position_size_quote);
            println!("Agent '{}' loaded. Ready to run.", definition.name);
        }
        Some(Commands::Telegram { token }) => {
            tracing::info!("Starting Telegram bot");
            let bot = ferrum_telegram::TelegramBot::new(token);
            bot.run().await;
        }
        Some(Commands::List { dir }) => {
            if dir.exists() {
                for entry in std::fs::read_dir(&dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let agent_file = entry.path().join("agent.md");
                        if agent_file.exists() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            println!("📋 {}", name);
                        }
                    }
                }
            } else {
                println!("No agents directory found at {:?}", dir);
            }
        }
        None => {
            println!("Ferrum - Rust Trading Agent Harness v{}", env!("CARGO_PKG_VERSION"));
            println!("\nUsage: ferrum <COMMAND>\n");
            println!("Commands:");
            println!("  serve     Start the API server");
            println!("  mcp       Start the MCP server");
            println!("  run       Run a trading agent");
            println!("  telegram  Start Telegram bot");
            println!("  list      List available agents");
        }
    }

    Ok(())
}
