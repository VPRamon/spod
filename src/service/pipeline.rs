//! # Synthetic batch POD pipeline
//!
//! ## Scientific scope
//!
//! This module assembles the current MVP synthetic orbit-determination
//! workflow: generate or receive a controlled GNSS arc, build scalar
//! observation equations, solve for a corrected spacecraft state and
//! nuisance parameters, and emit standard orbit and QC artifacts. The
//! scientific regime is deliberately narrow so the whole chain stays
//! deterministic and regression-testable.
//!
//! The pipeline presently targets synthetic GNSS data and simplified force
//! modelling. It is therefore a system-integration path for the broader
//! architecture, not yet a full operational POD service for heterogeneous
//! real data.
//!
//! ## Technical scope
//!
//! This is an internal implementation module. It assembles the synthetic arc
//! and Siderust execution after the public runner has validated a request.
//!
//! Observation physics, linear algebra, file-format serialization, and
//! manifest encoding are delegated to their respective crates and only
//! orchestrated here.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Consultative Committee for Space Data Systems. (2010). Orbit Data
//!   Messages, CCSDS 502.0-B-2 / 502.0-B-3.
use super::artifacts::ArtifactLayout;
use super::provenance::RunProvenance;
use super::synth::SyntheticArc;
use super::synth::SyntheticProviders;
use chrono::{SecondsFormat, Utc};
use siderust::astro::dynamics::context::DynamicsContext;
use siderust::astro::dynamics::forces::{TwoBody, J2};
use siderust::astro::dynamics::{OrbitState, Position, Velocity, EARTH_J2, GM_EARTH, R_EARTH};
use siderust::pod::estimation::{
    gauss_newton, NonlinearError, NonlinearOptions, NonlinearReport, NormalEquations,
};
use siderust::pod::force::{SiderustAccelerationModel, SiderustCompositeModel};
use siderust::pod::observation::obs_trait::{ObsResidual, Observation};
use siderust::pod::product::qc_json::{write_qc_json, QcDocument};
use siderust::pod::product::residuals_csv::{ResidualCsvWriter, ResidualRecord};
use siderust::pod::product::{write_oem_from_states, write_sp3_from_states};
use siderust::pod::propagation::{PropagatedArc, VariationalPropagator};
use siderust::pod::qc::ResidualsByGroup;
use siderust::pod::run::dataset::DatasetRef;
use siderust::pod::run::manifest::RunManifest;
use siderust::qtty::Second;
use std::fs;
use thiserror::Error;

/// Identifier for a simulated GPS satellite.
#[derive(Debug, Clone)]
pub(super) struct GpsSatellite {
    /// Three-character ID (e.g. `G01`).
    pub(super) id: String,
    /// Slot index used by the synthetic ephemeris.
    pub(super) slot: usize,
}

/// Per-epoch bundle of observations.
#[derive(Debug, Clone)]
pub(super) struct ArcEpoch {
    /// Index of the corresponding state in the propagated series.
    pub(super) state_index: usize,
    /// Code observations.
    pub(super) code: Vec<(
        GpsSatellite,
        siderust::pod::observation::gnss_obs::GnssPseudorangeObs,
    )>,
    /// Carrier observations.
    pub(super) carrier: Vec<(
        GpsSatellite,
        siderust::pod::observation::gnss_obs::GnssCarrierPhaseObs,
    )>,
}

