//! Errors exposed at the service boundary.

use super::pipeline::PipelineError;
use std::io;
use thiserror::Error;

/// Categorized failure from configuration, orchestration, or execution.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Configuration could not be parsed or violates the service schema.
    #[error("invalid configuration: {message}")]
    Configuration {
        /// Human-readable validation detail.
        message: String,
    },
    /// A requested workflow or force/input combination is not supported.
    #[error("unsupported workflow or input: {message}")]
    Unsupported {
        /// Unsupported workflow or input detail.
        message: String,
    },
    /// A configured input or configuration file could not be read.
    #[error("input resolution failed: {source}")]
    Input {
        /// Underlying filesystem error.
        source: io::Error,
    },
    /// Generated output files could not be created or finalized.
    #[error("artifact handling failed: {source}")]
    Artifact {
        /// Underlying filesystem error.
        source: io::Error,
    },
    /// Siderust-backed execution failed.
    #[error("scientific execution failed: {source}")]
    Scientific {
        /// Underlying Siderust-backed pipeline error.
        source: PipelineError,
    },
}

impl ServiceError {
    pub(crate) fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    pub(crate) fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported {
            message: message.into(),
        }
    }

    pub(crate) fn input(error: io::Error) -> Self {
        Self::Input { source: error }
    }

    pub(crate) fn artifact(error: io::Error) -> Self {
        Self::Artifact { source: error }
    }
}
