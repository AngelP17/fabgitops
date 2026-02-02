use crate::crd::{IndustrialPLC, IndustrialPLCStatus};
use crate::metrics::OperatorMetrics;
use crate::plc_client::PLCClient;
use kube::api::{Api, Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::events::{Event, EventType, Recorder, Reporter};
use kube::{Client, Resource, ResourceExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tracing::{error, info};

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

    // Update managed PLCs count
    let all_plcs = Api::<IndustrialPLC>::all(ctx.client.clone());
    if let Ok(plc_list) = all_plcs.list(&Default::default()).await {
        ctx.metrics.set_managed_plcs(plc_list.items.len() as i64);
    }

    // Create PLC client
    let plc_client = PLCClient::new(&plc.spec.device_address, plc.spec.port);

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
                            plc.spec.target_register, plc.spec.target_value, current_value
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
                                        plc.spec.target_register, plc.spec.target_value
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
    Ok(Action::requeue(Duration::from_secs(
        plc.spec.poll_interval_secs,
    )))
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
