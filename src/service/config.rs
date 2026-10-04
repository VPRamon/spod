//! # Run configuration schema
//!
//! ## Scientific scope
//!
//! A POD run needs a stable description of its input files, selected force
//! toggles, and output location before any scientific computation begins.
//! This module defines that configuration envelope for the current MVP
//! service path.
//!
//! The schema is intentionally rigid so integration tests can rely on
//! deterministic behaviour. Scientific interpretation enters later, when
//! the selected inputs and force switches are bound into an actual
//! estimation run.
//!
//! ## Technical scope
//!
//! The main public types are `RunConfig`, `InputsConfig`, and
//! `ForcesConfig`, along with `RunConfig::from_yaml_file` and
//! `RunConfig::validate`. The schema uses filesystem paths and booleans
//! rather than typed orbit quantities because it describes workflow wiring
//! rather than physical state.
//!
//! It does not execute the pipeline or hash artifacts; those
//! responsibilities belong to the runner and manifest modules.
//!
//! ## References
//!
//! - Ben-Kiki, O., Evans, C., & d'Otremont, I. (2021). YAML Ain't Markup
//!   Language (YAML) Version 1.2.2.
//! - Bray, T. (2017). The JavaScript Object Notation (JSON) Data
//!   Interchange Format. RFC 8259.
use serde::{Deserialize, Serialize};

/// Top-level run configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    /// Schema version (semver-ish). Must be `"1.0.0"` for MVP-1.
    pub schema_version: String,
    /// Free-form run identifier.
    pub run_id: String,
    /// Inputs.
    pub inputs: InputsConfig,
    /// Output directory (absolute or relative to the config file).
    pub output_dir: String,
    /// Force-model toggles.
    pub forces: ForcesConfig,
}

/// Input file paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct ForcesConfig {
    /// Two-body central gravity.
    pub two_body: bool,
    /// J2 oblateness.
    pub j2: bool,
    /// Sun + Moon third-body gravity. Currently rejected by the synthetic-only runner.
    pub third_body: bool,
}

impl RunConfig {
    /// Load from a YAML file.
    pub fn from_yaml_file(path: &str) -> Result<Self, std::io::Error> {
        let text = std::fs::read_to_string(path)?;
        serde_yaml::from_str(&text)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }

    /// Validate semantic invariants (paths exist, schema version known, …).
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != "1.0.0" {
            return Err(format!(
                "unsupported schema_version: {}",
                self.schema_version
            ));
        }
        if !(self.forces.two_body) {
            return Err("two_body force must be enabled".into());
        }
        if self.forces.third_body {
            return Err(
                "third_body force is not implemented in the current synthetic-only service".into(),
            );
        }
        Ok(())
    }
}
