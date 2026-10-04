//! # GNSS code and carrier observation models
//!
//! ## Scientific scope
//!
//! This module models one-way GNSS geometry between a spacecraft receiver
//! state and known transmitter states. The current regime is intentionally
//! compact: geometric range, receiver clock bias, carrier ambiguity, and a
//! Sagnac-style Earth-rotation correction for short light times typical of
//! LEO and MEO POD scenarios.
//!
//! It does not yet include full light-time iteration, relativistic clock
//! terms, troposphere, ionosphere, antenna phase-centre modelling, or
//! ambiguity fixing. Those omissions are deliberate and documented so the
//! current use stays within the MVP synthetic and controlled-data paths.
//!
//! ## Technical scope
//!
//! The public surface provides `PseudorangeObs`, `CarrierPhaseObs`,
//! `GnssCodeModel`, and `GnssCarrierModel`. Models consume orbit states and
//! caller-supplied auxiliary parameters, then return scalar predictions and
//! design-matrix partials in the ordering expected by the estimation layer.
//!
//! File parsing and orbit propagation are not handled here; they are
//! supplied by the IO and service crates.
//!
//! ## References
//!
//! - Misra, P., & Enge, P. (2012). Global Positioning System: Signals,
//!   Measurements, and Performance (2nd ed.). Ganga-Jamuna Press.
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). Statistical Orbit
//!   Determination. Elsevier Academic Press.
use super::model::{MeasurementModel, Partials, Prediction};
use siderust::astro::dynamics::forces::OMEGA_EARTH_RAD_S;
use siderust::astro::dynamics::state::VelocityUnit;
use siderust::astro::dynamics::{OrbitState, Position, Velocity};
use siderust::coordinates::frames::GCRS;
use siderust::qtty::unit::{Kilometer, Second};
use siderust::qtty::velocity::C;
use siderust::qtty::Per;

/// Pseudorange observation between a LEO receiver and a GPS satellite.
#[derive(Debug, Clone, Copy)]
pub struct PseudorangeObs {
    /// GPS satellite GCRF position at signal-emission instant, km.
    pub gps_pos_km: Position<GCRS>,
    /// GPS satellite GCRF velocity, km/s. Used for the relativistic correction.
    pub gps_vel_km_s: Velocity<GCRS, VelocityUnit>,
    /// Measured pseudorange, metres.
    pub measured_m: f64,
    /// Measurement standard deviation, metres.
    pub sigma_m: f64,
}

/// Carrier-phase observation in metres (already scaled by wavelength).
#[derive(Debug, Clone, Copy)]
pub struct CarrierPhaseObs {
    /// GPS satellite GCRF position, km.
    pub gps_pos_km: Position<GCRS>,
    /// GPS satellite GCRF velocity, km/s.
    pub gps_vel_km_s: Velocity<GCRS, VelocityUnit>,
    /// Measured carrier-phase range, metres.
    pub measured_m: f64,
    /// Measurement standard deviation, metres.
    pub sigma_m: f64,
}

/// GNSS pseudorange model. Carries indices into the parameter vector for the
/// receiver clock bias.
#[derive(Debug, Clone)]
pub struct GnssCodeModel {
    /// Observation data.
    pub obs: PseudorangeObs,
    /// Index, in `extra_params`, of the receiver clock bias (metres).
    pub clock_bias_index: usize,
}

/// GNSS carrier-phase model. Carries indices into the parameter vector for
/// the receiver clock bias and the per-pass float ambiguity.
#[derive(Debug, Clone)]
pub struct GnssCarrierModel {
    /// Observation data.
    pub obs: CarrierPhaseObs,
    /// Index, in `extra_params`, of the receiver clock bias (metres).
    pub clock_bias_index: usize,
    /// Index, in `extra_params`, of the float ambiguity (metres).
    pub ambiguity_index: usize,
}

fn sagnac_km(gps_pos_km: Position<GCRS>, rx_pos_km: Position<GCRS>) -> f64 {
    let c_km_s = C.to::<Per<Kilometer, Second>>().value();
    OMEGA_EARTH_RAD_S.value()
        * (gps_pos_km.x().value() * rx_pos_km.y().value()
            - gps_pos_km.y().value() * rx_pos_km.x().value())
        / c_km_s
}

fn relativistic_gps_clock_m(pos_km: Position<GCRS>, vel_km_s: Velocity<GCRS, VelocityUnit>) -> f64 {
    let dot = pos_km.x().value() * vel_km_s.x().value()
        + pos_km.y().value() * vel_km_s.y().value()
        + pos_km.z().value() * vel_km_s.z().value();
    -2.0 * dot * 1_000.0 / C.value()
}

