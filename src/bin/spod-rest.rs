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
    new: Result<String, std::env::VarError>,
    legacy: Result<String, std::env::VarError>,
    internal_default: Result<String, std::env::VarError>,
    default: &str,
    new_name: &str,
    legacy_name: &str,
) -> anyhow::Result<(String, bool)> {
    match new {
        Ok(value) => Ok((value, false)),
        Err(std::env::VarError::NotPresent) => match legacy {
            Ok(value) => Ok((value, true)),
            Err(std::env::VarError::NotPresent) => match internal_default {
                Ok(value) => Ok((value, false)),
                Err(std::env::VarError::NotPresent) => Ok((default.to_owned(), false)),
                Err(std::env::VarError::NotUnicode(_)) => {
                    anyhow::bail!("internal REST default is not valid Unicode")
                }
            },
            Err(std::env::VarError::NotUnicode(_)) => {
                anyhow::bail!("{legacy_name} is not valid Unicode")
            }
        },
        Err(std::env::VarError::NotUnicode(_)) => anyhow::bail!("{new_name} is not valid Unicode"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let (bind, bind_legacy) = resolve_setting(
        std::env::var("SPOD_REST_BIND"),
        std::env::var("SIDERUST_POD_REST_BIND"),
        std::env::var("SPOD_REST_DEFAULT_BIND"),
        "127.0.0.1:8080",
        "SPOD_REST_BIND",
        "SIDERUST_POD_REST_BIND",
    )?;
    if bind_legacy {
        log::warn!("SIDERUST_POD_REST_BIND is deprecated; use SPOD_REST_BIND instead");
    }
    let (out, out_legacy) = resolve_setting(
        std::env::var("SPOD_REST_OUT"),
        std::env::var("SIDERUST_POD_REST_OUT"),
        Err(std::env::VarError::NotPresent),
        &std::env::temp_dir().join("spod-rest").display().to_string(),
        "SPOD_REST_OUT",
        "SIDERUST_POD_REST_OUT",
    )?;
    if out_legacy {
        log::warn!("SIDERUST_POD_REST_OUT is deprecated; use SPOD_REST_OUT instead");
    }
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
    use std::ffi::OsString;
    #[cfg(unix)]
    use std::os::unix::ffi::OsStringExt;

    fn present(value: &str) -> Result<String, std::env::VarError> {
        Ok(value.to_owned())
    }

    #[test]
    fn new_variable_wins() {
        let result = resolve_setting(
            present("new"),
            present("legacy"),
            Err(std::env::VarError::NotPresent),
            "default",
            "NEW",
            "LEGACY",
        )
        .unwrap();
        assert_eq!(result, ("new".to_owned(), false));
    }

    #[test]
    fn legacy_variable_is_used_when_new_is_absent() {
        let result = resolve_setting(
            Err(std::env::VarError::NotPresent),
            present("legacy"),
            Err(std::env::VarError::NotPresent),
            "default",
            "NEW",
            "LEGACY",
        )
        .unwrap();
        assert_eq!(result, ("legacy".to_owned(), true));
    }

    #[test]
    fn default_is_used_when_variables_are_absent() {
        let result = resolve_setting(
            Err(std::env::VarError::NotPresent),
            Err(std::env::VarError::NotPresent),
            Err(std::env::VarError::NotPresent),
            "default",
            "NEW",
            "LEGACY",
        )
        .unwrap();
        assert_eq!(result, ("default".to_owned(), false));
    }

    #[cfg(unix)]
    #[test]
    fn invalid_new_variable_is_rejected() {
        let result = resolve_setting(
            Err(std::env::VarError::NotUnicode(OsString::from_vec(vec![
                0xff,
            ]))),
            present("legacy"),
            Err(std::env::VarError::NotPresent),
            "default",
            "NEW",
            "LEGACY",
        );
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn invalid_legacy_variable_is_rejected() {
        let result = resolve_setting(
            Err(std::env::VarError::NotPresent),
            Err(std::env::VarError::NotUnicode(OsString::from_vec(vec![
                0xff,
            ]))),
            Err(std::env::VarError::NotPresent),
            "default",
            "NEW",
            "LEGACY",
        );
        assert!(result.is_err());
    }

    #[test]
    fn internal_default_is_used_after_public_variables() {
        let result = resolve_setting(
            Err(std::env::VarError::NotPresent),
            Err(std::env::VarError::NotPresent),
            present("docker-default"),
            "native-default",
            "NEW",
            "LEGACY",
        )
        .unwrap();
        assert_eq!(result, ("docker-default".to_owned(), false));
    }
}
