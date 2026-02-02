# FabGitOps: Complete Project Implementation

> **The Infrastructure Layer** of the Industrial Cloud Stack  
> A Kubernetes Operator (Rust) + CLI Tool (`fabctl`) + Prometheus Metrics for Industrial PLC Management

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Architecture & Differentiation](#2-architecture--differentiation)
3. [Project Structure](#3-project-structure)
4. [Phase 1: Core Operator](#4-phase-1-the-core-operator-rust)
5. [Phase 2: Chaos Simulator](#5-phase-2-the-chaos-simulator)
6. [Phase 3: fabctl CLI Tool](#6-phase-3-the-fabctl-cli-tool)
7. [Phase 4: Observability Stack](#7-phase-4-the-observability-stack)
8. [Phase 5: Demo & Deployment](#8-phase-5-demo--deployment)
9. [Interview Talking Points](#9-interview-talking-points)

---

## 1. Project Overview

### The "Triple Threat" Portfolio Stack

| Project | **FactoryOps** | **ForgeMesh** | **FabGitOps** |
|---------|---------------|---------------|---------------|
| **The Layer** | **Presentation Layer** | **Data Layer** | **Infrastructure Layer** |
| **The Audience** | *Factory Managers* | *Software Architects* | *DevOps/SRE Engineers* |
| **The Interface** | **Web UI** (Next.js/React) | **API / P2P Mesh** (Backend) | **CLI Tool** (`fabctl`) & **Grafana** |
| **Key Skill** | "I can make it easy to use." | "I can handle complex data." | **"I can automate the platform."** |
| **The "Vibe"** | Polished, User-Centric | Complex, Algorithmic | **Robust, Utilities-Focused** |

### Why CLI + Metrics Over Web UI?

- **Internal Tooling**: Building `fabctl` proves you can build tools for *other developers*, not just end-users
- **Observability Engineering**: Shows you understand industry standards (Prometheus) rather than just custom JavaScript
- **Infrastructure-as-Code**: You're building the thing that *manages* deployments

**Interview Moment**: *Open the terminal. "Look how I control a global fleet of devices with one command."*

---

## 2. Architecture & Differentiation

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FabGitOps Architecture                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐                │
│  │   fabctl     │────▶│  Kubernetes  │────▶│   Operator   │                │
│  │    CLI       │     │     API      │     │   (Rust)     │                │
│  └──────────────┘     └──────────────┘     └──────┬───────┘                │
│        │                                          │                         │
│        │                                          ▼                         │
│        │                              ┌──────────────────────┐             │
│        │                              │  Reconciliation Loop │             │
│        │                              │  - Watch CRDs        │             │
│        │                              │  - Compare State     │             │
│        │                              │  - Fix Drift         │             │
│        │                              └──────────┬───────────┘             │
│        │                                         │                         │
│        │                    ┌────────────────────┼────────────────────┐    │
│        │                    │                    │                    │    │
│        ▼                    ▼                    ▼                    ▼    │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐│
│  │   Status     │   │   Modbus     │   │  Prometheus  │   │   Grafana    ││
│  │   Output     │   │   TCP/PLC    │   │   /metrics   │   │ Dashboard    ││
│  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘│
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow: GitOps in Action

```
Desired State (Git/K8s)          Actual State (PLC)              Action
─────────────────────           ─────────────────              ──────
target_register: 4001            register: 4001                 ✓ No drift
target_value: 2500               value: 2500                    ✓ Aligned

--- Chaos Event ---

Desired State (Git/K8s)          Actual State (PLC)              Action
─────────────────────           ─────────────────              ──────
target_register: 4001            register: 4001                 ✗ DRIFT DETECTED
target_value: 2500               value: 2847                    ✗ Fixing...
                                                                 ✓ Corrected
```

---

## 3. Project Structure

### Cargo Workspace Layout

```
fabgitops/
├── Cargo.toml                    # Workspace root
├── README.md
├── docker-compose.yml            # Prometheus + Grafana
├── prometheus.yml                # Prometheus config
├── grafana-dashboard.json        # Pre-built dashboard
│
├── crates/
│   ├── operator/                 # Kubernetes Operator (Rust)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── crd.rs           # IndustrialPLC CRD
│   │       ├── controller.rs    # Reconciliation loop
│   │       ├── metrics.rs       # Prometheus metrics
│   │       └── plc_client.rs    # Modbus client
│   │
│   ├── fabctl/                   # CLI Tool
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── commands.rs      # CLI commands
│   │       ├── output.rs        # Pretty table formatting
│   │       └── k8s_client.rs    # Kubernetes client
│   │
│   └── mock-plc/                 # Chaos Simulator
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── server.rs        # Modbus server
│           └── chaos.rs         # Chaos mode
│
└── k8s/
    ├── crd.yaml                  # CRD definition
    ├── rbac.yaml                 # Service account & permissions
    ├── deployment.yaml           # Operator deployment
    └── sample-plc.yaml           # Sample PLC resource
```

### Root Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/operator",
    "crates/fabctl",
    "crates/mock-plc",
]

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

---

## 4. Phase 1: The Core Operator (Rust)

### 4.1 Operator Cargo.toml

```toml
[package]
name = "operator"
version = "0.1.0"
edition = "2021"

[dependencies]
# Kubernetes
kube = { version = "0.87", features = ["runtime", "derive"] }
k8s-openapi = { version = "0.20", features = ["v1_28"] }

# Async runtime
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = "1.0"

# Observability
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
prometheus = "0.13"
axum = "0.7"

# Modbus
tokio-modbus = { version = "0.9", default-features = false, features = ["tcp"] }

# Utilities
schemars = "0.8"
futures = "0.3"
chrono = "0.4"
```

### 4.2 CRD Definition - `src/crd.rs`

```rust
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// IndustrialPLC is the Custom Resource Definition for managing
/// industrial PLCs via GitOps principles.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "fabgitops.io",
    version = "v1",
    kind = "IndustrialPLC",
    plural = "industrialplcs",
    shortname = "plc",
    namespaced,
    status = "IndustrialPLCStatus"
)]
#[serde(rename_all = "camelCase")]
pub struct IndustrialPLCSpec {
    /// IP address or hostname of the PLC device
    pub device_address: String,
    
    /// Port for Modbus TCP communication (default: 502)
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// The Modbus register address to monitor/control
    pub target_register: u16,
    
    /// The desired value for the target register
    pub target_value: u16,
    
    /// Polling interval in seconds (default: 5)
    #[serde(default = "default_interval")]
    pub poll_interval_secs: u64,
    
    /// Enable automatic drift correction
    #[serde(default = "default_auto_correct")]
    pub auto_correct: bool,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_port() -> u16 {
    502
}

fn default_interval() -> u64 {
    5
}

fn default_auto_correct() -> bool {
    true
}

/// Status subresource for IndustrialPLC
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndustrialPLCStatus {
    /// Current state of the PLC
    pub phase: PLCPhase,
    
    /// Last time the status was updated
    pub last_update: Option<String>,
    
    /// Current value read from the PLC
    pub current_value: Option<u16>,
    
    /// Whether the PLC matches desired state
    pub in_sync: bool,
    
    /// Number of drift events detected
    pub drift_events: u32,
    
    /// Number of successful corrections
    pub corrections_applied: u32,
    
    /// Last error message (if any)
    pub last_error: Option<String>,
    
    /// Human-readable message
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum PLCPhase {
    #[default]
    Pending,
    Connecting,
    Connected,
    DriftDetected,
    Correcting,
    Failed,
}

impl IndustrialPLCStatus {
    pub fn new() -> Self {
        Self {
            phase: PLCPhase::Pending,
            last_update: None,
            current_value: None,
            in_sync: false,
            drift_events: 0,
            corrections_applied: 0,
            last_error: None,
            message: "Initializing...".to_string(),
        }
    }
    
    pub fn set_synced(&mut self, value: u16) {
        self.phase = PLCPhase::Connected;
        self.current_value = Some(value);
        self.in_sync = true;
        self.last_error = None;
        self.message = format!("PLC in sync. Current value: {}", value);
        self.update_timestamp();
    }
    
    pub fn set_drift(&mut self, desired: u16, actual: u16) {
        self.phase = PLCPhase::DriftDetected;
        self.current_value = Some(actual);
        self.in_sync = false;
        self.drift_events += 1;
        self.message = format!(
            "DRIFT DETECTED! Desired: {}, Actual: {}",
            desired, actual
        );
        self.update_timestamp();
    }
    
    pub fn set_correcting(&mut self) {
        self.phase = PLCPhase::Correcting;
        self.message = "Applying correction...".to_string();
        self.update_timestamp();
    }
    
    pub fn set_corrected(&mut self, value: u16) {
        self.corrections_applied += 1;
        self.set_synced(value);
    }
    
    pub fn set_error(&mut self, error: String) {
        self.phase = PLCPhase::Failed;
        self.last_error = Some(error.clone());
        self.message = error;
        self.update_timestamp();
    }
    
    fn update_timestamp(&mut self) {
        self.last_update = Some(chrono::Utc::now().to_rfc3339());
    }
}
```

### 4.3 Metrics - `src/metrics.rs`

```rust
use prometheus::{Counter, Gauge, Registry, Opts};
use std::sync::Arc;

/// Metrics exposed by the operator
#[derive(Clone)]
pub struct OperatorMetrics {
    pub registry: Registry,
    
    /// Total drift events detected
    pub drift_events_total: Counter,
    
    /// Total corrections applied
    pub corrections_total: Counter,
    
    /// Current number of managed PLCs
    pub managed_plcs: Gauge,
    
    /// Reconciliation loop duration
    pub reconciliation_duration: Gauge,
    
    /// PLC connection status (1 = connected, 0 = disconnected)
    pub plc_connection_status: Gauge,
    
    /// Current register value
    pub register_value: Gauge,
}

impl OperatorMetrics {
    pub fn new() -> anyhow::Result<Self> {
        let registry = Registry::new();
        
        let drift_events_total = Counter::with_opts(
            Opts::new(
                "drift_events_total",
                "Total number of drift events detected across all PLCs"
            )
        )?;
        
        let corrections_total = Counter::with_opts(
            Opts::new(
                "corrections_total",
                "Total number of successful drift corrections"
            )
        )?;
        
        let managed_plcs = Gauge::with_opts(
            Opts::new(
                "managed_plcs",
                "Number of IndustrialPLC resources being managed"
            )
        )?;
        
        let reconciliation_duration = Gauge::with_opts(
            Opts::new(
                "reconciliation_duration_seconds",
                "Duration of last reconciliation loop in seconds"
            )
        )?;
        
        let plc_connection_status = Gauge::with_opts(
            Opts::new(
                "plc_connection_status",
                "Connection status of PLC (1 = connected, 0 = disconnected)"
            )
        )?;
        
        let register_value = Gauge::with_opts(
            Opts::new(
                "register_value",
                "Current value of the monitored register"
            )
        )?;
        
        registry.register(Box::new(drift_events_total.clone()))?;
        registry.register(Box::new(corrections_total.clone()))?;
        registry.register(Box::new(managed_plcs.clone()))?;
        registry.register(Box::new(reconciliation_duration.clone()))?;
        registry.register(Box::new(plc_connection_status.clone()))?;
        registry.register(Box::new(register_value.clone()))?;
        
        Ok(Self {
            registry,
            drift_events_total,
            corrections_total,
            managed_plcs,
            reconciliation_duration,
            plc_connection_status,
            register_value,
        })
    }
    
    pub fn record_drift(&self) {
        self.drift_events_total.inc();
    }
    
    pub fn record_correction(&self) {
        self.corrections_total.inc();
    }
    
    pub fn set_managed_plcs(&self, count: i64) {
        self.managed_plcs.set(count as f64);
    }
    
    pub fn set_connection_status(&self, connected: bool) {
        self.plc_connection_status.set(if connected { 1.0 } else { 0.0 });
    }
    
    pub fn set_register_value(&self, value: u16) {
        self.register_value.set(value as f64);
    }
}

impl Default for OperatorMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}
```

### 4.4 PLC Client - `src/plc_client.rs`

```rust
use tokio::net::TcpStream;
use tokio_modbus::prelude::*;
use anyhow::{Result, Context};

/// Client for communicating with Modbus TCP devices
pub struct PLCClient {
    address: String,
    port: u16,
}

impl PLCClient {
    pub fn new(address: impl Into<String>, port: u16) -> Self {
        Self {
            address: address.into(),
            port,
        }
    }
    
    /// Read a holding register from the PLC
    pub async fn read_register(&self, register: u16) -> Result<u16> {
        let socket_addr = format!("{}:{}", self.address, self.port)
            .parse()
            .context("Invalid PLC address")?;
        
        let stream = TcpStream::connect(socket_addr).await
            .context("Failed to connect to PLC")?;
        
        let mut ctx = tcp::attach(stream);
        
        // Modbus registers are 0-indexed internally
        let response = ctx.read_holding_registers(register, 1).await
            .context("Failed to read register")?;
        
        ctx.disconnect().await.ok();
        
        response.get(0)
            .copied()
            .context("Empty response from PLC")
    }
    
    /// Write a value to a holding register
    pub async fn write_register(&self, register: u16, value: u16) -> Result<()> {
        let socket_addr = format!("{}:{}", self.address, self.port)
            .parse()
            .context("Invalid PLC address")?;
        
        let stream = TcpStream::connect(socket_addr).await
            .context("Failed to connect to PLC")?;
        
        let mut ctx = tcp::attach(stream);
        
        ctx.write_single_register(register, value).await
            .context("Failed to write register")?;
        
        ctx.disconnect().await.ok();
        
        Ok(())
    }
    
    /// Check if the PLC is reachable
    pub async fn health_check(&self) -> Result<bool> {
        match TcpStream::connect(format!("{}:{}", self.address, self.port)).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
```

### 4.5 Controller - `src/controller.rs`

```rust
use crate::crd::{IndustrialPLC, IndustrialPLCStatus, PLCPhase};
use crate::metrics::OperatorMetrics;
use crate::plc_client::PLCClient;
use kube::api::{Api, Patch, PatchParams};
use kube::runtime::controller::{Action, Controller};
use kube::runtime::events::{Event, EventType, Recorder, Reporter};
use kube::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tracing::{error, info, warn};

/// Context passed to reconciliation
#[derive(Clone)]
pub struct Context {
    pub client: Client,
    pub metrics: Arc<OperatorMetrics>,
    pub reporter: Reporter,
}

/// Main reconciliation function
pub async fn reconcile(plc: Arc<IndustrialPLC>, ctx: Arc<Context>) -> Result<Action, Error> {
    let start = Instant::now();
    let name = plc.name_any();
    let namespace = plc.namespace().unwrap_or_default();
    
    info!("Reconciling PLC: {}/{}", namespace, name);
    
    let api: Api<IndustrialPLC> = Api::namespaced(ctx.client.clone(), &namespace);
    let mut status = IndustrialPLCStatus::new();
    
    // Create PLC client
    let plc_client = PLCClient::new(
        &plc.spec.device_address,
        plc.spec.port
    );
    
    // Health check
    match plc_client.health_check().await {
        Ok(true) => {
            ctx.metrics.set_connection_status(true);
            info!("PLC {}/{} is reachable", namespace, name);
        }
        Ok(false) | Err(_) => {
            ctx.metrics.set_connection_status(false);
            status.set_error("PLC unreachable".to_string());
            update_status(&api, &name, status).await?;
            return Ok(Action::requeue(Duration::from_secs(10)));
        }
    }
    
    // Read current value from PLC
    match plc_client.read_register(plc.spec.target_register).await {
        Ok(current_value) => {
            ctx.metrics.set_register_value(current_value);
            info!(
                "Register {} current value: {}, desired: {}",
                plc.spec.target_register, current_value, plc.spec.target_value
            );
            
            // Check for drift
            if current_value != plc.spec.target_value {
                // Drift detected!
                ctx.metrics.record_drift();
                status.set_drift(plc.spec.target_value, current_value);
                
                // Emit event
                let recorder = Recorder::new(
                    ctx.client.clone(),
                    ctx.reporter.clone(),
                    plc.object_ref(&()),
                );
                recorder
                    .publish(Event {
                        type_: EventType::Warning,
                        reason: "DriftDetected".to_string(),
                        note: Some(format!(
                            "Register {} drifted: desired={}, actual={}",
                            plc.spec.target_register,
                            plc.spec.target_value,
                            current_value
                        )),
                        action: "Reconcile".to_string(),
                        secondary: None,
                    })
                    .await
                    .ok();
                
                // Auto-correct if enabled
                if plc.spec.auto_correct {
                    status.set_correcting();
                    update_status(&api, &name, status.clone()).await?;
                    
                    match plc_client
                        .write_register(plc.spec.target_register, plc.spec.target_value)
                        .await
                    {
                        Ok(()) => {
                            ctx.metrics.record_correction();
                            status.set_corrected(plc.spec.target_value);
                            
                            recorder
                                .publish(Event {
                                    type_: EventType::Normal,
                                    reason: "DriftCorrected".to_string(),
                                    note: Some(format!(
                                        "Register {} corrected to {}",
                                        plc.spec.target_register,
                                        plc.spec.target_value
                                    )),
                                    action: "Reconcile".to_string(),
                                    secondary: None,
                                })
                                .await
                                .ok();
                            
                            info!(
                                "Corrected register {} to {}",
                                plc.spec.target_register, plc.spec.target_value
                            );
                        }
                        Err(e) => {
                            status.set_error(format!("Failed to correct: {}", e));
                            error!("Failed to correct drift: {}", e);
                        }
                    }
                }
            } else {
                // In sync
                status.set_synced(current_value);
            }
        }
        Err(e) => {
            status.set_error(format!("Failed to read register: {}", e));
            error!("Failed to read register: {}", e);
        }
    }
    
    // Update status
    update_status(&api, &name, status).await?;
    
    // Record metrics
    let duration = start.elapsed().as_secs_f64();
    ctx.metrics.reconciliation_duration.set(duration);
    
    // Requeue based on poll interval
    Ok(Action::requeue(Duration::from_secs(plc.spec.poll_interval_secs)))
}

/// Update the status subresource
async fn update_status(
    api: &Api<IndustrialPLC>,
    name: &str,
    status: IndustrialPLCStatus,
) -> Result<(), Error> {
    let patch = Patch::Merge(serde_json::json!({
        "status": status
    }));
    
    api.patch_status(name, &PatchParams::default(), &patch)
        .await
        .map_err(Error::KubeError)?;
    
    Ok(())
}

/// Error policy for failed reconciliations
pub fn error_policy(_plc: Arc<IndustrialPLC>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("Reconciliation failed: {:?}", error);
    Action::requeue(Duration::from_secs(5))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kubernetes error: {0}")]
    KubeError(#[from] kube::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
```

### 4.6 Main - `src/main.rs`

```rust
mod controller;
mod crd;
mod metrics;
mod plc_client;

use crate::controller::{error_policy, reconcile, Context};
use crate::crd::IndustrialPLC;
use crate::metrics::OperatorMetrics;
use axum::{routing::get, Router};
use kube::runtime::events::Reporter;
use kube::Client;
use prometheus::TextEncoder;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
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
    encoder.encode_to_string(&metric_families).unwrap_or_default()
}

/// Handler for /health endpoint
async fn health_handler() -> &'static str {
    "OK"
}
```

---

## 5. Phase 2: The Chaos Simulator

### 5.1 mock-plc Cargo.toml

```toml
[package]
name = "mock-plc"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
tokio-modbus = { version = "0.9", default-features = false, features = ["tcp", "server"] }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
anyhow = { workspace = true }
clap = { version = "4.4", features = ["derive"] }
rand = "0.8"
```

### 5.2 Chaos Module - `src/chaos.rs`

```rust
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

/// Chaos mode configuration
#[derive(Clone)]
pub struct ChaosConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub max_drift: u16,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: 10,
            max_drift: 500,
        }
    }
}

/// Manages chaos mode for simulated PLC drift
pub struct ChaosEngine {
    config: ChaosConfig,
    running: Arc<AtomicBool>,
}

impl ChaosEngine {
    pub fn new(config: ChaosConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Start the chaos engine in background
    pub fn spawn(&self, register_value: Arc<std::sync::Mutex<u16>>) {
        if !self.config.enabled {
            info!("Chaos mode disabled");
            return;
        }
        
        let running = self.running.clone();
        let interval_secs = self.config.interval_secs;
        let max_drift = self.config.max_drift;
        
        running.store(true, Ordering::SeqCst);
        
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            let mut rng = rand::thread_rng();
            
            info!(
                "🌀 CHAOS MODE ACTIVATED! Drifting every {}s (max drift: {})",
                interval_secs, max_drift
            );
            
            while running.load(Ordering::SeqCst) {
                ticker.tick().await;
                
                let drift: i16 = rng.gen_range(-(max_drift as i16)..=max_drift as i16);
                
                if let Ok(mut value) = register_value.lock() {
                    let old_value = *value;
                    let new_value = (*value as i16 + drift).clamp(0, 65535) as u16;
                    *value = new_value;
                    
                    warn!(
                        "🌀 CHAOS DRIFT! Register changed: {} → {} (drift: {})",
                        old_value, new_value, drift
                    );
                }
            }
        });
    }
    
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("Chaos mode stopped");
    }
}
```

### 5.3 Server Module - `src/server.rs`

```rust
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_modbus::prelude::*;
use tokio_modbus::server::tcp::{accept_tcp_connection, Server};
use tracing::{error, info};

/// Shared state for the mock PLC
pub struct PLCState {
    pub register_value: u16,
    pub register_address: u16,
}

impl PLCState {
    pub fn new(initial_value: u16, register_address: u16) -> Self {
        Self {
            register_value: initial_value,
            register_address,
        }
    }
}

/// Start the mock Modbus TCP server
pub async fn start_server(
    bind_addr: &str,
    port: u16,
    state: Arc<Mutex<PLCState>>,
) -> anyhow::Result<()> {
    let socket_addr: SocketAddr = format!("{}:{}", bind_addr, port).parse()?;
    
    info!("Starting mock PLC server on {}", socket_addr);
    
    let listener = TcpListener::bind(socket_addr).await?;
    let server = Server::new(listener);
    
    let new_service = |_socket_addr| {
        let state = state.clone();
        Ok(Some(ModbusService { state }))
    };
    
    let on_connected = |stream, socket_addr| async move {
        accept_tcp_connection(stream, socket_addr, new_service)
    };
    
    let on_process_error = |err| {
        error!("Server error: {}", err);
    };
    
    server.serve(&on_connected, on_process_error).await?;
    
    Ok(())
}

/// Modbus service implementation
#[derive(Clone)]
struct ModbusService {
    state: Arc<Mutex<PLCState>>,
}

impl tokio_modbus::server::Service for ModbusService {
    type Request = Request;
    type Response = Response;
    type Error = std::io::Error;
    type Future = std::future::Ready<std::result::Result<Self::Response, Self::Error>>;
    
    fn call(&self, req: Self::Request) -> Self::Future {
        let response = match req {
            Request::ReadHoldingRegisters(addr, count) => {
                if let Ok(state) = self.state.lock() {
                    if addr == state.register_address && count == 1 {
                        Response::ReadHoldingRegisters(vec![state.register_value])
                    } else {
                        Response::Custom(0x83, vec![0x02]) // Illegal data address
                    }
                } else {
                    Response::Custom(0x83, vec![0x04]) // Server failure
                }
            }
            Request::WriteSingleRegister(addr, value) => {
                if let Ok(mut state) = self.state.lock() {
                    if addr == state.register_address {
                        state.register_value = value;
                        info!("Register {} written with value: {}", addr, value);
                        Response::WriteSingleRegister(addr, value)
                    } else {
                        Response::Custom(0x86, vec![0x02]) // Illegal data address
                    }
                } else {
                    Response::Custom(0x86, vec![0x04]) // Server failure
                }
            }
            _ => Response::Custom(req.function_code() | 0x80, vec![0x01]), // Illegal function
        };
        
        std::future::ready(Ok(response))
    }
}
```

### 5.4 Main - `src/main.rs`

```rust
mod chaos;
mod server;

use crate::chaos::{ChaosConfig, ChaosEngine};
use crate::server::{start_server, PLCState};
use clap::Parser;
use std::sync::{Arc, Mutex};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "mock-plc")]
#[command(about = "Mock PLC server with chaos mode for testing FabGitOps")]
struct Args {
    /// Bind address
    #[arg(short, long, default_value = "0.0.0.0")]
    bind: String,
    
    /// Port to listen on
    #[arg(short, long, default_value = "5502")]
    port: u16,
    
    /// Initial register value
    #[arg(short, long, default_value = "2500")]
    value: u16,
    
    /// Register address to serve
    #[arg(short, long, default_value = "4001")]
    register: u16,
    
    /// Enable chaos mode (random drift)
    #[arg(long)]
    chaos: bool,
    
    /// Chaos interval in seconds
    #[arg(long, default_value = "10")]
    chaos_interval: u64,
    
    /// Maximum drift amount
    #[arg(long, default_value = "500")]
    max_drift: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    let args = Args::parse();
    
    info!("╔════════════════════════════════════════╗");
    info!("║     🔧 FabGitOps Mock PLC Server       ║");
    info!("╚════════════════════════════════════════╝");
    info!("");
    info!("Configuration:");
    info!("  Bind Address: {}:{}", args.bind, args.port);
    info!("  Register: {}", args.register);
    info!("  Initial Value: {}", args.value);
    info!("  Chaos Mode: {}", if args.chaos { "ENABLED" } else { "disabled" });
    if args.chaos {
        info!("  Chaos Interval: {}s", args.chaos_interval);
        info!("  Max Drift: {}", args.max_drift);
    }
    info!("");
    
    // Create shared state
    let state = Arc::new(Mutex::new(PLCState::new(args.value, args.register)));
    
    // Start chaos engine if enabled
    if args.chaos {
        let register_value = Arc::new(std::sync::Mutex::new(args.value));
        let chaos = ChaosEngine::new(ChaosConfig {
            enabled: true,
            interval_secs: args.chaos_interval,
            max_drift: args.max_drift,
        });
        chaos.spawn(register_value.clone());
        
        // Sync chaos value with server state
        let state_clone = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
            loop {
                interval.tick().await;
                if let Ok(chaos_val) = register_value.lock() {
                    if let Ok(mut server_state) = state_clone.lock() {
                        server_state.register_value = *chaos_val;
                    }
                }
            }
        });
    }
    
    // Start server
    start_server(&args.bind, args.port, state).await?;
    
    Ok(())
}
```

---

## 6. Phase 3: The fabctl CLI Tool

### 6.1 fabctl Cargo.toml

```toml
[package]
name = "fabctl"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "fabctl"
path = "src/main.rs"

[dependencies]
# CLI
clap = { version = "4.4", features = ["derive", "cargo"] }
colored = "2.1"
tabled = "0.15"
comfy-table = "7.1"

# Kubernetes
kube = { version = "0.87", features = ["derive"] }
k8s-openapi = { version = "0.20", features = ["v1_28"] }

# Async
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Error handling
anyhow = { workspace = true }

# Observability
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# Utilities
chrono = "0.4"
indicatif = "0.17"
```

### 6.2 Commands Module - `src/commands.rs`

```rust
use crate::k8s_client::K8sClient;
use crate::output::{print_plc_table, print_status_summary, StatusStyle};
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use tracing::info;

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
    let plcs = client.list_plcs(namespace).await?;
    
    let filtered: Vec<_> = if let Some(name) = name_filter {
        plcs.into_iter()
            .filter(|p| p.metadata.name.as_ref().map(|n| n.contains(name)).unwrap_or(false))
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
    
    println!("{}", "╔════════════════════════════════════════════════════════════╗".bright_blue());
    println!("{}", "║              Industrial PLC Resource Details               ║".bright_blue());
    println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_blue());
    println!();
    
    // Metadata
    println!("{}", "📋 Metadata:".bold().underline());
    println!("  Name:        {}", plc.metadata.name.as_deref().unwrap_or("N/A").cyan());
    println!("  Namespace:   {}", plc.metadata.namespace.as_deref().unwrap_or("N/A"));
    println!("  Created:     {}", plc.metadata.creation_timestamp.as_ref().map(|t| t.0.to_string()).unwrap_or_default());
    println!();
    
    // Spec
    println!("{}", "⚙️  Specification:".bold().underline());
    println!("  Device Address:  {}", plc.spec.device_address.cyan());
    println!("  Port:            {}", plc.spec.port);
    println!("  Target Register: {}", plc.spec.target_register);
    println!("  Target Value:    {}", plc.spec.target_value.to_string().green());
    println!("  Poll Interval:   {}s", plc.spec.poll_interval_secs);
    println!("  Auto Correct:    {}", if plc.spec.auto_correct { "✓ enabled".green() } else { "✗ disabled".red() });
    if !plc.spec.tags.is_empty() {
        println!("  Tags:            {}", plc.spec.tags.join(", "));
    }
    println!();
    
    // Status
    if let Some(status) = plc.status {
        let style = if status.in_sync {
            StatusStyle::Success
        } else if status.phase.to_string().contains("Drift") {
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
pub async fn cmd_sync(
    client: &K8sClient,
    namespace: &str,
    name: &str,
    force: bool,
) -> Result<()> {
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
    use crossterm::{cursor, terminal, ExecutableCommand};
    use std::io::stdout;
    
    println!("{}", "👁️  Watching PLC status (Ctrl+C to exit)...".cyan());
    println!();
    
    let mut stdout = stdout();
    
    loop {
        // Clear screen
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        stdout.execute(cursor::MoveTo(0, 0))?;
        
        // Print header
        println!("{}", "╔════════════════════════════════════════════════════════════╗".bright_blue());
        println!("{}", "║           FabGitOps Live Dashboard (fabctl watch)          ║".bright_blue());
        println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_blue());
        println!("  Namespace: {} | Refresh: {}s | Press Ctrl+C to exit", 
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
        println!("{}", format!("Last updated: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).dimmed());
        
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
        
        println!("{} {} @ {}:{}", 
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
    println!("{}", r#"
    ███████╗ █████╗ ██████╗  ██████╗ ██╗████████╗ ██████╗ ██████╗ ███████╗
    ██╔════╝██╔══██╗██╔══██╗██╔════╝ ██║╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝
    █████╗  ███████║██████╔╝██║  ███╗██║   ██║   ██║   ██║██████╔╝███████╗
    ██╔══╝  ██╔══██║██╔══██╗██║   ██║██║   ██║   ██║   ██║██╔═══╝ ╚════██║
    ██║     ██║  ██║██████╔╝╚██████╔╝██║   ██║   ╚██████╔╝██║     ███████║
    ╚═╝     ╚═╝  ╚═╝╚═════╝  ╚═════╝ ╚═╝   ╚═╝    ╚═════╝ ╚═╝     ╚══════╝
    "#.bright_cyan());
    
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
```

### 6.3 Output Module - `src/output.rs`

```rust
use operator::crd::{IndustrialPLC, PLCPhase};
use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};

pub enum StatusStyle {
    Success,
    Warning,
    Error,
    Neutral,
}

/// Print a beautiful ASCII table of PLC status
pub fn print_plc_table(plcs: &[IndustrialPLC]) {
    if plcs.is_empty() {
        println!("{}", "⚠️  No IndustrialPLC resources found".yellow());
        return;
    }
    
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("PLC Name").fg(Color::Cyan),
            Cell::new("Device").fg(Color::Cyan),
            Cell::new("Register").fg(Color::Cyan),
            Cell::new("Desired").fg(Color::Cyan),
            Cell::new("Actual").fg(Color::Cyan),
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Phase").fg(Color::Cyan),
            Cell::new("Drifts").fg(Color::Cyan),
        ]);
    
    for plc in plcs {
        let name = plc.metadata.name.as_deref().unwrap_or("unknown");
        let device = format!("{}:{}", plc.spec.device_address, plc.spec.port);
        let register = plc.spec.target_register.to_string();
        let desired = plc.spec.target_value.to_string();
        
        let (actual, status, phase, drifts) = if let Some(ref s) = plc.status {
            let actual_str = s.current_value.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
            
            let status_str = if s.in_sync {
                "✓ SYNCED".to_string()
            } else if s.phase == PLCPhase::DriftDetected {
                "⚠ DRIFT".to_string()
            } else {
                "✗ UNKNOWN".to_string()
            };
            
            (actual_str, status_str, format!("{:?}", s.phase), s.drift_events.to_string())
        } else {
            ("-".to_string(), "PENDING".to_string(), "Pending".to_string(), "0".to_string())
        };
        
        // Colorize status
        let status_cell = match status.as_str() {
            "✓ SYNCED" => Cell::new(status).fg(Color::Green),
            "⚠ DRIFT" => Cell::new(status).fg(Color::Yellow),
            _ => Cell::new(status).fg(Color::Red),
        };
        
        // Colorize phase
        let phase_cell = match phase.as_str() {
            "Connected" => Cell::new(phase).fg(Color::Green),
            "DriftDetected" => Cell::new(phase).fg(Color::Yellow),
            "Correcting" => Cell::new(phase).fg(Color::Blue),
            "Failed" => Cell::new(phase).fg(Color::Red),
            _ => Cell::new(phase).fg(Color::Grey),
        };
        
        table.add_row(vec![
            Cell::new(name),
            Cell::new(device),
            Cell::new(register),
            Cell::new(desired).fg(Color::Green),
            Cell::new(actual),
            status_cell,
            phase_cell,
            Cell::new(drifts),
        ]);
    }
    
    println!("{}", table);
}

/// Print a status summary box
pub fn print_status_summary(status: &operator::crd::IndustrialPLCStatus, style: StatusStyle) {
    let border_color = match style {
        StatusStyle::Success => Color::Green,
        StatusStyle::Warning => Color::Yellow,
        StatusStyle::Error => Color::Red,
        StatusStyle::Neutral => Color::Grey,
    };
    
    let status_icon = match style {
        StatusStyle::Success => "✓",
        StatusStyle::Warning => "⚠",
        StatusStyle::Error => "✗",
        StatusStyle::Neutral => "○",
    };
    
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    
    table.set_header(vec![Cell::new(format!("{} Status Summary", status_icon)).fg(border_color)]);
    
    table.add_row(vec![
        Cell::new("Phase:"),
        Cell::new(format!("{:?}", status.phase)).fg(border_color),
    ]);
    
    table.add_row(vec![
        Cell::new("In Sync:"),
        Cell::new(if status.in_sync { "Yes ✓" } else { "No ✗" })
            .fg(if status.in_sync { Color::Green } else { Color::Red }),
    ]);
    
    if let Some(value) = status.current_value {
        table.add_row(vec![
            Cell::new("Current Value:"),
            Cell::new(value.to_string()),
        ]);
    }
    
    table.add_row(vec![
        Cell::new("Drift Events:"),
        Cell::new(status.drift_events.to_string()),
    ]);
    
    table.add_row(vec![
        Cell::new("Corrections:"),
        Cell::new(status.corrections_applied.to_string()).fg(Color::Green),
    ]);
    
    if let Some(ref error) = status.last_error {
        table.add_row(vec![
            Cell::new("Last Error:"),
            Cell::new(error).fg(Color::Red),
        ]);
    }
    
    table.add_row(vec![
        Cell::new("Message:"),
        Cell::new(&status.message),
    ]);
    
    if let Some(ref updated) = status.last_update {
        table.add_row(vec![
            Cell::new("Last Update:"),
            Cell::new(updated).fg(Color::Grey),
        ]);
    }
    
    println!("{}", table);
}

/// Print a simple status line
pub fn print_status_line(plc: &IndustrialPLC) {
    let name = plc.metadata.name.as_deref().unwrap_or("unknown");
    
    if let Some(ref status) = plc.status {
        let emoji = if status.in_sync { "✓" } else { "✗" };
        let color = if status.in_sync { "green" } else { "red" };
        
        println!("{} {}: {} (phase: {:?})", 
            emoji,
            name,
            if status.in_sync { "SYNCED".color(color) } else { "DRIFT".color(color) },
            status.phase
        );
    } else {
        println!("○ {}: {}", name, "PENDING".dimmed());
    }
}
```

### 6.4 K8s Client Module - `src/k8s_client.rs`

```rust
use anyhow::{Context, Result};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use kube::api::{ListParams, Patch, PatchParams};
use kube::{Api, Client};
use operator::crd::IndustrialPLC;
use std::collections::BTreeMap;

/// Kubernetes client wrapper for FabGitOps operations
pub struct K8sClient {
    client: Client,
}

impl K8sClient {
    /// Create a new K8sClient from default configuration
    pub async fn new() -> Result<Self> {
        let client = Client::try_default()
            .await
            .context("Failed to create Kubernetes client")?;
        Ok(Self { client })
    }
    
    /// List all IndustrialPLC resources in a namespace
    pub async fn list_plcs(&self, namespace: &str) -> Result<Vec<IndustrialPLC>> {
        let api: Api<IndustrialPLC> = Api::namespaced(self.client.clone(), namespace);
        let plcs = api.list(&ListParams::default()).await?;
        Ok(plcs.items)
    }
    
    /// Get a specific IndustrialPLC resource
    pub async fn get_plc(&self, namespace: &str, name: &str) -> Result<IndustrialPLC> {
        let api: Api<IndustrialPLC> = Api::namespaced(self.client.clone(), namespace);
        let plc = api.get(name).await?;
        Ok(plc)
    }
    
    /// Trigger a reconciliation by annotating the resource
    pub async fn trigger_reconcile(&self, namespace: &str, name: &str, force: bool) -> Result<()> {
        let api: Api<IndustrialPLC> = Api::namespaced(self.client.clone(), namespace);
        
        let mut annotations: BTreeMap<String, String> = BTreeMap::new();
        annotations.insert(
            "fabgitops.io/last-sync-request".to_string(),
            Time(chrono::Utc::now()).0.to_rfc3339(),
        );
        
        if force {
            annotations.insert(
                "fabgitops.io/force-sync".to_string(),
                "true".to_string(),
            );
        }
        
        let patch = Patch::Merge(serde_json::json!({
            "metadata": {
                "annotations": annotations
            }
        }));
        
        api.patch(name, &PatchParams::default(), &patch).await?;
        
        Ok(())
    }
}
```

### 6.5 Main - `src/main.rs`

```rust
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
        Commands::Describe { name } => {
            cmd_describe(&client, &cli.namespace, name).await
        }
        Commands::Sync { name, force } => {
            cmd_sync(&client, &cli.namespace, name, *force).await
        }
        Commands::Watch { interval } => {
            cmd_watch(&client, &cli.namespace, *interval).await
        }
        Commands::List => {
            cmd_list(&client, &cli.namespace).await
        }
        Commands::Version => {
            cmd_version().await
        }
    };
    
    if let Err(ref e) = result {
        error!("{}", e);
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
    
    Ok(())
}

fn print_banner() {
    println!("{}", r#"
    ███████╗ █████╗ ██████╗  ██████╗ ██╗████████╗ ██████╗ ██████╗ ███████╗
    ██╔════╝██╔══██╗██╔══██╗██╔════╝ ██║╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝
    █████╗  ███████║██████╔╝██║  ███╗██║   ██║   ██║   ██║██████╔╝███████╗
    ██╔══╝  ██╔══██║██╔══██╗██║   ██║██║   ██║   ██║   ██║██╔═══╝ ╚════██║
    ██║     ██║  ██║██████╔╝╚██████╔╝██║   ██║   ╚██████╔╝██║     ███████║
    ╚═╝     ╚═╝  ╚═╝╚═════╝  ╚═════╝ ╚═╝   ╚═╝    ╚═════╝ ╚═╝     ╚══════╝
    "#.bright_cyan().dimmed());
}
```

---

## 7. Phase 4: The Observability Stack

### 7.1 docker-compose.yml

```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:v2.48.0
    container_name: fabgitops-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/usr/share/prometheus/console_libraries'
      - '--web.console.templates=/usr/share/prometheus/consoles'
      - '--web.enable-lifecycle'
    networks:
      - fabgitops-network

  grafana:
    image: grafana/grafana:10.2.0
    container_name: fabgitops-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_USER=admin
      - GF_SECURITY_ADMIN_PASSWORD=fabgitops
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./grafana/datasources:/etc/grafana/provisioning/datasources:ro
    depends_on:
      - prometheus
    networks:
      - fabgitops-network

  # Optional: Node exporter for host metrics
  node-exporter:
    image: prom/node-exporter:v1.7.0
    container_name: fabgitops-node-exporter
    ports:
      - "9100:9100"
    volumes:
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /:/rootfs:ro
    command:
      - '--path.procfs=/host/proc'
      - '--path.rootfs=/rootfs'
      - '--path.sysfs=/host/sys'
      - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
    networks:
      - fabgitops-network

volumes:
  prometheus-data:
  grafana-data:

networks:
  fabgitops-network:
    driver: bridge
```

### 7.2 prometheus.yml

```yaml
global:
  scrape_interval: 5s
  evaluation_interval: 5s

alerting:
  alertmanagers: []

rule_files: []

scrape_configs:
  # FabGitOps Operator metrics
  - job_name: 'fabgitops-operator'
    static_configs:
      - targets: ['host.docker.internal:8080']
    metrics_path: /metrics
    scrape_interval: 5s

  # Prometheus self-monitoring
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  # Node exporter (host metrics)
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
```

### 7.3 Grafana Datasource - `grafana/datasources/prometheus.yml`

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
```

### 7.4 Grafana Dashboard - `grafana/dashboards/fabgitops-dashboard.json`

```json
{
  "dashboard": {
    "id": null,
    "title": "FabGitOps - Industrial PLC Monitoring",
    "tags": ["fabgitops", "plc", "gitops"],
    "timezone": "browser",
    "schemaVersion": 38,
    "refresh": "5s",
    "panels": [
      {
        "id": 1,
        "title": "PLC Connection Status",
        "type": "stat",
        "targets": [
          {
            "expr": "plc_connection_status",
            "legendFormat": "{{instance}}"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "mappings": [
              {"options": {"0": {"text": "Disconnected", "color": "red"}}, "type": "value"},
              {"options": {"1": {"text": "Connected", "color": "green"}}, "type": "value"}
            ],
            "thresholds": {
              "steps": [{"color": "red", "value": 0}, {"color": "green", "value": 1}]
            }
          }
        },
        "gridPos": {"h": 4, "w": 6, "x": 0, "y": 0}
      },
      {
        "id": 2,
        "title": "Register Value",
        "type": "stat",
        "targets": [
          {
            "expr": "register_value",
            "legendFormat": "Current Value"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "none",
            "min": 0,
            "max": 65535
          }
        },
        "gridPos": {"h": 4, "w": 6, "x": 6, "y": 0}
      },
      {
        "id": 3,
        "title": "Managed PLCs",
        "type": "stat",
        "targets": [
          {
            "expr": "managed_plcs",
            "legendFormat": "Count"
          }
        ],
        "gridPos": {"h": 4, "w": 6, "x": 12, "y": 0}
      },
      {
        "id": 4,
        "title": "Reconciliation Duration",
        "type": "stat",
        "targets": [
          {
            "expr": "reconciliation_duration_seconds",
            "legendFormat": "Seconds"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "s"
          }
        },
        "gridPos": {"h": 4, "w": 6, "x": 18, "y": 0}
      },
      {
        "id": 5,
        "title": "Drift Events (Total)",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(drift_events_total[5m])",
            "legendFormat": "Drift Rate"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ops",
            "custom": {"drawStyle": "line", "lineWidth": 2}
          }
        },
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 4}
      },
      {
        "id": 6,
        "title": "Corrections Applied (Total)",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(corrections_total[5m])",
            "legendFormat": "Correction Rate"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ops",
            "custom": {"drawStyle": "line", "lineWidth": 2}
          }
        },
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 4}
      },
      {
        "id": 7,
        "title": "Register Value Over Time",
        "type": "graph",
        "targets": [
          {
            "expr": "register_value",
            "legendFormat": "Register Value"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "none",
            "min": 0,
            "max": 65535,
            "custom": {"drawStyle": "line", "lineWidth": 2}
          }
        },
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 12}
      },
      {
        "id": 8,
        "title": "System Overview",
        "type": "logs",
        "targets": [
          {
            "expr": "{job=\"fabgitops-operator\"}"
          }
        ],
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 20}
      }
    ]
  },
  "overwrite": true
}
```

---

## 8. Phase 5: Demo & Deployment

### 8.1 Kubernetes CRD - `k8s/crd.yaml`

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: industrialplcs.fabgitops.io
spec:
  group: fabgitops.io
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              required:
                - deviceAddress
                - targetRegister
                - targetValue
              properties:
                deviceAddress:
                  type: string
                  description: IP address or hostname of the PLC device
                port:
                  type: integer
                  default: 502
                  description: Port for Modbus TCP communication
                targetRegister:
                  type: integer
                  description: The Modbus register address to monitor/control
                targetValue:
                  type: integer
                  description: The desired value for the target register
                pollIntervalSecs:
                  type: integer
                  default: 5
                  description: Polling interval in seconds
                autoCorrect:
                  type: boolean
                  default: true
                  description: Enable automatic drift correction
                tags:
                  type: array
                  items:
                    type: string
                  description: Tags for categorization
            status:
              type: object
              properties:
                phase:
                  type: string
                  enum: ["Pending", "Connecting", "Connected", "DriftDetected", "Correcting", "Failed"]
                lastUpdate:
                  type: string
                  format: date-time
                currentValue:
                  type: integer
                inSync:
                  type: boolean
                driftEvents:
                  type: integer
                correctionsApplied:
                  type: integer
                lastError:
                  type: string
                message:
                  type: string
      subresources:
        status: {}
      additionalPrinterColumns:
        - name: Device
          type: string
          jsonPath: .spec.deviceAddress
        - name: Register
          type: integer
          jsonPath: .spec.targetRegister
        - name: Desired
          type: integer
          jsonPath: .spec.targetValue
        - name: Actual
          type: integer
          jsonPath: .status.currentValue
        - name: Status
          type: string
          jsonPath: .status.phase
        - name: Age
          type: date
          jsonPath: .metadata.creationTimestamp
  scope: Namespaced
  names:
    plural: industrialplcs
    singular: industrialplc
    kind: IndustrialPLC
    shortNames:
      - plc
```

