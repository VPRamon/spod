// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Low-thrust ΔV bookkeeping.
//!
//! This module accumulates the per-arc and total propellant cost of a
//! sequence of constant-thrust [`ThrustArc`]s using the ideal Tsiolkovsky
//! rocket equation:
//!
//! ```text
//! ṁ  = F / (Isp · g₀)                 [kg/s]
//! Δm = ṁ · Δt                         [kg]
//! Δv = Isp · g₀ · ln(m₀ / (m₀ − Δm))  [m/s]
//! ```
//!
//! The bookkeeping assumes the spacecraft mass at the start of each arc is
//! the running mass (i.e. arcs deplete the same propellant tank). This is
//! the convention POD uses when reconciling estimated ΔV against telemetry,
//! and is what mission-design pipelines need before they hand off to a
//! detailed Sims-Flanagan or Pontryagin solver (which lives elsewhere).
//!
//! For thrust-direction modelling and the per-step acceleration vector, see
//! the [`crate::dynamics::thrust`] module.

use super::thrust::{mass_flow_rate, ManeuverError, ThrustArc, G0_M_PER_S2};

/// One log entry recording the propellant cost of a single arc.
///
/// `dry_mass_kg_after = wet_mass_kg_before - propellant_used_kg`.
#[derive(Debug, Clone, PartialEq)]
pub struct LowThrustRecord {
    /// Spacecraft mass at the start of the arc, in kilograms.
    pub wet_mass_kg_before: f64,
    /// Propellant consumed during the arc, in kilograms.
    pub propellant_used_kg: f64,
    /// Ideal Tsiolkovsky ΔV imparted by the arc, in metres per second.
    pub delta_v_m_per_s: f64,
    /// Burn duration in seconds (`stop - start`).
    pub duration_s: f64,
}

/// Cumulative low-thrust propellant ledger.
///
/// Append [`ThrustArc`]s in chronological order; each call updates the
/// running spacecraft mass and appends a [`LowThrustRecord`].
///
/// # Example
///
/// ```
/// use siderust_pod::dynamics::{LowThrustLog, ThrustArc};
/// use qtty::force::Newtons;
/// use tempoch::{Time, UTC};
/// use chrono::{TimeZone, Utc};
///
/// let t0 = Time::<UTC>::try_from_chrono(Utc.timestamp_opt(946_728_000, 0).single().unwrap()).unwrap();
/// let t1 = Time::<UTC>::try_from_chrono(Utc.timestamp_opt(946_728_600, 0).single().unwrap()).unwrap();
/// let arc = ThrustArc::with_normalised_direction(
///     t0, t1, Newtons::new(0.5), 1500.0, [1.0, 0.0, 0.0],
/// ).unwrap();
///
/// let mut log = LowThrustLog::new(100.0);
/// log.append(&arc).unwrap();
/// assert!(log.total_delta_v_m_per_s() > 0.0);
/// assert!(log.current_mass_kg() < 100.0);
/// ```
#[derive(Debug, Clone)]
pub struct LowThrustLog {
    current_mass_kg: f64,
    total_delta_v_m_per_s: f64,
    total_propellant_kg: f64,
    records: Vec<LowThrustRecord>,
}

impl LowThrustLog {
    /// Create a new empty ledger seeded with the spacecraft wet mass.
    ///
    /// # Errors
    ///
    /// Returns [`ManeuverError::NonPositiveMass`] if `initial_wet_mass_kg`
    /// is non-positive or non-finite.
    pub fn try_new(initial_wet_mass_kg: f64) -> Result<Self, ManeuverError> {
        if !initial_wet_mass_kg.is_finite() || initial_wet_mass_kg <= 0.0 {
            return Err(ManeuverError::NonPositiveMass(initial_wet_mass_kg));
        }
        Ok(Self {
            current_mass_kg: initial_wet_mass_kg,
            total_delta_v_m_per_s: 0.0,
            total_propellant_kg: 0.0,
            records: Vec::new(),
        })
    }

    /// Convenience constructor that panics on a non-positive mass.
    ///
    /// Prefer [`LowThrustLog::try_new`] in library code.
    pub fn new(initial_wet_mass_kg: f64) -> Self {
        Self::try_new(initial_wet_mass_kg).expect("initial wet mass must be > 0")
    }

    /// Spacecraft mass after every appended arc has been deducted.
    pub fn current_mass_kg(&self) -> f64 {
        self.current_mass_kg
    }

    /// Cumulative ideal Tsiolkovsky ΔV across all appended arcs, in m/s.
    pub fn total_delta_v_m_per_s(&self) -> f64 {
        self.total_delta_v_m_per_s
    }

    /// Cumulative propellant consumed across all appended arcs, in kg.
    pub fn total_propellant_kg(&self) -> f64 {
        self.total_propellant_kg
    }

    /// Per-arc records, in append order.
    pub fn records(&self) -> &[LowThrustRecord] {
        &self.records
    }

