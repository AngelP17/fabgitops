use prometheus::{Counter, Gauge, Registry, Opts};

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