### 8.2 RBAC - `k8s/rbac.yaml`

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: fabgitops-operator
  namespace: default
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: fabgitops-operator
rules:
  # Manage IndustrialPLC CRDs
  - apiGroups: ["fabgitops.io"]
    resources: ["industrialplcs"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  - apiGroups: ["fabgitops.io"]
    resources: ["industrialplcs/status"]
    verbs: ["get", "update", "patch"]
  - apiGroups: ["fabgitops.io"]
    resources: ["industrialplcs/finalizers"]
    verbs: ["update"]
  
  # Events
  - apiGroups: [""]
    resources: ["events"]
    verbs: ["create", "patch"]
  
  # ConfigMaps (for leader election)
  - apiGroups: [""]
    resources: ["configmaps"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  
  # Leases (for leader election)
  - apiGroups: ["coordination.k8s.io"]
    resources: ["leases"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: fabgitops-operator
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: fabgitops-operator
subjects:
  - kind: ServiceAccount
    name: fabgitops-operator
    namespace: default
```

### 8.3 Operator Deployment - `k8s/deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fabgitops-operator
  namespace: default
  labels:
    app: fabgitops-operator
spec:
  replicas: 1
  selector:
    matchLabels:
      app: fabgitops-operator
  template:
    metadata:
      labels:
        app: fabgitops-operator
    spec:
      serviceAccountName: fabgitops-operator
      containers:
        - name: operator
          image: fabgitops-operator:latest
          imagePullPolicy: IfNotPresent
          ports:
            - name: metrics
              containerPort: 8080
              protocol: TCP
          env:
            - name: RUST_LOG
              value: "info"
            - name: HOSTNAME
              valueFrom:
                fieldRef:
                  fieldPath: spec.nodeName
          resources:
            requests:
              memory: "64Mi"
              cpu: "100m"
            limits:
              memory: "256Mi"
              cpu: "500m"
          livenessProbe:
            httpGet:
              path: /health
              port: metrics
            initialDelaySeconds: 10
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health
              port: metrics
            initialDelaySeconds: 5
            periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: fabgitops-operator-metrics
  namespace: default
  labels:
    app: fabgitops-operator
spec:
  type: ClusterIP
  ports:
    - port: 8080
      targetPort: metrics
      protocol: TCP
      name: metrics
  selector:
    app: fabgitops-operator
```

### 8.4 Sample PLC - `k8s/sample-plc.yaml`

```yaml
apiVersion: fabgitops.io/v1
kind: IndustrialPLC
metadata:
  name: production-line-1
  namespace: default
  labels:
    environment: production
    line: assembly-1
spec:
  deviceAddress: mock-plc.default.svc.cluster.local
  port: 5502
  targetRegister: 4001
  targetValue: 2500
  pollIntervalSecs: 5
  autoCorrect: true
  tags:
    - production
    - assembly
    - critical
---
apiVersion: fabgitops.io/v1
kind: IndustrialPLC
metadata:
  name: test-plc-local
  namespace: default
  labels:
    environment: development
spec:
  deviceAddress: host.docker.internal
  port: 5502
  targetRegister: 4001
  targetValue: 3000
  pollIntervalSecs: 3
  autoCorrect: true
  tags:
    - development
    - testing
```

### 8.5 Demo Script - `demo.sh`

```bash
#!/bin/bash

# FabGitOps Demo Script
# This script demonstrates the complete GitOps workflow

set -e

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║              🏭 FabGitOps - GitOps for Industrial PLCs             ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_section() {
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

print_step() {
    echo -e "${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Check prerequisites
print_section "Prerequisites Check"

commands=("kubectl" "docker" "cargo")
for cmd in "${commands[@]}"; do
    if command -v $cmd &> /dev/null; then
        print_success "$cmd is installed"
    else
        echo -e "${RED}✗ $cmd is not installed${NC}"
        exit 1
    fi
done

# Step 1: Start Observability Stack
print_section "Step 1: Starting Observability Stack"
print_step "Starting Prometheus and Grafana..."
docker-compose up -d
print_success "Observability stack started"
echo "  📊 Grafana: http://localhost:3000 (admin/fabgitops)"
echo "  📈 Prometheus: http://localhost:9090"

# Step 2: Build the project
print_section "Step 2: Building FabGitOps"
print_step "Building Rust workspace..."
cargo build --release
print_success "Build complete"

# Step 3: Start Mock PLC (Chaos Mode)
print_section "Step 3: Starting Mock PLC (Chaos Mode)"
print_step "Starting mock PLC with chaos mode enabled..."
./target/release/mock-plc \
    --bind 0.0.0.0 \
    --port 5502 \
    --value 2500 \
    --register 4001 \
    --chaos \
    --chaos-interval 10 \
    --max-drift 500 &
MOCK_PID=$!
print_success "Mock PLC started (PID: $MOCK_PID)"
sleep 2

# Step 4: Deploy to Kubernetes
print_section "Step 4: Deploying to Kubernetes"
print_step "Applying CRD..."
kubectl apply -f k8s/crd.yaml
print_success "CRD applied"

print_step "Applying RBAC..."
kubectl apply -f k8s/rbac.yaml
print_success "RBAC applied"

print_step "Building operator Docker image..."
docker build -t fabgitops-operator:latest -f Dockerfile.operator .
print_success "Docker image built"

print_step "Loading image into kind/minikube..."
kind load docker-image fabgitops-operator:latest 2>/dev/null || \
    minikube image load fabgitops-operator:latest 2>/dev/null || \
    print_warning "Could not load image - ensure cluster is running"

print_step "Deploying operator..."
kubectl apply -f k8s/deployment.yaml
print_success "Operator deployed"

print_step "Creating sample PLC resources..."
kubectl apply -f k8s/sample-plc.yaml
print_success "Sample PLCs created"

# Wait for operator to be ready
print_step "Waiting for operator to be ready..."
kubectl wait --for=condition=available --timeout=60s deployment/fabgitops-operator
print_success "Operator is ready"

# Step 5: Demo fabctl commands
print_section "Step 5: fabctl CLI Demo"

echo ""
echo -e "${YELLOW}Press Enter to run: fabctl version${NC}"
read
./target/release/fabctl version

echo ""
echo -e "${YELLOW}Press Enter to run: fabctl list${NC}"
read
./target/release/fabctl list

echo ""
echo -e "${YELLOW}Press Enter to run: fabctl get-status${NC}"
read
./target/release/fabctl get-status

echo ""
echo -e "${YELLOW}Press Enter to run: fabctl describe production-line-1${NC}"
read
./target/release/fabctl describe production-line-1

# Step 6: Live Dashboard
print_section "Step 6: Live Terminal Dashboard"
print_step "Starting live watch (Press Ctrl+C to continue)..."
echo ""
sleep 2
./target/release/fabctl watch --interval 2 &
WATCH_PID=$!
sleep 10
kill $WATCH_PID 2>/dev/null || true

# Step 7: Manual Sync Demo
print_section "Step 7: Manual Sync Demo"
print_step "Triggering manual sync..."
./target/release/fabctl sync production-line-1 --force

# Step 8: Show Grafana
print_section "Step 8: Grafana Dashboard"
echo "Open http://localhost:3000 in your browser"
echo "Login: admin / fabgitops"
echo ""
echo -e "${YELLOW}Press Enter when ready to cleanup...${NC}"
read

# Cleanup
print_section "Cleanup"
print_step "Stopping mock PLC..."
kill $MOCK_PID 2>/dev/null || true
print_success "Mock PLC stopped"

print_step "Stopping observability stack..."
docker-compose down
print_success "Observability stack stopped"

print_step "Deleting Kubernetes resources..."
kubectl delete -f k8s/sample-plc.yaml --ignore-not-found
kubectl delete -f k8s/deployment.yaml --ignore-not-found
kubectl delete -f k8s/rbac.yaml --ignore-not-found
kubectl delete -f k8s/crd.yaml --ignore-not-found
print_success "Kubernetes resources deleted"

print_section "Demo Complete!"
echo -e "${GREEN}Thank you for trying FabGitOps!${NC}"
echo ""
echo "Key takeaways:"
echo "  ✓ CLI-first interface (fabctl)"
echo "  ✓ Prometheus metrics exposed"
echo "  ✓ Automatic drift detection & correction"
echo "  ✓ Chaos mode for testing"
echo "  ✓ GitOps principles applied to industrial PLCs"
```

### 8.6 Dockerfile - `Dockerfile.operator`

```dockerfile
# Build stage
FROM rust:1.75-slim AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/operator/Cargo.toml ./crates/operator/
COPY crates/operator/src ./crates/operator/src/

# Build the operator
RUN cargo build --release -p operator

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/operator /usr/local/bin/fabgitops-operator

# Expose metrics port
EXPOSE 8080

# Run the operator
ENTRYPOINT ["fabgitops-operator"]
```

---

## 9. Interview Talking Points

### The "Terminal Moment"

When presenting FabGitOps in an interview, open your terminal and run:

```bash
# 1. Show the live dashboard
$ watch -n 1 fabctl get-status

# 2. Show detailed status
$ fabctl describe production-line-1

# 3. Trigger a manual sync
$ fabctl sync production-line-1 --force

# 4. Show the chaos in action
$ fabctl watch --interval 1
```

### Key Talking Points

1. **"I built internal tooling"**
   - "fabctl is a developer-grade CLI, not a web UI. This proves I can build tools for other engineers."

2. **"I understand observability standards"**
   - "Instead of custom JavaScript charts, I expose Prometheus metrics. This is what production systems use."

3. **"I applied GitOps to the physical world"**
   - "The same principles that manage Kubernetes deployments now manage industrial hardware."

4. **"I can handle the full stack"**
   - "From Rust systems programming to Kubernetes operators to CLI design - I've covered it all."

### Architecture Questions You Should Be Ready For

| Question | Your Answer |
|----------|-------------|
| "Why Rust?" | "Memory safety without GC, excellent async support, native Kubernetes SDK (kube-rs)" |
| "Why CLI over Web UI?" | "DevOps engineers live in terminals. CLI proves I understand the user." |
| "How do you handle failures?" | "Exponential backoff, event recording, status subresource for visibility" |
| "What about scaling?" | "Controller pattern scales horizontally, metrics help identify bottlenecks" |

---

## Quick Start Commands

```bash
# 1. Clone and build
git clone <repo>
cd fabgitops
cargo build --release

# 2. Start observability
docker-compose up -d

# 3. Start mock PLC with chaos
./target/release/mock-plc --chaos --chaos-interval 10

# 4. Deploy to Kubernetes
kubectl apply -f k8s/

# 5. Use fabctl
./target/release/fabctl get-status
./target/release/fabctl watch

# 6. View Grafana
open http://localhost:3000  # admin/fabgitops
```

---

*FabGitOps - The Infrastructure Layer of Your Industrial Cloud Stack*
