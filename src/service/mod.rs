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
//! The public surface exposes run configuration, synthetic-arc generation,
//! the batch pipeline, and the top-level `run` helper. Reusable scientific
//! types are imported directly from Siderust rather than re-exported through
//! this service namespace. Inputs are configuration objects and filesystem
//! roots; outputs are reports and artifact paths.
//!
//! HTTP transport and command-line dispatch live in the `spod-rest` and
//! `spod` binaries, while
//! reusable numerical estimation remains in `siderust::pod::estimation`.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Consultative Committee for Space Data Systems. (2010). Orbit Data
//!   Messages, CCSDS 502.0-B-2 / 502.0-B-3.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod pipeline;
pub mod providers;
pub mod rest;
pub mod runner;
pub mod synth;

pub use config::RunConfig;
pub use pipeline::{
    run_synth, ArcEpoch, GpsSatellite, PipelineError, PipelineReport, RunProvenance,
};
pub use runner::{run, RunReport};
pub use synth::{generate, SyntheticArc, SyntheticArcConfig};

