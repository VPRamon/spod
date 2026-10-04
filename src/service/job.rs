//! Application-level run requests and results.

use super::artifacts::Artifacts;
use super::config::RunConfig;
use super::provenance::RunProvenance;
use super::workflow::Workflow;
use siderust::astro::dynamics::OrbitState;
use std::path::PathBuf;

/// Stable identifier assigned by a caller to a run.
pub type JobId = String;

/// Inputs to the high-level service runner.
#[derive(Debug, Clone)]
pub struct RunRequest {
    /// Parsed run configuration.
    pub config: RunConfig,
    /// Source YAML file used to establish configuration provenance.
    pub config_path: PathBuf,
}

/// Structured outcome of a completed service run.
#[derive(Debug)]
pub struct RunResult {
    /// Identifier from the run configuration.
    pub run_id: JobId,
    /// Workflow that executed.
    pub workflow: Workflow,
    /// Final estimated state.
    pub final_state: OrbitState,
    /// Number of integration steps.
    pub n_steps: usize,
    /// Number of estimator iterations.
    pub estimator_iterations: usize,
    /// Final reduced chi-squared statistic.
    pub reduced_chi2: f64,
    /// Generated files and their hashes.
    pub artifacts: Artifacts,
    /// Inputs and configuration identity used by the run.
    pub provenance: RunProvenance,
}

/// Application result name retained for callers of the previous service API.
pub type RunReport = RunResult;
