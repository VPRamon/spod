//! Service-owned provenance records.

use serde::Serialize;
use siderust::pod::run::dataset::DatasetRef;

/// Inputs and identity recorded for a run.
#[derive(Debug, Clone, Serialize)]
pub struct RunProvenance {
    /// SHA-256 of the configuration document.
    pub config_sha256: String,
    /// Configuration and external datasets consumed by the run.
    pub inputs: Vec<DatasetRef>,
}

impl RunProvenance {
    /// Build provenance from the configuration dataset and other inputs.
    pub fn from_config(config: DatasetRef, inputs: Vec<DatasetRef>) -> Self {
        let config_sha256 = config.sha256.clone();
        let mut all_inputs = Vec::with_capacity(inputs.len() + 1);
        all_inputs.push(config);
        all_inputs.extend(inputs);
        Self {
            config_sha256,
            inputs: all_inputs,
        }
    }
}