/// Pipeline errors.
#[derive(Debug, Error)]
pub(super) enum PipelineError {
    /// Filesystem failure while creating or writing service artifacts.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Estimation failure.
    #[error("estimation: {0}")]
    Estimation(#[from] NonlinearError),
    /// Propagation failure.
    #[error("propagation: {0}")]
    Propagation(#[from] siderust::pod::propagation::PodDynamicsError),
    /// Product or manifest serialization failure.
    #[error("product serialization: {0}")]
    Product(String),
}

/// Outcome of an MVP-1 pipeline run.
#[derive(Debug)]
pub(super) struct PipelineReport {
    /// Estimator convergence report.
    pub(super) estimator: NonlinearReport,
    /// Final state at end of arc.
    pub(super) estimated_final: OrbitState,
}

/// Run MVP-1 against a synthetic arc and write all artifacts to `output_dir`.
pub(super) fn run_synth(
    arc: &SyntheticArc,
    initial_guess: OrbitState,
    initial_clock_guess_m: f64,
    layout: &ArtifactLayout,
    run_id: &str,
    enable_j2: bool,
    provenance: RunProvenance,
) -> Result<PipelineReport, PipelineError> {
    let started_at = Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
    let dt_s = step_size(arc);
    let n_steps = arc.epochs.len().saturating_sub(1);
    let n_params = 7; // state + receiver clock; carrier ambiguity is fixed in the service MVP

    let mut params = vec![0.0_f64; n_params];
    params[0] = initial_guess.position.x().value();
    params[1] = initial_guess.position.y().value();
    params[2] = initial_guess.position.z().value();
    params[3] = initial_guess.velocity.x().value();
    params[4] = initial_guess.velocity.y().value();
    params[5] = initial_guess.velocity.z().value();
    params[6] = initial_clock_guess_m;

    let opts = NonlinearOptions {
        max_iter: 20,
        tol_rel: 1e-7,
        tol_chi2_rel: 1e-4,
    };

    let force = build_force(enable_j2);
    let arc_ref = arc;
    let report = gauss_newton(
        params,
        opts,
        |params| -> Result<NormalEquations, NonlinearError> {
            assemble_normal_equations(arc_ref, params, dt_s, n_steps, &force)
                .map_err(NonlinearError::Solver)
        },
    )?;

    let estimated_initial = OrbitState::new(
        initial_guess.epoch,
        Position::new(
            report.parameters[0],
            report.parameters[1],
            report.parameters[2],
        ),
        Velocity::new(
            report.parameters[3],
            report.parameters[4],
            report.parameters[5],
        ),
    );
    let estimated_arc = propagate_with_stm(&force, estimated_initial, dt_s, n_steps)?;
    let estimated_states = states_with_initial(estimated_initial, &estimated_arc);
    let estimated_final = *estimated_states.last().unwrap_or(&estimated_initial);

    // Postfit residuals.
    let residual_rows = postfit_residuals(arc, &estimated_states, &report.parameters);

    // Group statistics.
    let groups = ResidualsByGroup::from_pairs(
        residual_rows
            .iter()
            .map(|r| (r.obs_type.as_str(), r.residual_m)),
    );

    fs::create_dir_all(layout.root.join("products"))?;
    fs::create_dir_all(layout.root.join("residuals"))?;
    fs::create_dir_all(layout.root.join("qc"))?;

    // SP3.
    {
        let mut f = fs::File::create(&layout.orbit_sp3)?;
        write_sp3_from_states(&mut f, "L01", &estimated_states)
            .map_err(|e| PipelineError::Product(e.to_string()))?;
    }
    // OEM.
    {
        let mut f = fs::File::create(&layout.orbit_oem)?;
        write_oem_from_states(&mut f, "1900-001A", "POD-LEO", &estimated_states)
            .map_err(|e| PipelineError::Product(e.to_string()))?;
    }
    // Residuals CSV.
    {
        let mut f = fs::File::create(&layout.residuals_csv)?;
        let mut writer =
            ResidualCsvWriter::new(&mut f).map_err(|e| PipelineError::Product(e.to_string()))?;
        for row in &residual_rows {
            writer
                .write_record(row)
                .map_err(|e| PipelineError::Product(e.to_string()))?;
        }
    }
    // qc.json.
    {
        let doc = QcDocument {
            schema_version: "0.1.0".into(),
            run_id: run_id.into(),
            software_version: env!("CARGO_PKG_VERSION").into(),
            n_obs: residual_rows.len(),
            n_params,
            reduced_chi2: report.last.reduced_chi2(),
            iterations: report.iterations,
            residuals: groups,
        };
        let mut f = fs::File::create(&layout.qc_json)?;
        write_qc_json(&mut f, &doc).map_err(|e| PipelineError::Product(e.to_string()))?;
    }

    // Manifest.
    let mut manifest = RunManifest {
        run_id: run_id.into(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        config_sha256: provenance.config_sha256,
        inputs: provenance.inputs,
        outputs: vec![
            DatasetRef::from_file(layout.orbit_sp3.clone(), "orbit-sp3")?,
            DatasetRef::from_file(layout.orbit_oem.clone(), "orbit-oem")?,
            DatasetRef::from_file(layout.residuals_csv.clone(), "residuals")?,
            DatasetRef::from_file(layout.qc_json.clone(), "qc")?,
        ],
        started_at,
        finished_at: Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true),
    };
    manifest.canonicalize();
    fs::write(
        &layout.manifest,
        manifest
            .to_json_pretty()
            .map_err(|e| PipelineError::Product(e.to_string()))?,
    )?;

    Ok(PipelineReport {
        estimator: report,
        estimated_final,
    })
}

fn build_force(enable_j2: bool) -> SiderustCompositeModel {
    let mut f = SiderustCompositeModel::empty().push(Box::new(TwoBody::new(GM_EARTH)));
    if enable_j2 {
        f = f.push(Box::new(J2::new(GM_EARTH, R_EARTH, EARTH_J2)));
    }
    f
}

fn step_size(arc: &SyntheticArc) -> f64 {
    if arc.truth_states.len() < 2 {
        return 30.0;
    }
    let dt_jd = arc.truth_states[1]
        .epoch
        .to::<siderust::tempoch::JD>()
        .value()
        - arc.truth_states[0]
            .epoch
            .to::<siderust::tempoch::JD>()
            .value();
    dt_jd * 86_400.0
}

fn assemble_normal_equations<F: SiderustAccelerationModel>(
    arc: &SyntheticArc,
    params: &[f64],
    dt_s: f64,
    n_steps: usize,
    force: &F,
) -> Result<NormalEquations, siderust::pod::estimation::WlsSolverError> {
    let s0 = OrbitState::new(
        arc.truth_states[0].epoch,
        Position::new(params[0], params[1], params[2]),
        Velocity::new(params[3], params[4], params[5]),
    );
    let propagated = propagate_with_stm(force, s0, dt_s, n_steps).map_err(|e| {
        siderust::pod::estimation::WlsSolverError::other(format!("propagation failed: {e}"))
    })?;
    let states = states_with_initial(s0, &propagated);
    let n_params = 7;
    let mut ne = NormalEquations::new(n_params);
    let providers = SyntheticProviders {
        receiver_clock_m: params[6],
    };

    for ep in &arc.epochs {
        let s = &states[ep.state_index];
        let phi = if ep.state_index == 0 {
            identity_stm()
        } else {
            *propagated.steps[ep.state_index - 1].1.as_array()
        };
        for (_sat, obs) in &ep.code {
            let resid = obs
                .residual(s, &providers)
                .map_err(|e| siderust::pod::estimation::WlsSolverError::other(e.to_string()))?;
            let row = observation_row(obs, s, &providers, &phi, 6);
            ne.add_row(&row, resid, obs.sigma.value())?;
        }
        for (_sat, obs) in &ep.carrier {
            let resid = obs
                .residual(s, &providers)
                .map_err(|e| siderust::pod::estimation::WlsSolverError::other(e.to_string()))?
                .residual_m;
            let row = observation_row(obs, s, &providers, &phi, 6);
            ne.add_row(&row, resid, obs.sigma.value())?;
        }
    }
    Ok(ne)
}

fn propagate_with_stm<F: SiderustAccelerationModel>(
    force: &F,
    initial: OrbitState,
    dt_s: f64,
    n_steps: usize,
) -> Result<PropagatedArc, siderust::pod::propagation::PodDynamicsError> {
    VariationalPropagator {
        step: Second::new(dt_s),
    }
    .propagate(force, initial, n_steps, &DynamicsContext::empty())
}

fn states_with_initial(initial: OrbitState, propagated: &PropagatedArc) -> Vec<OrbitState> {
    let mut states = Vec::with_capacity(propagated.steps.len() + 1);
    states.push(initial);
    states.extend(propagated.steps.iter().map(|(state, _)| *state));
    states
}

fn identity_stm() -> [[f64; 6]; 6] {
    [
        [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
    ]
}

fn observation_row<O: Observation>(
    obs: &O,
    state: &OrbitState,
    providers: &SyntheticProviders,
    phi: &[[f64; 6]; 6],
    clock_index: usize,
) -> Vec<(usize, f64)>
where
    ObsResidual: From<O::Residual>,
{
    let h = [1e-3, 1e-3, 1e-3, 1e-6, 1e-6, 1e-6];
    let mut local = [0.0; 6];
    for i in 0..6 {
        let mut p = *state;
        let mut q = *state;
        let mut pv = [
            p.position.x().value(),
            p.position.y().value(),
            p.position.z().value(),
            p.velocity.x().value(),
            p.velocity.y().value(),
            p.velocity.z().value(),
        ];
        let mut qv = pv;
        pv[i] += h[i];
        qv[i] -= h[i];
        p = OrbitState::new(
            p.epoch,
            Position::new(pv[0], pv[1], pv[2]),
            Velocity::new(pv[3], pv[4], pv[5]),
        );
        q = OrbitState::new(
            q.epoch,
            Position::new(qv[0], qv[1], qv[2]),
            Velocity::new(qv[3], qv[4], qv[5]),
        );
        let rp = observation_residual(obs, &p, providers);
        let rq = observation_residual(obs, &q, providers);
        local[i] = -(rp - rq) / (2.0 * h[i]);
    }
    let mut out = Vec::with_capacity(7);
    for k in 0..6 {
        let value: f64 = (0..6).map(|i| local[i] * phi[i][k]).sum();
        if value != 0.0 {
            out.push((k, value));
        }
    }
    out.push((clock_index, 1.0));
    out
}

fn postfit_residuals(
    arc: &SyntheticArc,
    states: &[OrbitState],
    params: &[f64],
) -> Vec<ResidualRecord> {
    let providers = SyntheticProviders {
        receiver_clock_m: params[6],
    };
    let mut out = Vec::new();
    for ep in &arc.epochs {
        let s = &states[ep.state_index];
        for (sat, obs) in &ep.code {
            let residual = obs.residual(s, &providers).expect("postfit observation");
            out.push(ResidualRecord {
                epoch_jd_tt: s.epoch.to::<siderust::tempoch::JD>().value(),
                obs_type: "code".into(),
                satellite: sat.id.clone(),
                residual_m: residual,
                sigma_m: obs.sigma.value(),
                rejected: false,
            });
        }
        for (sat, obs) in &ep.carrier {
            let residual = obs
                .residual(s, &providers)
                .expect("postfit observation")
                .residual_m;
            out.push(ResidualRecord {
                epoch_jd_tt: s.epoch.to::<siderust::tempoch::JD>().value(),
                obs_type: "phase".into(),
                satellite: sat.id.clone(),
                residual_m: residual,
                sigma_m: obs.sigma.value(),
                rejected: false,
            });
        }
    }
    out
}

fn observation_residual<O>(obs: &O, state: &OrbitState, providers: &SyntheticProviders) -> f64
where
    O: Observation,
    ObsResidual: From<O::Residual>,
{
    match ObsResidual::from(
        obs.residual(state, providers)
            .expect("synthetic observation"),
    ) {
        ObsResidual::Scalar(value) => value,
        ObsResidual::Phase(value) => value.residual_m,
    }
}