/// Predict a pseudorange in metres given the current LEO state and clock bias.
fn predict_range_m(
    state: &OrbitState,
    gps_pos_km: Position<GCRS>,
    gps_vel_km_s: Velocity<GCRS, VelocityUnit>,
) -> (f64, [f64; 3]) {
    let los = state.position - gps_pos_km;
    let geom_km = los.magnitude().value();
    let geom_m = geom_km * 1_000.0;
    let sagnac_m = sagnac_km(gps_pos_km, state.position) * 1_000.0;
    let rel_m = relativistic_gps_clock_m(gps_pos_km, gps_vel_km_s);
    let u = if geom_km > 0.0 {
        [
            los.x().value() / geom_km,
            los.y().value() / geom_km,
            los.z().value() / geom_km,
        ]
    } else {
        [0.0; 3]
    };
    (geom_m + sagnac_m + rel_m, u)
}

impl MeasurementModel for GnssCodeModel {
    fn predict(&self, state: &OrbitState, extra_params: &[f64]) -> Prediction {
        let clk_m = extra_params
            .get(self.clock_bias_index)
            .copied()
            .unwrap_or(0.0);
        let (range_m, u) = predict_range_m(state, self.obs.gps_pos_km, self.obs.gps_vel_km_s);
        let value = range_m + clk_m;
        // Partials w.r.t. position (km → metres factor): d/dr (geom_m) = u * 1000.
        let partials = Partials::from_pairs(vec![
            (0, u[0] * 1_000.0),
            (1, u[1] * 1_000.0),
            (2, u[2] * 1_000.0),
            (self.clock_bias_index + 6, 1.0),
        ]);
        Prediction { value, partials }
    }
    fn sigma(&self) -> f64 {
        self.obs.sigma_m
    }
}

impl MeasurementModel for GnssCarrierModel {
    fn predict(&self, state: &OrbitState, extra_params: &[f64]) -> Prediction {
        let clk_m = extra_params
            .get(self.clock_bias_index)
            .copied()
            .unwrap_or(0.0);
        let amb_m = extra_params
            .get(self.ambiguity_index)
            .copied()
            .unwrap_or(0.0);
        let (range_m, u) = predict_range_m(state, self.obs.gps_pos_km, self.obs.gps_vel_km_s);
        let value = range_m + clk_m + amb_m;
        let partials = Partials::from_pairs(vec![
            (0, u[0] * 1_000.0),
            (1, u[1] * 1_000.0),
            (2, u[2] * 1_000.0),
            (self.clock_bias_index + 6, 1.0),
            (self.ambiguity_index + 6, 1.0),
        ]);
        Prediction { value, partials }
    }
    fn sigma(&self) -> f64 {
        self.obs.sigma_m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::astro::dynamics::{Position, Velocity};
    use siderust::time::JulianDate;

    #[test]
    fn pseudorange_partials_match_finite_difference() {
        let state = OrbitState::new(
            JulianDate::new(2_451_545.0).to_j2000s(),
            Position::new(7000.0, 0.0, 0.0),
            Velocity::new(0.0, 7.5, 0.0),
        );
        let gps_pos = Position::<GCRS>::new(26_000.0, 1_000.0, 5_000.0);
        let gps_vel = Velocity::<GCRS, VelocityUnit>::new(0.0, 3.0, 0.0);
        let model = GnssCodeModel {
            obs: PseudorangeObs {
                gps_pos_km: gps_pos,
                gps_vel_km_s: gps_vel,
                measured_m: 0.0,
                sigma_m: 1.0,
            },
            clock_bias_index: 0,
        };
        let extra = [0.0];
        let p0 = model.predict(&state, &extra);
        let h = 1e-3;
        for i in 0..3 {
            let rx = state.position.x().value() + if i == 0 { h } else { 0.0 };
            let ry = state.position.y().value() + if i == 1 { h } else { 0.0 };
            let rz = state.position.z().value() + if i == 2 { h } else { 0.0 };
            let s = OrbitState::new(state.epoch, Position::new(rx, ry, rz), state.velocity);
            let p1 = model.predict(&s, &extra);
            let fd = (p1.value - p0.value) / h;
            let analytic = p0.partials.entries.iter().find(|(j, _)| *j == i).unwrap().1;
            assert!(
                (fd - analytic).abs() / analytic.abs() < 1e-3,
                "partial[{i}] fd={fd} analytic={analytic}"
            );
        }
    }
}
