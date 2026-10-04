//! # spod core
//!
//! POD-specific domain primitives. Geometric and physical types from
//! `siderust`, `affn`, `qtty`, and `tempoch` are used directly throughout
//! the codebase; this module contains only what cannot be expressed by those
//! upstream crates.
//!
//! ## Contents
//!
//! - POD error taxonomy ([`error::PodError`])
//! - Run-level provenance ([`manifest::RunManifest`], [`dataset::DatasetRef`])
//! - Estimator parameter typing ([`parameter::ParameterKind`],
//!   [`parameter::Parameter`], [`covariance::ParameterCovariance`])
//! - Arc definitions ([`arc::ArcId`], [`arc::ArcDefinition`])
//! - Provider traits bridging POD code to external services ([`providers`])

#![forbid(unsafe_code)]

pub mod arc;
pub mod covariance;
pub mod dataset;
pub mod error;
pub mod manifest;
pub mod parameter;
pub mod providers;

pub use arc::{ArcDefinition, ArcId};
pub use error::PodError;
