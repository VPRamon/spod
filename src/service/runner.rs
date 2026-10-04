//! # Configuration-driven pipeline runner
//!
//! ## Scientific scope
//!
//! The runner binds a user-facing run configuration to an executable POD
//! path. In the current workspace that mainly means selecting the
//! deterministic synthetic-arc workflow, validating its inputs, and
//! packaging the resulting outputs and manifest.
//!
//! No new scientific model is introduced in this layer. Its validity regime
//! is whichever estimation path it dispatches to, with current support
//! intentionally limited to the MVP synthetic branch.
//!
//! ## Technical scope
//!
//! The public surface is `RunReport` and `run`. Given a parsed `RunConfig`,
//! the runner resolves output locations, selects the appropriate pipeline
//! branch, and returns a summary of the completed run.
//!
//! It does not parse every underlying file format itself or solve
//! estimation subproblems directly.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Consultative Committee for Space Data Systems. (2010). Orbit Data
//!   Messages, CCSDS 502.0-B-2 / 502.0-B-3.
use super::config::RunConfig;
use super::pipeline::{run_synth, RunProvenance};
use super::synth::{generate, SyntheticArcConfig};
use siderust::astro::dynamics::{OrbitState, Position, Velocity};
use siderust::pod::run::dataset::DatasetRef;
use std::path::PathBuf;

/// Outcome of a run.
#[derive(Debug)]
pub struct RunReport {
    /// Final state at the end of the propagated arc.
    pub final_state: OrbitState,
    /// Run manifest written to disk.
    pub manifest_path: PathBuf,
    /// Number of integration steps.
    pub n_steps: usize,
    /// Number of nonlinear estimation iterations.
    pub estimator_iterations: usize,
    /// Final reduced chi-squared statistic.
    pub reduced_chi2: f64,
}

/// Run a configuration end-to-end.
pub fn run(cfg: &RunConfig, config_path: &str) -> std::io::Result<RunReport> {
    cfg.validate()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let output_dir = PathBuf::from(&cfg.output_dir);

    if cfg.inputs.sp3.is_some() || cfg.inputs.rinex_obs.is_some() {
        // Refuse rather than silently propagate-and-ignore.
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "not implemented: real GNSS ingestion (SP3 / RINEX OBS → estimator) is scheduled \
             for milestone M9; see plan.md §13.3. The current build only supports the \
             synthetic-arc MVP-1 pipeline (`inputs.sp3` and `inputs.rinex_obs` both null).",
        ));
    }

    let config_ref = DatasetRef::from_file(config_path, "configuration")?;
    let mut input_refs = Vec::new();
    for (path, kind) in [
        (cfg.inputs.rinex_nav.as_deref(), "rinex-nav"),
        (cfg.inputs.antex.as_deref(), "antex"),
    ] {
        if let Some(path) = path {
            input_refs.push(DatasetRef::from_file(path, kind)?);
        }
    }
    let provenance = RunProvenance::from_config(config_ref, input_refs);

    // Synthetic-arc MVP-1 pipeline.
    let synth_cfg = SyntheticArcConfig::default();
    let arc = generate(&synth_cfg);
    let t0 = arc.truth_states[0];
    let init = OrbitState::new(
        t0.epoch,
        Position::new(
            t0.position.x().value() + 0.05,
            t0.position.y().value() - 0.05,
            t0.position.z().value() + 0.05,
        ),
        Velocity::new(
            t0.velocity.x().value() + 5e-5,
            t0.velocity.y().value() - 5e-5,
            t0.velocity.z().value() + 5e-5,
        ),
    );
    let report = run_synth(
        &arc,
        init,
        0.0,
        &output_dir,
        &cfg.run_id,
        cfg.forces.j2,
        provenance,
    )
    .map_err(std::io::Error::other)?;
    Ok(RunReport {
        final_state: report.estimated_final,
        manifest_path: report.manifest_path,
        n_steps: arc.epochs.len(),
        estimator_iterations: report.estimator.iterations,
        reduced_chi2: report.estimator.last.reduced_chi2(),
    })
}
