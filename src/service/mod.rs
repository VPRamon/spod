#![allow(clippy::needless_range_loop, clippy::inconsistent_digit_grouping)]
//! # `spod::service` orchestration
//!
//! ## Scientific scope
//!
//! This module is the orchestration layer that turns configuration,
//! synthetic inputs, estimation kernels, observation models, and product
//! writers into an end-to-end POD workflow. Its scientific regime is the
//! currently supported MVP pipeline: deterministic synthetic GNSS
//! estimation with standard output artifacts and QC products.
//!
//! The crate does not own the low-level force or measurement physics.
//! Instead it composes those building blocks into reproducible runs and
//! exposes a stable application-facing surface.
//!
//! ## Technical scope
//!
//! The public surface is the [`Runner`], [`RunRequest`], and [`RunResult`]
//! service API. Reusable scientific types are imported directly from Siderust
//! rather than re-exported through this service namespace. Inputs are
//! configuration objects and filesystem roots; outputs are structured
//! artifact and provenance records.
//!
//! HTTP transport and command-line dispatch live in the `spod-rest` and
//! `spod` binaries, while reusable numerical estimation remains in
//! `siderust::pod::estimation`. The synthetic workflow is a reference
//! integration workflow, not the definition of the whole service.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Consultative Committee for Space Data Systems. (2010). Orbit Data
//!   Messages, CCSDS 502.0-B-2 / 502.0-B-3.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod artifacts;
pub mod config;
pub mod error;
pub mod job;
pub mod pipeline;
pub mod provenance;
pub mod rest;
pub mod runner;
pub mod synth;
pub mod workflow;

pub use artifacts::Artifacts;
pub use config::{ForcesConfig, InputsConfig, RunConfig};
pub use error::ServiceError;
pub use job::{JobId, RunReport, RunRequest, RunResult};
pub use pipeline::{run_synth, ArcEpoch, GpsSatellite, PipelineError, PipelineReport};
pub use provenance::RunProvenance;
pub use runner::{run, Runner};
pub use synth::{generate, SyntheticArc, SyntheticArcConfig};
pub use workflow::Workflow;
