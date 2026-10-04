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

fn resolve_setting(
    value: Result<String, std::env::VarError>,
    default: &str,
    name: &str,
) -> anyhow::Result<String> {
    match value {
        Ok(value) => Ok(value),
        Err(std::env::VarError::NotPresent) => Ok(default.to_owned()),
        Err(std::env::VarError::NotUnicode(_)) => anyhow::bail!("{name} is not valid Unicode"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let bind = resolve_setting(
        std::env::var("SPOD_REST_BIND"),
        "127.0.0.1:8080",
        "SPOD_REST_BIND",
    )?;
    let out = resolve_setting(
        std::env::var("SPOD_REST_OUT"),
        &std::env::temp_dir().join("spod-rest").display().to_string(),
        "SPOD_REST_OUT",
    )?;
    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out)?;
    let state = AppState::new(out);
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    log::info!("spod-rest listening on {}", bind);
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::resolve_setting;

    fn present(value: &str) -> Result<String, std::env::VarError> {
        Ok(value.to_owned())
    }

    #[test]
    fn new_variable_wins() {
        let result = resolve_setting(present("new"), "default", "NEW").unwrap();
        assert_eq!(result, "new");
    }

    #[test]
    fn default_is_used_when_variables_are_absent() {
        let result =
            resolve_setting(Err(std::env::VarError::NotPresent), "default", "NEW").unwrap();
        assert_eq!(result, "default");
    }

    #[cfg(unix)]
    #[test]
    fn invalid_variable_is_rejected() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let result = resolve_setting(
            Err(std::env::VarError::NotUnicode(OsString::from_vec(vec![
                0xff,
            ]))),
            "default",
            "SPOD_REST_BIND",
        );
        assert!(result.is_err());
    }
}
