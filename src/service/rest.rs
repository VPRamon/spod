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
    let status = JobStatus::pending(id.clone(), Workflow::Synthetic);
    state.store.insert(status.clone());

    let state_clone = state.clone();
    let id_clone = id.clone();
    tokio::task::spawn_blocking(move || run_job(state_clone, id_clone, req));

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
    if let Err(error) = state.store.transition(&id, |status| status.start()) {
        log::error!("could not start job {id}: {error:?}");
        return;
    }

    let out = state.output_root.join(&id);
    let request_ref = match persist_request(&out, &req) {
        Ok(request_ref) => request_ref,
        Err(error) => {
            let job_error = JobError::from(&crate::service::ServiceError::artifact(error));
            if let Err(transition_error) =
                state.store.transition(&id, |status| status.fail(job_error))
            {
                log::error!("could not fail persisted job {id}: {transition_error:?}");
            }
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
            if let Err(error) = state
                .store
                .transition(&id, |status| status.succeed(job_result))
            {
                log::error!("could not complete job {id}: {error:?}");
            }
        }
        Err(error) => {
            let job_error = JobError::from(&error);
            if let Err(transition_error) =
                state.store.transition(&id, |status| status.fail(job_error))
            {
                log::error!("could not fail job {id}: {transition_error:?}");
            }
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
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use serde_json::Value;
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

    #[tokio::test]
    async fn health_endpoint_returns_status_and_version() {
        let app = router(AppState::new(std::env::temp_dir()));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["status"], "ok");
        assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn submit_returns_pending_and_status_eventually_succeeds() {
        let root = std::env::temp_dir().join(format!("spod-rest-http-{}", std::process::id()));
        let app = router(AppState::new(root.clone()));
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/jobs")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"enable_j2":false}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let accepted: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(accepted["state"], "pending");
        assert!(accepted["result"].is_null());
        assert!(accepted["error"].is_null());
        let id = accepted["id"].as_str().unwrap();

        let status = poll_status(&app, id).await;
        assert_eq!(status["state"], "succeeded");
        assert!(status["result"]["artifacts"]["manifest"].is_object());
        fs::remove_dir_all(root).unwrap();
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

    #[tokio::test]
    async fn persistence_failure_reaches_failed_with_artifact_code() {
        let root = std::env::temp_dir().join(format!("spod-rest-failure-{}", std::process::id()));
        fs::write(&root, "not a directory").unwrap();
        let app = router(AppState::new(root.clone()));
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/jobs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let accepted: Value = serde_json::from_slice(&body).unwrap();
        let status = poll_status(&app, accepted["id"].as_str().unwrap()).await;
        assert_eq!(status["state"], "failed");
        assert_eq!(status["error"]["code"], "artifact");
        fs::remove_file(root).unwrap();
    }

    async fn poll_status(app: &Router, id: &str) -> Value {
        for _ in 0..100 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/jobs/{id}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let value: Value = serde_json::from_slice(&body).unwrap();
            if matches!(value["state"].as_str(), Some("succeeded" | "failed")) {
                return value;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("job did not reach a terminal state");
    }
}
