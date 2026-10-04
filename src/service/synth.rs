//! # Synthetic GNSS arc generation
//!
//! ## Scientific scope
//!
//! Synthetic truth data is essential for end-to-end POD tests because it
//! gives a known reference orbit and controlled measurements without
//! external archive dependencies. This module generates a small
//! deterministic GNSS scenario with spacecraft truth states, transmitter
//! states, and corresponding measurements.
//!
//! The regime is intentionally idealized: two-body dynamics, fixed
//! transmitter orbits, and configurable but simple measurement noise. It is
//! appropriate for pipeline verification, not for representing the full
//! complexity of operational GNSS tracking.
//!
//! ## Technical scope
//!
//! The synthetic types are internal workflow data. External callers select
//! this workflow through [`super::Runner`] rather than constructing arcs.
//!
//! Real data ingestion, richer force models, and product writing are
//! delegated elsewhere.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
//! - Consultative Committee for Space Data Systems. (2010). Orbit Data
//!   Messages, CCSDS 502.0-B-2 / 502.0-B-3.
use super::pipeline::{ArcEpoch, GpsSatellite};
use siderust::astro::dynamics::context::DynamicsContext;
use siderust::astro::dynamics::forces::TwoBody;
use siderust::astro::dynamics::state::VelocityUnit;
use siderust::astro::dynamics::{OrbitState, Position, Velocity};
use siderust::coordinates::frames::GCRS;
use siderust::pod::observation::gnss_obs::{
    GnssCarrierPhaseObs, GnssPseudorangeObs, IonoModel, TropModel,
};
use siderust::pod::observation::obs_trait::Observation;
use siderust::pod::observation::provider_bundle::ProviderBundle;
use siderust::principia::integrators::rk4_propagate_series;
use siderust::qtty::{Hertzs, Meters, Second};
use siderust::time::JulianDate;

/// Configuration for a synthetic arc.
#[derive(Debug, Clone)]
pub(super) struct SyntheticArcConfig {
    /// Truth LEO state at the arc start (epoch in TT).
    pub truth_initial: OrbitState,
    /// Step size, seconds.
    pub dt_s: f64,
    /// Number of steps (arc length = dt_s * n_steps).
    pub n_steps: usize,
    /// Number of GPS satellites to simulate (placed on circular orbits).
    pub n_gps_sats: usize,
    /// Standard deviation of code noise, metres.
    pub code_sigma_m: f64,
    /// Standard deviation of carrier noise, metres.
    pub carrier_sigma_m: f64,
    /// Receiver clock bias (truth), metres.
    pub clock_bias_m: f64,
    /// Random seed.
    pub seed: u64,
}

impl Default for SyntheticArcConfig {
    fn default() -> Self {
        let r0 = 6_378.137 + 500.0;
        let v0 = (398_600.441_8_f64 / r0).sqrt();
        Self {
            truth_initial: OrbitState::new(
                JulianDate::new(2_451_545.0).to_j2000s(),
                Position::new(r0, 0.0, 0.0),
                Velocity::new(0.0, v0, 0.0),
            ),
            dt_s: 30.0,
            n_steps: 120,
            n_gps_sats: 6,
            code_sigma_m: 0.5,
            carrier_sigma_m: 0.005,
            clock_bias_m: 12_345.6,
            seed: 0xC0FFEE,
        }
    }
}

/// Synthetic arc bundle: truth states + per-epoch observations.
#[derive(Debug, Clone)]
pub(super) struct SyntheticArc {
    /// Truth states at every epoch.
    pub(super) truth_states: Vec<OrbitState>,
    /// One bundle per epoch.
    pub(super) epochs: Vec<ArcEpoch>,
}

#[derive(Clone, Copy)]
pub(crate) struct SyntheticProviders {
    pub(crate) receiver_clock_m: f64,
}

impl ProviderBundle for SyntheticProviders {
    fn gnss_satellite_state_gcrs(
        &self,
        _prn: &str,
        _epoch: JulianDate,
    ) -> Option<(Position<GCRS>, Velocity<GCRS, VelocityUnit>)> {
        None
    }
    fn gnss_satellite_clock_m(&self, _prn: &str, _epoch: JulianDate) -> Meters {
        Meters::new(0.0)
    }
    fn receiver_clock_m(&self) -> Meters {
        Meters::new(self.receiver_clock_m)
    }
    fn station_gcrs_km(&self, _station_id: &str, _epoch: JulianDate) -> Option<Position<GCRS>> {
        None
    }
}

