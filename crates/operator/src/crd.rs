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
        self.message = format!("DRIFT DETECTED! Desired: {}, Actual: {}", desired, actual);
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
