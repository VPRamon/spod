//! Supported service workflows.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A workflow dispatch target owned by the service layer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Workflow {
    /// Deterministic synthetic LEO/GNSS reference workflow.
    #[default]
    Synthetic,
}

impl fmt::Display for Workflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Synthetic => f.write_str("synthetic"),
        }
    }
}
