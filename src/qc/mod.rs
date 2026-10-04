//! # spod quality control
//!
//! ## Scientific scope
//!
//! This crate collects post-fit diagnostics for POD workflows: residual
//! summaries, orbit-to-orbit comparisons in local frames, and SLR
//! validation summaries. The scientific intent is to quantify solution
//! quality without modifying the estimated orbit itself.
//!
//! The current regime emphasizes compact deterministic reports suitable for
//! regression tests and MVP product generation. It does not attempt to
//! replace a full operational analysis toolchain.
//!
//! ## Technical scope
//!
//! The crate re-exports orbit-comparison helpers, grouped residual
//! statistics, HTML rendering, and SLR validation report types. Inputs are
//! already computed trajectories, residual lists, or QC JSON payloads
//! prepared by upstream stages.
//!
//! This crate does not read raw measurement files or perform estimation.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod html;
pub mod orbit_compare;
pub mod residuals;
pub mod slr_validation;

pub use html::render_html;
pub use orbit_compare::{rtn_diff, rtn_summary, RtnDiff, RtnSummary};
pub use residuals::{ResidualStats, ResidualsByGroup};
pub use slr_validation::{SlrResidual, SlrValidationReport};
