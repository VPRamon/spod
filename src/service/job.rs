//! Application-level run and job contracts.

use super::artifacts::Artifacts;
use super::config::RunConfig;
use super::error::ServiceError;
use super::provenance::RunProvenance;
use super::workflow::Workflow;
use serde::{Deserialize, Serialize};
use siderust::astro::dynamics::OrbitState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Stable identifier assigned by a caller to a run.
pub type JobId = String;

/// Stable lifecycle state for a submitted service job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    /// The service accepted the job but has not started execution.
    Pending,
    /// The runner is executing the job.
    Running,
    /// The runner completed successfully.
    Succeeded,
    /// The runner completed with a categorized failure.
    Failed,
}

/// Stable, machine-readable failure exposed by job APIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobError {
    /// Stable category/code for programmatic handling.
    pub code: JobErrorCode,
    /// Human-readable diagnostic, without implementation-specific details.
    pub message: String,
}

/// Categories of failures that can be returned by a job API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobErrorCode {
    /// Configuration could not be parsed or validated.
    Configuration,
    /// An input could not be resolved.
    Input,
    /// The requested workflow or input is unsupported.
    Unsupported,
    /// An output artifact could not be written or finalized.
    Artifact,
    /// Scientific execution failed.
    Scientific,
    /// An unexpected service failure occurred.
    Internal,
}

