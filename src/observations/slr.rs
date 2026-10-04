//! # Satellite laser ranging observation model
//!
//! ## Scientific scope
//!
//! Satellite laser ranging measures a two-way light-time between a ground
//! station and a spacecraft retroreflector. This module provides a minimal
//! physically interpretable range model suitable for self-consistency
//! checks and early POD validation against synthetic or controlled CRD
//! data.
//!
//! The regime is intentionally limited: the model omits full atmospheric
//! delay, relativity, station eccentricity, and advanced timing
//! calibrations. It is therefore appropriate for integration tests and
//! architectural plumbing, not for millimetre-accurate operational SLR
//! analysis.
//!
//! ## Technical scope
//!
//! The public items are `SlrRangeObs` and `SlrRangeModel`. Given an
//! inertial spacecraft state, an Earth-fixed station location, and frame-
//! transform support, the model returns a scalar two-way range prediction
//! and the corresponding partials for the estimation layer.
//!
//! Station data ingestion, CRD parsing, and QC summarization are handled by
//! the IO and QC crates rather than here.
//!
//! ## References
//!
//! - Degnan, J. J. (1993). Millimeter accuracy satellite laser ranging: a
//!   review. Contributions of Space Geodesy to Geodynamics: Technology, 25,
//!   133-162.
//! - Pearlman, M. R., Noll, C. E., et al. (2019). The ILRS: Current status
//!   and future prospects. Journal of Geodesy, 93, 2161-2180.
use super::model::{MeasurementModel, Partials, Prediction};
use siderust::affn::cartesian::Displacement;
use siderust::astro::dynamics::{OrbitState, Position};
use siderust::coordinates::frames::GCRS;
use siderust::qtty::unit::{Kilometer, Second};
use siderust::qtty::velocity::C;
use siderust::qtty::Per;
/// One SLR range observation (two-way time-of-flight converted to metres).
#[derive(Debug, Clone)]
pub struct SlrRangeObs {
    /// Two-way range, metres.
    pub range_m: f64,
    /// Standard deviation of the observation, metres.
    pub sigma_m: f64,
}

/// SLR two-way range model.
#[derive(Debug, Clone)]
pub struct SlrRangeModel {
    /// Station position in the same inertial frame as the satellite state, km.
    pub station_inertial_km: Position<GCRS>,
    /// Constant tropospheric range delay (metres, applied symmetrically).
    pub trop_bias_m: f64,
    /// Assumed measurement standard deviation (metres).
    pub sigma_m: f64,
    /// Index of an additive station-bias parameter inside `extra_params`,
    /// or `None` if no station bias is being estimated.
    pub station_bias_index: Option<usize>,
}

impl SlrRangeModel {
    /// New model with no bias estimation.
    pub fn new(station_inertial_km: Position<GCRS>, trop_bias_m: f64, sigma_m: f64) -> Self {
        Self {
            station_inertial_km,
            trop_bias_m,
            sigma_m,
            station_bias_index: None,
        }
    }
}

