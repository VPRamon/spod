//! Stable YAML configuration for service runs.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::error::ServiceError;
use super::workflow::Workflow;

/// Top-level run configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunConfig {
    /// Schema version. Must be `"1.0.0"` for the current service API.
    pub schema_version: String,
    /// Free-form run identifier.
    pub run_id: String,
    /// Workflow selected for this run.
    #[serde(default)]
    pub workflow: Workflow,
    /// Input datasets. Real-data inputs are currently unsupported.
    pub inputs: InputsConfig,
    /// Output directory, absolute or relative to the configuration file.
    pub output_dir: String,
    /// Force-model toggles supported by the selected workflow.
    pub forces: ForcesConfig,
}

/// Input file paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputsConfig {
    /// Precise GNSS satellite ephemeris (SP3).
    pub sp3: Option<String>,
    /// RINEX observation file.
    pub rinex_obs: Option<String>,
    /// RINEX broadcast navigation.
    pub rinex_nav: Option<String>,
    /// ANTEX antenna file.
    pub antex: Option<String>,
}

/// Which forces are enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForcesConfig {
    /// Two-body central gravity.
    pub two_body: bool,
    /// J2 oblateness.
    pub j2: bool,
    /// Sun + Moon third-body gravity, currently unsupported.
    pub third_body: bool,
}

impl RunConfig {
    /// Load and parse a YAML configuration.
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self, ServiceError> {
        let text = std::fs::read_to_string(path).map_err(ServiceError::input)?;
        serde_yaml::from_str(&text).map_err(|e| ServiceError::configuration(e.to_string()))
    }

    /// Validate semantic invariants before scientific execution.
    pub fn validate(&self) -> Result<(), ServiceError> {
        if self.schema_version != "1.0.0" {
            return Err(ServiceError::configuration(format!(
                "unsupported schema_version: {}",
                self.schema_version
            )));
        }
        if self.run_id.trim().is_empty() {
            return Err(ServiceError::configuration("run_id must not be empty"));
        }
        if !self.forces.two_body {
            return Err(ServiceError::configuration(
                "two_body force must be enabled",
            ));
        }
        if self.forces.third_body {
            return Err(ServiceError::unsupported(
                "third_body force is not implemented in the current synthetic-only service",
            ));
        }
        if self.workflow != Workflow::Synthetic {
            return Err(ServiceError::unsupported(format!(
                "workflow `{}` is not implemented",
                self.workflow
            )));
        }
        Ok(())
    }

    /// Resolve the configured output directory relative to the configuration file.
    pub fn output_dir_relative_to(&self, config_path: impl AsRef<Path>) -> PathBuf {
        let output = PathBuf::from(&self.output_dir);
        if output.is_absolute() {
            output
        } else {
            config_path
                .as_ref()
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(output)
        }
    }
}
