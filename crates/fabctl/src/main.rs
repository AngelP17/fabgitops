mod commands;
mod k8s_client;
mod output;

use crate::commands::*;
use crate::k8s_client::K8sClient;
use clap::Parser;
use colored::*;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Print banner
    print_banner();

    // Create K8s client
    let client = K8sClient::new().await?;

    // Execute command
    let result = match &cli.command {
        Commands::GetStatus { name } => {
            cmd_get_status(&client, &cli.namespace, name.as_deref(), cli.output).await
        }
        Commands::Describe { name } => cmd_describe(&client, &cli.namespace, name).await,
        Commands::Sync { name, force } => cmd_sync(&client, &cli.namespace, name, *force).await,
        Commands::Watch { interval } => cmd_watch(&client, &cli.namespace, *interval).await,
        Commands::List => cmd_list(&client, &cli.namespace).await,
        Commands::Version => cmd_version().await,
    };

    if let Err(ref e) = result {
        error!("{}", e);
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}

fn print_banner() {
    println!(
        "{}",
        r#"
    ███████╗ █████╗ ██████╗  ██████╗ ██╗████████╗ ██████╗ ██████╗ ███████╗
    ██╔════╝██╔══██╗██╔══██╗██╔════╝ ██║╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝
    █████╗  ███████║██████╔╝██║  ███╗██║   ██║   ██║   ██║██████╔╝███████╗
    ██╔══╝  ██╔══██║██╔══██╗██║   ██║██║   ██║   ██║   ██║██╔═══╝ ╚════██║
    ██║     ██║  ██║██████╔╝╚██████╔╝██║   ██║   ╚██████╔╝██║     ███████║
    ╚═╝     ╚═╝  ╚═╝╚═════╝  ╚═════╝ ╚═╝   ╚═╝    ╚═════╝ ╚═╝     ╚══════╝
    "#
        .bright_cyan()
        .dimmed()
    );
}
