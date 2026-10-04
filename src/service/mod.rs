#![allow(clippy::needless_range_loop, clippy::inconsistent_digit_grouping)]
//! # spod service orchestration
//!
//! ## Scientific scope
//!
//! This crate is the orchestration layer that turns configuration,
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
//! The public surface re-exports run configuration, manifest helpers,
//! synthetic-arc generation, the batch pipeline, and the top-level `run`
//! helper, along with selected orbit-state types used by downstream
//! interfaces. Inputs are configuration objects and filesystem roots;
//! outputs are reports and artifact paths.
//!
//! HTTP transport and command-line dispatch live in sibling crates, while
//! numerical estimation remains in `spod::estimation`.
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
pub mod manifest;
pub mod pipeline;
pub mod rest;
pub mod runner;
pub mod synth;

pub use config::RunConfig;
pub use manifest::{canonical_json, DatasetRef, RunManifest};
pub use pipeline::{run_synth, ArcEpoch, GpsSatellite, PipelineError, PipelineReport};
pub use runner::{run, RunReport};
pub use synth::{generate, SyntheticArc, SyntheticArcConfig};

// Re-export the core orbit state types so downstream crates (pod-rest)
// that only depend on this crate can access them without adding siderust directly.
pub use siderust::astro::dynamics::state::VelocityUnit;
pub use siderust::astro::dynamics::{OrbitState, Position, Velocity};
