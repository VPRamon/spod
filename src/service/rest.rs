//! # REST interface for POD jobs
//!
//! ## Scientific scope
//!
//! This module exposes a minimal HTTP surface for launching and querying
//! POD jobs. It does not change the scientific content of the underlying
//! pipeline; it only presents that pipeline through a network-facing
//! control layer suited to local automation and integration tests.
//!
//! The current regime is intentionally narrow: in-memory job state,
//! synthetic job execution, and JSON status payloads. It is not a
//! distributed scheduler or a general production API.
//!
//! ## Technical scope
//!
//! The public items are `JobStatus`, `AppState`, and `router`. The router
//! exposes health and job-submission endpoints, while the worker path
//! delegates actual arc generation and estimation to `spod::service`.
//!
//! Authentication, persistent storage, and arbitrary user-supplied datasets
//! are out of scope.
//!
//! ## References
//!
//! - Fielding, R., Nottingham, M., & Reschke, J. (2022). HTTP Semantics.
//!   RFC 9110.
//! - Bray, T. (2017). The JavaScript Object Notation (JSON) Data
//!   Interchange Format. RFC 8259.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path as FsPath, PathBuf};
use std::sync::{Arc, Mutex};

use crate::service::{ForcesConfig, InputsConfig, RunConfig, RunRequest, Runner, Workflow};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use siderust::pod::run::dataset::DatasetRef;
use uuid::Uuid;

/// Job status snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum JobStatus {
    /// Job has been accepted but not yet started.
    Pending,
    /// Job is currently running.
    Running,
    /// Job finished successfully.
    Finished {
        /// Path to the run manifest.
        manifest_path: String,
    },
    /// Job failed.
    Failed {
        /// Error message returned by the worker.
        error: String,
    },
}

/// Shared application state (in-memory job table).
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<HashMap<String, JobStatus>>>,
    /// Base output directory for produced artefacts.
    pub output_root: Arc<PathBuf>,
}

impl AppState {
    /// Create a new state rooted at the given output directory.
    pub fn new(output_root: PathBuf) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            output_root: Arc::new(output_root),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct JobRequest {
    #[serde(default)]
    enable_j2: bool,
}

#[derive(Debug, Serialize)]
struct JobAccepted {
    id: String,
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn submit_job(
    State(state): State<AppState>,
    body: Option<Json<JobRequest>>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let req = body.map(|Json(r)| r).unwrap_or_default();
    {
        let mut t = state.inner.lock().unwrap();
        t.insert(id.clone(), JobStatus::Pending);
    }
    let id_clone = id.clone();
    let state_clone = state.clone();
    tokio::task::spawn_blocking(move || {
        run_job(state_clone, id_clone, req);
    });
    (StatusCode::ACCEPTED, Json(JobAccepted { id }))
}

async fn job_status(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let table = state.inner.lock().unwrap();
    match table.get(&id) {
        Some(s) => (StatusCode::OK, Json(s.clone())).into_response(),
        None => (StatusCode::NOT_FOUND, "unknown job id").into_response(),
    }
}

fn persist_request(output_dir: &FsPath, req: &JobRequest) -> std::io::Result<DatasetRef> {
    fs::create_dir_all(output_dir)?;
    let request_path = output_dir.join("request.json");
    let request_bytes = serde_json::to_vec(req).map_err(std::io::Error::other)?;
    fs::write(&request_path, request_bytes)?;
    DatasetRef::from_file(request_path, "configuration")
}

fn fail_job(state: &AppState, id: &str, error: impl ToString) {
    let mut table = state.inner.lock().unwrap();
    table.insert(
        id.to_owned(),
        JobStatus::Failed {
            error: error.to_string(),
        },
    );
}

fn run_job(state: AppState, id: String, req: JobRequest) {
    {
        let mut t = state.inner.lock().unwrap();
        t.insert(id.clone(), JobStatus::Running);
    }
    let out = state.output_root.join(&id);
    let request_ref = match persist_request(&out, &req) {
        Ok(request_ref) => request_ref,
        Err(error) => {
            fail_job(&state, &id, error);
            return;
        }
    };
    let config = RunConfig {
        schema_version: "1.0.0".into(),
        run_id: id.clone(),
        workflow: Workflow::Synthetic,
        inputs: InputsConfig {
            sp3: None,
            rinex_obs: None,
            rinex_nav: None,
            antex: None,
        },
        output_dir: out.display().to_string(),
        forces: ForcesConfig {
            two_body: true,
            j2: req.enable_j2,
            third_body: false,
        },
    };
    let result = Runner.run(RunRequest {
        config,
        config_path: request_ref.path,
    });
    let mut t = state.inner.lock().unwrap();
    match result {
        Ok(report) => {
            t.insert(
                id,
                JobStatus::Finished {
                    manifest_path: report.artifacts.manifest.path.display().to_string(),
                },
            );
        }
        Err(e) => {
            t.insert(
                id,
                JobStatus::Failed {
                    error: e.to_string(),
                },
            );
        }
    }
}

/// Build the axum router with the given application state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/jobs", post(submit_job))
        .route("/jobs/:id", get(job_status))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn persisted_request_is_a_real_hashed_manifest_input() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "spod-rest-request-test-{}-{suffix}",
            std::process::id()
        ));
        let req = JobRequest { enable_j2: true };

        let dataset = persist_request(&root, &req).unwrap();
        let request_path = root.join("request.json");

        assert_eq!(dataset.path, request_path);
        assert_eq!(dataset.kind, "configuration");
        assert!(dataset.path.is_file());

        let actual = DatasetRef::from_file(&dataset.path, "configuration").unwrap();
        assert_eq!(dataset.bytes, actual.bytes);
        assert_eq!(dataset.sha256, actual.sha256);

        let persisted: JobRequest =
            serde_json::from_slice(&fs::read(&dataset.path).unwrap()).unwrap();
        assert_eq!(persisted.enable_j2, req.enable_j2);

        fs::remove_dir_all(root).unwrap();
    }
}