impl MeasurementModel for SlrRangeModel {
    fn predict(&self, state: &OrbitState, extra: &[f64]) -> Prediction {
        let c_km_s = C.to::<Per<Kilometer, Second>>().value();

        let r_sat = state.position;
        let vx = state.velocity.x().value();
        let vy = state.velocity.y().value();
        let vz = state.velocity.z().value();
        let r_sta = self.station_inertial_km;

        let mut down_dt = 0.0_f64;
        for _ in 0..3 {
            let r_bounce = r_sat
                - Displacement::<GCRS, Kilometer>::new(vx * down_dt, vy * down_dt, vz * down_dt);
            let d_km = (r_bounce - r_sta).magnitude().value();
            down_dt = d_km / c_km_s;
        }
        let r_bounce =
            r_sat - Displacement::<GCRS, Kilometer>::new(vx * down_dt, vy * down_dt, vz * down_dt);

        let mut up_dt = 0.0_f64;
        for _ in 0..3 {
            let r_arrive =
                r_bounce + Displacement::<GCRS, Kilometer>::new(vx * up_dt, vy * up_dt, vz * up_dt);
            let d_km = (r_arrive - r_sta).magnitude().value();
            up_dt = d_km / c_km_s;
        }
        let r_arrive =
            r_bounce + Displacement::<GCRS, Kilometer>::new(vx * up_dt, vy * up_dt, vz * up_dt);

        let down_vec = r_bounce - r_sta;
        let up_vec = r_arrive - r_sta;
        let down_km = down_vec.magnitude().value();
        let up_km = up_vec.magnitude().value();
        let mut value = (down_km + up_km) * 1000.0 + 2.0 * self.trop_bias_m;

        let mut entries = Vec::with_capacity(4);
        let los_down = if down_km > 0.0 {
            [
                down_vec.x().value() / down_km,
                down_vec.y().value() / down_km,
                down_vec.z().value() / down_km,
            ]
        } else {
            [0.0; 3]
        };
        let los_up = if up_km > 0.0 {
            [
                up_vec.x().value() / up_km,
                up_vec.y().value() / up_km,
                up_vec.z().value() / up_km,
            ]
        } else {
            [0.0; 3]
        };
        for i in 0..3 {
            // ∂range/∂r_sat — sum of downlink and uplink unit vectors,
            // converted to per-km because state position is in km.
            let g = (los_down[i] + los_up[i]) * 1000.0;
            entries.push((i, g));
        }

        if let Some(idx) = self.station_bias_index {
            let global = 6 + idx;
            value += extra.get(idx).copied().unwrap_or(0.0);
            entries.push((global, 1.0));
        }

        Prediction {
            value,
            partials: Partials { entries },
        }
    }

    fn sigma(&self) -> f64 {
        self.sigma_m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::astro::dynamics::{Position, Velocity};
    use siderust::time::JulianDate;

    #[test]
    fn predicts_two_way_range_at_rest() {
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0).to_j2000s(),
            Position::new(0.0, 0.0, 7378.137),
            Velocity::new(0.0, 0.0, 0.0),
        );
        let m = SlrRangeModel::new(Position::<GCRS>::new(0.0, 0.0, 6378.137), 0.0, 0.01);
        let p = m.predict(&s, &[]);
        assert!((p.value - 2_000_000.0).abs() < 1e-3);
        assert_eq!(p.partials.entries.len(), 3);
    }

    #[test]
    fn partial_matches_finite_difference() {
        let s = OrbitState::new(
            JulianDate::new(2_451_545.0).to_j2000s(),
            Position::new(7100.0, 200.0, 50.0),
            Velocity::new(0.5, 7.5, 0.1),
        );
        let m = SlrRangeModel::new(Position::<GCRS>::new(6378.0, 100.0, 0.0), 0.0, 0.01);
        let pred = m.predict(&s, &[]);
        let h = 1e-3;
        for (axis, idx) in [(0, 0), (1, 1), (2, 2)] {
            let mut up = s;
            let mut dn = s;
            let rx = s.position.x().value();
            let ry = s.position.y().value();
            let rz = s.position.z().value();
            match axis {
                0 => {
                    up.position = Position::<GCRS>::new(rx + h, ry, rz);
                    dn.position = Position::<GCRS>::new(rx - h, ry, rz);
                }
                1 => {
                    up.position = Position::<GCRS>::new(rx, ry + h, rz);
                    dn.position = Position::<GCRS>::new(rx, ry - h, rz);
                }
                _ => {
                    up.position = Position::<GCRS>::new(rx, ry, rz + h);
                    dn.position = Position::<GCRS>::new(rx, ry, rz - h);
                }
            }
            let fd = (m.predict(&up, &[]).value - m.predict(&dn, &[]).value) / (2.0 * h);
            let analytic = pred
                .partials
                .entries
                .iter()
                .find(|(i, _)| *i == idx)
                .unwrap()
                .1;
            assert!(
                (fd - analytic).abs() < 1e-2,
                "axis {axis}: fd={fd}, analytic={analytic}"
            );
        }
    }
}
