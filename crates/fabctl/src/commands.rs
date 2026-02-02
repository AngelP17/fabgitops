use crate::k8s_client::K8sClient;
use crate::output::{print_plc_table, print_status_summary, StatusStyle};
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(name = "fabctl")]
#[command(about = "FabGitOps CLI - Control industrial PLCs via GitOps")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Kubernetes namespace
    #[arg(short, long, global = true, default_value = "default")]
    pub namespace: String,

    /// Output format
    #[arg(short, long, global = true, value_enum, default_value = "table")]
    pub output: OutputFormat,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Get status of all managed PLCs
    GetStatus {
        /// Filter by PLC name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Get detailed information about a specific PLC
    Describe {
        /// Name of the PLC resource
        name: String,
    },

    /// Manually trigger a sync (reconciliation)
    Sync {
        /// Name of the PLC resource
        name: String,

        /// Force sync even if in sync
        #[arg(long)]
        force: bool,
    },

    /// Watch PLC status in real-time
    Watch {
        /// Refresh interval in seconds
        #[arg(short, long, default_value = "2")]
        interval: u64,
    },

    /// List all managed PLCs
    List,

    /// Show version information
    Version,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

/// Execute the get-status command
pub async fn cmd_get_status(
    client: &K8sClient,
    namespace: &str,
    name_filter: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let plcs: Vec<operator::crd::IndustrialPLC> = client.list_plcs(namespace).await?;

    let filtered: Vec<_> = if let Some(name) = name_filter {
        plcs.into_iter()
            .filter(|p: &operator::crd::IndustrialPLC| {
                p.metadata
                    .name
                    .as_ref()
                    .map(|n| n.contains(name))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        plcs
    };

    match format {
        OutputFormat::Table => print_plc_table(&filtered),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&filtered)?),
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&filtered)?),
    }

    Ok(())
}

/// Execute the describe command
pub async fn cmd_describe(client: &K8sClient, namespace: &str, name: &str) -> Result<()> {
    let plc = client.get_plc(namespace, name).await?;

    println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".bright_blue()
    );
    println!(
        "{}",
        "║              Industrial PLC Resource Details               ║".bright_blue()
    );
    println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".bright_blue()
    );
    println!();

    // Metadata
    println!("{}", "📋 Metadata:".bold().underline());
    println!(
        "  Name:        {}",
        plc.metadata.name.as_deref().unwrap_or("N/A").cyan()
    );
    println!(
        "  Namespace:   {}",
        plc.metadata.namespace.as_deref().unwrap_or("N/A")
    );
    println!(
        "  Created:     {}",
        plc.metadata
            .creation_timestamp
            .as_ref()
            .map(|t| t.0.to_string())
            .unwrap_or_default()
    );
    println!();

    // Spec
    println!("{}", "⚙️  Specification:".bold().underline());
    println!("  Device Address:  {}", plc.spec.device_address.cyan());
    println!("  Port:            {}", plc.spec.port);
    println!("  Target Register: {}", plc.spec.target_register);
    println!(
        "  Target Value:    {}",
        plc.spec.target_value.to_string().green()
    );
    println!("  Poll Interval:   {}s", plc.spec.poll_interval_secs);
    println!(
        "  Auto Correct:    {}",
        if plc.spec.auto_correct {
            "✓ enabled".green()
        } else {
            "✗ disabled".red()
        }
    );
    if !plc.spec.tags.is_empty() {
        println!("  Tags:            {}", plc.spec.tags.join(", "));
    }
    println!();

    // Status
    if let Some(status) = plc.status {
        let style = if status.in_sync {
            StatusStyle::Success
        } else if matches!(status.phase, operator::crd::PLCPhase::DriftDetected) {
            StatusStyle::Warning
        } else {
            StatusStyle::Error
        };

        print_status_summary(&status, style);
    } else {
        println!("{}", "⚠️  No status available".yellow());
    }

    Ok(())
}

/// Execute the sync command
pub async fn cmd_sync(client: &K8sClient, namespace: &str, name: &str, force: bool) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};

    println!("{}", "🔄 Triggering manual sync...".cyan());

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Annotating resource...");

    client.trigger_reconcile(namespace, name, force).await?;

    spinner.finish_with_message(format!("{}", "✓ Sync triggered successfully!".green()));

    // Show updated status
    println!();
    println!("{}", "Fetching updated status...".dimmed());
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    cmd_describe(client, namespace, name).await?;

    Ok(())
}

/// Execute the watch command
pub async fn cmd_watch(client: &K8sClient, namespace: &str, interval_secs: u64) -> Result<()> {
    use std::io::stdout;

    println!("{}", "👁️  Watching PLC status (Ctrl+C to exit)...".cyan());
    println!();

    let mut stdout = stdout();

    loop {
        // Clear screen using ANSI escape codes
        print!("\x1B[2J\x1B[1;1H");

        // Print header
        println!(
            "{}",
            "╔════════════════════════════════════════════════════════════╗".bright_blue()
        );
        println!(
            "{}",
            "║           FabGitOps Live Dashboard (fabctl watch)          ║".bright_blue()
        );
        println!(
            "{}",
            "╚════════════════════════════════════════════════════════════╝".bright_blue()
        );
        println!(
            "  Namespace: {} | Refresh: {}s | Press Ctrl+C to exit",
            namespace.cyan(),
            interval_secs
        );
        println!();

        // Fetch and display
        match client.list_plcs(namespace).await {
            Ok(plcs) => print_plc_table(&plcs),
            Err(e) => println!("{} {}", "Error:".red().bold(), e),
        }

        println!();
        println!(
            "{}",
            format!(
                "Last updated: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            )
            .dimmed()
        );

        // Flush stdout
        use std::io::Write;
        stdout.flush()?;

        tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
    }
}

/// Execute the list command
pub async fn cmd_list(client: &K8sClient, namespace: &str) -> Result<()> {
    let plcs = client.list_plcs(namespace).await?;

    println!("{}", "Managed Industrial PLCs".bold().underline());
    println!();

    for plc in plcs {
        let name = plc.metadata.name.as_deref().unwrap_or("unknown");
        let status_icon = if plc.status.as_ref().map(|s| s.in_sync).unwrap_or(false) {
            "✓".green()
        } else {
            "✗".red()
        };

        println!(
            "{} {} @ {}:{}",
            status_icon,
            name.cyan(),
            plc.spec.device_address,
            plc.spec.port
        );
    }

    Ok(())
}

/// Execute the version command
pub async fn cmd_version() -> Result<()> {
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
    );

    println!("Version: {}", "0.1.0".green());
    println!("Description: FabGitOps CLI - Control industrial PLCs via GitOps");
    println!("Author: Your Name");
    println!();
    println!("Stack:");
    println!("  - Rust 2021 Edition");
    println!("  - kube-rs (Kubernetes client)");
    println!("  - clap (CLI framework)");
    println!("  - tokio (Async runtime)");

    Ok(())
}