fn gps_state_at(
    jd: JulianDate,
    slot: usize,
    n: usize,
) -> (Position<GCRS>, Velocity<GCRS, VelocityUnit>) {
    // Simple circular orbits at GPS altitude (~26 600 km), evenly spaced
    // in argument of latitude across two planes.
    let r = 26_600.0_f64;
    let mu = 398_600.441_8_f64;
    let n_mean = (mu / (r * r * r)).sqrt(); // rad/s
    let theta0 = 2.0 * std::f64::consts::PI * (slot as f64) / (n as f64);
    let inc: f64 = if slot.is_multiple_of(2) { 0.95 } else { 1.05 }; // ~55°
    let dt = (jd.value() - 2_451_545.0) * 86_400.0;
    let theta = theta0 + n_mean * dt;
    let cos_t = theta.cos();
    let sin_t = theta.sin();
    let (cos_i, sin_i) = (inc.cos(), inc.sin());
    let vmag = (mu / r).sqrt();
    (
        Position::<GCRS>::new(r * cos_t, r * sin_t * cos_i, r * sin_t * sin_i),
        Velocity::<GCRS, VelocityUnit>::new(
            -vmag * sin_t,
            vmag * cos_t * cos_i,
            vmag * cos_t * sin_i,
        ),
    )
}

/// Tiny linear-congruential PRNG (deterministic, no external dep).
struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(1))
    }
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 11) as f64) / (1_u64 << 53) as f64
    }
    /// Standard normal via Box-Muller.
    fn normal(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-300);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// Generate the synthetic arc.
pub(super) fn generate(cfg: &SyntheticArcConfig) -> SyntheticArc {
    let force = TwoBody::new(siderust::astro::dynamics::GM_EARTH);
    let truth = rk4_propagate_series(
        &force,
        cfg.truth_initial,
        Second::new(cfg.dt_s),
        cfg.n_steps,
        &DynamicsContext::empty(),
    )
    .expect("synthetic two-body propagation must succeed");
    let mut rng = Lcg::new(cfg.seed);
    let gps_sats: Vec<GpsSatellite> = (0..cfg.n_gps_sats)
        .map(|i| GpsSatellite {
            id: format!("G{:02}", i + 1),
            slot: i,
        })
        .collect();
    let mut epochs = Vec::with_capacity(truth.len());
    for s in &truth {
        let mut code = Vec::new();
        let mut carrier = Vec::new();
        for sat in &gps_sats {
            let (gps_pos, gps_vel) = gps_state_at(
                s.epoch.to::<siderust::tempoch::JD>(),
                sat.slot,
                cfg.n_gps_sats,
            );
            let epoch = s.epoch.to::<siderust::tempoch::JD>();
            let providers = SyntheticProviders {
                receiver_clock_m: cfg.clock_bias_m,
            };
            let mut code_obs = GnssPseudorangeObs {
                prn: sat.id.clone(),
                epoch,
                measured_m: Meters::new(0.0),
                sigma: Meters::new(cfg.code_sigma_m),
                sat_pos_gcrs_km: gps_pos,
                sat_vel_gcrs_km_s: gps_vel,
                trop: TropModel::None,
                iono: IonoModel::IonoFree,
                frequency_hz: Hertzs::new(1_575_420_000.0),
            };
            let mut phase_obs = GnssCarrierPhaseObs {
                prn: sat.id.clone(),
                epoch,
                measured_m: Meters::new(0.0),
                sigma: Meters::new(cfg.carrier_sigma_m),
                sat_pos_gcrs_km: gps_pos,
                sat_vel_gcrs_km_s: gps_vel,
                trop: TropModel::None,
                iono: IonoModel::IonoFree,
                frequency_hz: Hertzs::new(1_575_420_000.0),
                integer_ambiguity: 0,
            };
            let code_truth = -code_obs
                .residual(s, &providers)
                .expect("synthetic observation");
            let phase_truth = -phase_obs
                .residual(s, &providers)
                .expect("synthetic observation")
                .residual_m;
            let measured_code = code_truth + cfg.code_sigma_m * rng.normal();
            let measured_phase = phase_truth + cfg.carrier_sigma_m * rng.normal();
            code_obs.measured_m = Meters::new(measured_code);
            phase_obs.measured_m = Meters::new(measured_phase);
            code.push((sat.clone(), code_obs));
            carrier.push((sat.clone(), phase_obs));
        }
        epochs.push(ArcEpoch {
            state_index: epochs.len(),
            code,
            carrier,
        });
    }
    SyntheticArc {
        truth_states: truth,
        epochs,
    }
}
