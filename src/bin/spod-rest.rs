//! # REST server entrypoint
//!
//! ## Scientific scope
//!
//! This binary boots the Axum-based REST wrapper around the POD service
//! crate. The scientific behaviour is inherited completely from the
//! underlying service path; the executable only selects bind addresses,
//! output directories, and server lifetime.
//!
//! It is intended for local experimentation and integration testing with
//! the synthetic pipeline rather than for a fully hardened production
//! deployment.
//!
//! ## Technical scope
//!
//! The `main` function reads environment variables for bind and output
//! paths, initializes logging, builds `AppState`, and serves the router
//! returned by `spod-rest`. Requests and responses are JSON over
//! HTTP.
//!
//! No estimation, parsing, or product logic lives in this file.
//!
//! ## References
//!
//! - Fielding, R., Nottingham, M., & Reschke, J. (2022). HTTP Semantics.
//!   RFC 9110.
//! - Bray, T. (2017). The JavaScript Object Notation (JSON) Data
//!   Interchange Format. RFC 8259.
use std::path::PathBuf;

use spod::service::rest::{router, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let bind = match std::env::var("SPOD_REST_BIND") {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => match std::env::var("SIDERUST_POD_REST_BIND") {
            Ok(value) => {
                log::warn!("SIDERUST_POD_REST_BIND is deprecated; use SPOD_REST_BIND instead");
                value
            }
            Err(std::env::VarError::NotPresent) => "127.0.0.1:8080".into(),
            Err(std::env::VarError::NotUnicode(_)) => {
                anyhow::bail!("SIDERUST_POD_REST_BIND is not valid Unicode")
            }
        },
        Err(std::env::VarError::NotUnicode(_)) => {
            anyhow::bail!("SPOD_REST_BIND is not valid Unicode")
        }
    };
    let out = match std::env::var("SPOD_REST_OUT") {
        Ok(value) => PathBuf::from(value),
        Err(std::env::VarError::NotPresent) => match std::env::var("SIDERUST_POD_REST_OUT") {
            Ok(value) => {
                log::warn!("SIDERUST_POD_REST_OUT is deprecated; use SPOD_REST_OUT instead");
                PathBuf::from(value)
            }
            Err(std::env::VarError::NotPresent) => std::env::temp_dir().join("spod-rest"),
            Err(std::env::VarError::NotUnicode(_)) => {
                anyhow::bail!("SIDERUST_POD_REST_OUT is not valid Unicode")
            }
        },
        Err(std::env::VarError::NotUnicode(_)) => {
            anyhow::bail!("SPOD_REST_OUT is not valid Unicode")
        }
    };
    std::fs::create_dir_all(&out)?;
    let state = AppState::new(out);
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    log::info!("spod-rest listening on {}", bind);
    axum::serve(listener, app).await?;
    Ok(())
}
