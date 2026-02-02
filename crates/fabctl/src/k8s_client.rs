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
            annotations.insert("fabgitops.io/force-sync".to_string(), "true".to_string());
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