impl From<&ServiceError> for JobError {
    fn from(error: &ServiceError) -> Self {
        let code = match error {
            ServiceError::Configuration { .. } => JobErrorCode::Configuration,
            ServiceError::Unsupported { .. } => JobErrorCode::Unsupported,
            ServiceError::Input { .. } => JobErrorCode::Input,
            ServiceError::Artifact { .. } => JobErrorCode::Artifact,
            ServiceError::Scientific { .. } => JobErrorCode::Scientific,
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

/// Public result summary attached to a succeeded job.
#[derive(Debug, Clone, Serialize)]
pub struct JobResult {
    /// Identifier of the completed run.
    pub run_id: JobId,
    /// Workflow that produced the result.
    pub workflow: Workflow,
    /// Generated files and their hashes.
    pub artifacts: Artifacts,
}

impl From<&RunResult> for JobResult {
    fn from(result: &RunResult) -> Self {
        Self {
            run_id: result.run_id.clone(),
            workflow: result.workflow,
            artifacts: result.artifacts.clone(),
        }
    }
}

/// Canonical status snapshot shared by CLI and REST consumers.
#[derive(Debug, Clone, Serialize)]
pub struct JobStatus {
    /// Stable service-assigned or caller-provided identifier.
    pub id: JobId,
    /// Current lifecycle state.
    pub state: JobState,
    /// Workflow selected for this job.
    pub workflow: Workflow,
    /// Present after successful execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JobResult>,
    /// Present after failed execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JobError>,
}

impl JobStatus {
    /// Construct a newly accepted job.
    pub fn pending(id: JobId, workflow: Workflow) -> Self {
        Self {
            id,
            state: JobState::Pending,
            workflow,
            result: None,
            error: None,
        }
    }

    /// Advance a pending job to execution.
    pub fn start(&mut self) -> Result<(), JobTransitionError> {
        if self.state != JobState::Pending {
            return Err(JobTransitionError {
                from: self.state,
                to: JobState::Running,
            });
        }
        self.state = JobState::Running;
        Ok(())
    }

    /// Complete a running job successfully.
    pub fn succeed(&mut self, result: JobResult) -> Result<(), JobTransitionError> {
        self.require_running(JobState::Succeeded)?;
        self.state = JobState::Succeeded;
        self.result = Some(result);
        self.error = None;
        Ok(())
    }

    /// Complete a running job with a categorized failure.
    pub fn fail(&mut self, error: JobError) -> Result<(), JobTransitionError> {
        self.require_running(JobState::Failed)?;
        self.state = JobState::Failed;
        self.result = None;
        self.error = Some(error);
        Ok(())
    }

    fn require_running(&self, to: JobState) -> Result<(), JobTransitionError> {
        if self.state != JobState::Running {
            return Err(JobTransitionError {
                from: self.state,
                to,
            });
        }
        Ok(())
    }
}

impl JobStatus {
    /// Construct a completed status from a synchronous runner result.
    pub fn from_run(
        id: JobId,
        workflow: Workflow,
        result: Result<RunResult, ServiceError>,
    ) -> Self {
        let mut status = Self::pending(id, workflow);
        status
            .start()
            .expect("newly constructed job must transition to running");
        match result {
            Ok(result) => {
                status
                    .succeed(JobResult::from(&result))
                    .expect("running job must transition to succeeded");
            }
            Err(error) => {
                status
                    .fail(JobError::from(&error))
                    .expect("running job must transition to failed");
            }
        }
        status
    }
}

/// An attempted lifecycle transition that is not permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobTransitionError {
    /// State before the attempted transition.
    pub from: JobState,
    /// Requested destination state.
    pub to: JobState,
}

/// Small synchronized store for the current in-memory job snapshots.
#[derive(Clone, Default)]
pub struct JobStore {
    jobs: Arc<Mutex<HashMap<JobId, JobStatus>>>,
}

impl JobStore {
    /// Insert an accepted job.
    pub fn insert(&self, status: JobStatus) {
        self.with_jobs(|jobs| {
            jobs.insert(status.id.clone(), status);
        });
    }

    /// Read a status snapshot, if the identifier is known.
    pub fn get(&self, id: &str) -> Option<JobStatus> {
        self.with_jobs(|jobs| jobs.get(id).cloned())
    }

    /// Apply one explicit transition while holding the store lock.
    pub fn transition(
        &self,
        id: &str,
        transition: impl FnOnce(&mut JobStatus) -> Result<(), JobTransitionError>,
    ) -> Result<(), JobStoreError> {
        self.with_jobs(|jobs| {
            let status = jobs.get_mut(id).ok_or(JobStoreError::UnknownJob)?;
            transition(status).map_err(JobStoreError::InvalidTransition)
        })
    }

    fn with_jobs<T>(&self, operation: impl FnOnce(&mut HashMap<JobId, JobStatus>) -> T) -> T {
        let mut jobs = self
            .jobs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        operation(&mut jobs)
    }
}

/// Failure while updating the job store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStoreError {
    /// No job exists for the requested identifier.
    UnknownJob,
    /// The requested state transition is invalid.
    InvalidTransition(JobTransitionError),
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_allows_success_only_from_running() {
        let mut status = JobStatus::pending("job-1".into(), Workflow::Synthetic);
        assert!(status.start().is_ok());
        assert_eq!(status.state, JobState::Running);
        let artifact = |path: &str, kind: &str| {
            siderust::pod::run::dataset::DatasetRef::from_bytes(path, kind, b"x")
        };
        let result = JobResult {
            run_id: "job-1".into(),
            workflow: Workflow::Synthetic,
            artifacts: Artifacts {
                orbit_sp3: artifact("orbit.sp3", "orbit-sp3"),
                orbit_oem: artifact("orbit.oem", "orbit-oem"),
                residuals_csv: artifact("residuals.csv", "residuals"),
                qc_json: artifact("qc.json", "qc"),
                manifest: artifact("manifest.json", "manifest"),
            },
        };
        assert!(status.succeed(result).is_ok());
        assert_eq!(status.state, JobState::Succeeded);
        assert!(status.result.is_some());
        assert!(status.error.is_none());
        assert_eq!(serde_json::to_value(status).unwrap()["state"], "succeeded");
    }

    #[test]
    fn lifecycle_rejects_invalid_transitions() {
        let mut status = JobStatus::pending("job-1".into(), Workflow::Synthetic);
        assert!(status.start().is_ok());
        assert!(status.start().is_err());
        assert!(status
            .fail(JobError {
                code: JobErrorCode::Scientific,
                message: "failed".into(),
            })
            .is_ok());
        assert_eq!(status.state, JobState::Failed);
        assert!(status.result.is_none());
        assert!(status.error.is_some());
        assert!(status.start().is_err());
    }

    #[test]
    fn failed_terminal_state_has_error_and_stable_state_name() {
        let mut status = JobStatus::pending("job-2".into(), Workflow::Synthetic);
        status.start().unwrap();
        status
            .fail(JobError {
                code: JobErrorCode::Unsupported,
                message: "unsupported input".into(),
            })
            .unwrap();

        assert_eq!(status.state, JobState::Failed);
        assert!(status.result.is_none());
        assert_eq!(
            status.error.as_ref().unwrap().code,
            JobErrorCode::Unsupported
        );
        assert_eq!(serde_json::to_value(status).unwrap()["state"], "failed");
    }

    #[test]
    fn service_errors_map_to_stable_codes() {
        let error = JobError::from(&ServiceError::unsupported("not available"));
        assert_eq!(error.code, JobErrorCode::Unsupported);
        assert_eq!(serde_json::to_value(error).unwrap()["code"], "unsupported");
    }
}
