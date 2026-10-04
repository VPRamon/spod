//! # REST adapter for POD jobs
//!
//! REST is an adapter over the service-owned [`JobStatus`] model. It owns
//! request parsing, HTTP status mapping, and the temporary in-memory store,
//! but not lifecycle or scientific execution semantics.
//!
//! The API is intentionally local and experimental: persistence,
//! authentication, and scheduling are deferred to later issues.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fs;
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;

use crate::service::{
    ForcesConfig, InputsConfig, JobError, JobStatus, JobStore, RunConfig, RunRequest, Runner,
    Workflow,
};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use siderust::pod::run::dataset::DatasetRef;
use uuid::Uuid;

/// Shared application state for the REST adapter.
#[derive(Clone)]
pub struct AppState {
    store: JobStore,
    /// Base output directory for produced artefacts.
    pub output_root: Arc<PathBuf>,
}

impl AppState {
    /// Create a new state rooted at the given output directory.
    pub fn new(output_root: PathBuf) -> Self {
        Self {
            store: JobStore::default(),
            output_root: Arc::new(output_root),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct JobRequest {
    #[serde(default)]
    enable_j2: bool,
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
    let req = body.map(|Json(request)| request).unwrap_or_default();
    state
        .store
        .insert(JobStatus::pending(id.clone(), Workflow::Synthetic));

    let state_clone = state.clone();
    let id_clone = id.clone();
    tokio::task::spawn_blocking(move || run_job(state_clone, id_clone, req));

    let status = state.store.get(&id).expect("inserted job must exist");
    (StatusCode::ACCEPTED, Json(status))
}

async fn job_status(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.store.get(&id) {
        Some(status) => (StatusCode::OK, Json(status)).into_response(),
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

fn run_job(state: AppState, id: String, req: JobRequest) {
    if state
        .store
        .transition(&id, |status| status.start())
        .is_err()
    {
        return;
    }

    let out = state.output_root.join(&id);
    let request_ref = match persist_request(&out, &req) {
        Ok(request_ref) => request_ref,
        Err(error) => {
            let job_error = JobError {
                code: crate::service::JobErrorCode::Input,
                message: format!("input resolution failed: {error}"),
            };
            let _ = state.store.transition(&id, |status| status.fail(job_error));
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
        output_dir: ".".into(),
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
    match result {
        Ok(report) => {
            let job_result = crate::service::JobResult::from(&report);
            let _ = state
                .store
                .transition(&id, |status| status.succeed(job_result));
        }
        Err(error) => {
            let job_error = JobError::from(&error);
            let _ = state.store.transition(&id, |status| status.fail(job_error));
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
    use axum::body::Body;
    use axum::http::Request;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;

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

    #[test]
    fn relative_job_output_resolves_to_the_job_directory() {
        let root =
            std::env::temp_dir().join(format!("spod-rest-relative-output-{}", std::process::id()));
        let request_path = root.join("job-id/request.json");
        let config = RunConfig {
            schema_version: "1.0.0".into(),
            run_id: "job-id".into(),
            workflow: Workflow::Synthetic,
            inputs: InputsConfig {
                sp3: None,
                rinex_obs: None,
                rinex_nav: None,
                antex: None,
            },
            output_dir: ".".into(),
            forces: ForcesConfig {
                two_body: true,
                j2: false,
                third_body: false,
            },
        };

        assert_eq!(
            config.output_dir_relative_to(&request_path),
            root.join("job-id")
        );
    }

    #[test]
    fn accepted_status_has_stable_shape() {
        let status = JobStatus::pending("job-id".into(), Workflow::Synthetic);
        let value = serde_json::to_value(status).unwrap();
        assert_eq!(value["state"], "pending");
        assert_eq!(value["workflow"], "synthetic");
        assert!(value.get("result").is_none());
        assert!(value.get("error").is_none());
    }

    #[tokio::test]
    async fn unknown_job_id_returns_not_found() {
        let app = router(AppState::new(std::env::temp_dir()));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/jobs/missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn execution_failures_are_structured() {
        let root = std::env::temp_dir().join(format!("spod-rest-failure-{}", std::process::id()));
        fs::write(&root, "not a directory").unwrap();
        let state = AppState::new(root.clone());
        let id = "failure-job".to_owned();
        state
            .store
            .insert(JobStatus::pending(id.clone(), Workflow::Synthetic));
        run_job(state.clone(), id.clone(), JobRequest::default());

        let status = state.store.get(&id).unwrap();
        assert_eq!(status.state, crate::service::JobState::Failed);
        assert_eq!(
            status.error.as_ref().unwrap().code,
            crate::service::JobErrorCode::Input
        );
        fs::remove_file(root).unwrap();
    }
}
