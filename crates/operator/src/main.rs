mod controller;
mod crd;
mod metrics;
mod plc_client;

use crate::controller::{error_policy, reconcile, Context};
use crate::crd::IndustrialPLC;
use crate::metrics::OperatorMetrics;
use axum::{routing::get, Router};
use futures::StreamExt;
use kube::runtime::events::Reporter;
use kube::{Api, Client};
use prometheus::TextEncoder;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting FabGitOps Operator...");

    // Initialize Kubernetes client
    let client = Client::try_default().await?;
    info!("Connected to Kubernetes cluster");

    // Initialize metrics
    let metrics = Arc::new(OperatorMetrics::new()?);
    info!("Metrics initialized");

    // Create context for controller
    let ctx = Arc::new(Context {
        client: client.clone(),
        metrics: metrics.clone(),
        reporter: Reporter {
            controller: "fabgitops-operator".to_string(),
            instance: std::env::var("HOSTNAME").ok(),
        },
    });

    // Start metrics server
    let metrics_router = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler));

    let metrics_addr: SocketAddr = "0.0.0.0:8080".parse()?;
    let metrics_clone = metrics.clone();

    tokio::spawn(async move {
        info!("Starting metrics server on {}", metrics_addr);
        let app = metrics_router.layer(axum::Extension(metrics_clone));
        axum::serve(
            tokio::net::TcpListener::bind(metrics_addr).await.unwrap(),
            app,
        )
        .await
        .unwrap();
    });

    // Start controller
    info!("Starting IndustrialPLC controller...");
    let plcs = Api::<IndustrialPLC>::all(client.clone());

    // Ensure CRD exists
    if let Err(e) = plcs.list(&Default::default()).await {
        info!("CRD may not exist yet: {}", e);
    }

    kube::runtime::Controller::new(plcs, Default::default())
        .run(reconcile, error_policy, ctx)
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled: {:?}", o),
                Err(e) => error!("Reconciliation error: {:?}", e),
            }
        })
        .await;

    Ok(())
}

/// Handler for /metrics endpoint
async fn metrics_handler(
    axum::Extension(metrics): axum::Extension<Arc<OperatorMetrics>>,
) -> String {
    let encoder = TextEncoder::new();
    let metric_families = metrics.registry.gather();
    encoder
        .encode_to_string(&metric_families)
        .unwrap_or_default()
}

/// Handler for /health endpoint
async fn health_handler() -> &'static str {
    "OK"
}