    /// Append `arc`, deducting its propellant from the running mass and
    /// adding its Tsiolkovsky ΔV to the total.
    ///
    /// # Errors
    ///
    /// * [`ManeuverError::NonPositiveIsp`] if the arc's specific impulse
    ///   is non-positive.
    /// * [`ManeuverError::EmptyInterval`] if the arc has `stop <= start`
    ///   (also caught at construction by
    ///   [`ThrustArc::with_normalised_direction`]).
    /// * [`ManeuverError::NonPositiveMass`] if the arc would deplete the
    ///   propellant tank below zero (i.e. `Δm >= m_current`).
    pub fn append(&mut self, arc: &ThrustArc) -> Result<&LowThrustRecord, ManeuverError> {
        let duration_s = duration_seconds(arc)?;
        let mdot = mass_flow_rate(arc.thrust, arc.isp_seconds)?;
        let propellant_used_kg = mdot * duration_s;
        let wet_mass_kg_before = self.current_mass_kg;
        let mass_after = wet_mass_kg_before - propellant_used_kg;
        if mass_after <= 0.0 || mass_after.is_nan() {
            return Err(ManeuverError::NonPositiveMass(mass_after));
        }
        let delta_v_m_per_s =
            arc.isp_seconds * G0_M_PER_S2 * (wet_mass_kg_before / mass_after).ln();
        self.current_mass_kg = mass_after;
        self.total_delta_v_m_per_s += delta_v_m_per_s;
        self.total_propellant_kg += propellant_used_kg;
        self.records.push(LowThrustRecord {
            wet_mass_kg_before,
            propellant_used_kg,
            delta_v_m_per_s,
            duration_s,
        });
        Ok(self.records.last().expect("just pushed"))
    }
}

fn duration_seconds(arc: &ThrustArc) -> Result<f64, ManeuverError> {
    let dt = arc.stop - arc.start;
    let secs = dt.value();
    if secs <= 0.0 || secs.is_nan() {
        return Err(ManeuverError::EmptyInterval {
            start: arc.start,
            stop: arc.stop,
        });
    }
    Ok(secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use qtty::force::Newtons;
    use tempoch::{Time, UTC};

    fn epoch(secs: i64) -> Time<UTC> {
        Time::<UTC>::try_from_chrono(Utc.timestamp_opt(946_728_000 + secs, 0).single().unwrap())
            .unwrap()
    }

    fn arc(start: i64, stop: i64, thrust_n: f64, isp: f64) -> ThrustArc {
        ThrustArc::with_normalised_direction(
            epoch(start),
            epoch(stop),
            Newtons::new(thrust_n),
            isp,
            [1.0, 0.0, 0.0],
        )
        .unwrap()
    }

    #[test]
    fn empty_log_is_zero() {
        let log = LowThrustLog::new(100.0);
        assert_eq!(log.current_mass_kg(), 100.0);
        assert_eq!(log.total_delta_v_m_per_s(), 0.0);
        assert_eq!(log.total_propellant_kg(), 0.0);
        assert!(log.records().is_empty());
    }

    #[test]
    fn try_new_rejects_non_positive_mass() {
        assert!(matches!(
            LowThrustLog::try_new(0.0),
            Err(ManeuverError::NonPositiveMass(_))
        ));
        assert!(matches!(
            LowThrustLog::try_new(-1.0),
            Err(ManeuverError::NonPositiveMass(_))
        ));
    }

    #[test]
    fn single_arc_matches_tsiolkovsky() {
        let mut log = LowThrustLog::new(100.0);
        let a = arc(0, 600, 0.5, 1500.0);
        log.append(&a).unwrap();
        let mdot = 0.5 / (1500.0 * G0_M_PER_S2);
        let dm = mdot * 600.0;
        let expected_dv = 1500.0 * G0_M_PER_S2 * (100.0 / (100.0 - dm)).ln();
        assert!(
            (log.total_delta_v_m_per_s() - expected_dv).abs() < 1e-6,
            "got {} expected {}",
            log.total_delta_v_m_per_s(),
            expected_dv,
        );
        assert!((log.current_mass_kg() - (100.0 - dm)).abs() < 1e-9);
        assert!((log.total_propellant_kg() - dm).abs() < 1e-9);
        assert_eq!(log.records().len(), 1);
    }

    #[test]
    fn multi_arc_sums_correctly() {
        let mut log = LowThrustLog::new(100.0);
        log.append(&arc(0, 600, 0.5, 1500.0)).unwrap();
        log.append(&arc(600, 1200, 0.5, 1500.0)).unwrap();
        assert_eq!(log.records().len(), 2);
        // ΔV should be very nearly 2× the single-arc ΔV (slightly larger
        // because mass decreases between arcs).
        let single = LowThrustLog::new(100.0)
            .also_append(arc(0, 600, 0.5, 1500.0))
            .total_delta_v_m_per_s();
        assert!(log.total_delta_v_m_per_s() > 2.0 * single - 1e-9);
    }

    #[test]
    fn rejects_propellant_overdraw() {
        let mut log = LowThrustLog::new(0.5);
        let r = log.append(&arc(0, 600, 100.0, 100.0));
        assert!(matches!(r, Err(ManeuverError::NonPositiveMass(_))));
    }

    // small fluent helper kept inside the test module
    impl LowThrustLog {
        fn also_append(mut self, arc: ThrustArc) -> Self {
            self.append(&arc).unwrap();
            self
        }
    }
}
