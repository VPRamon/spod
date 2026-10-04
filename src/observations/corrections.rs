// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Range corrections applied sequentially by a [`CorrectionRegistry`].
//!
//! ## Scientific scope
//!
//! Observation models often need a chain of additive range corrections that
//! do not fit neatly inside a single sensor model (phase-centre offsets,
//! relativistic delays, solid-earth tides).  This module provides a
//! composable registry pattern so the corrections are explicit, auditable, and
//! easily toggled by a configuration layer.
//!
//! ## References
//!
//! - Petit, G., Luzum, B. (Eds.) (2010). *IERS Conventions (2010)*, Chapter 9.
//! - Shapiro, I. I. (1964). Fourth test of general relativity.
//!   *Physical Review Letters*, 13(26), 789–791.
//! - Zhu, S. Y., et al. (1997). Accurate determination of laser-ranging station
//!   coordinates by means of simultaneous observations.
//!   *Bulletin géodésique*, 70(1–2), 73–87.

/// Additive range correction applied by the [`CorrectionRegistry`].
///
/// Each correction receives the current range (in metres) and the GCRS
/// positions of the receiver and the satellite/reflector, and returns a new
/// range that includes the correction.  Corrections are applied in the order
/// they were registered.
///
/// # Examples
///
/// ```
/// use spod::observations::corrections::Correction;
///
/// struct ZeroCorrection;
/// impl Correction for ZeroCorrection {
///     fn name(&self) -> &str { "zero" }
///     fn apply_to_range(&self, range_m: f64, _rx: [f64; 3], _sat: [f64; 3]) -> f64 {
///         range_m
///     }
/// }
///
/// let c = ZeroCorrection;
/// assert_eq!(c.apply_to_range(1_000.0, [0.0; 3], [0.0; 3]), 1_000.0);
/// ```
pub trait Correction: Send + Sync {
    /// Descriptive name used in logs and API snapshots.
    fn name(&self) -> &str;

    /// Apply the correction to `range_m` and return the corrected range.
    ///
    /// * `range_m` — current accumulated range (metres).
    /// * `rx_gcrs_km` — receiver GCRS position (km).
    /// * `sat_gcrs_km` — satellite / reflector GCRS position (km).
    fn apply_to_range(&self, range_m: f64, rx_gcrs_km: [f64; 3], sat_gcrs_km: [f64; 3]) -> f64;
}

// ─── Registry ────────────────────────────────────────────────────────────────

/// Sequential pipeline of [`Correction`] objects.
///
/// Corrections are applied in insertion order, each receiving the output of
/// the previous one.
///
/// # Examples
///
/// ```
/// use spod::observations::corrections::{
///     CorrectionRegistry, ShapiroDelay,
/// };
///
/// let mut reg = CorrectionRegistry::new();
/// reg.push(Box::new(ShapiroDelay));
/// assert_eq!(reg.len(), 1);
///
/// let rx = [6_378.0, 0.0, 0.0_f64];
/// let sat = [26_560.0, 0.0, 0.0_f64];
/// let corrected = reg.apply(20_182_000.0, rx, sat);
/// assert!(corrected > 20_182_000.0); // Shapiro adds a positive delay
/// ```
pub struct CorrectionRegistry {
    corrections: Vec<Box<dyn Correction>>,
}

impl CorrectionRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            corrections: Vec::new(),
        }
    }

    /// Append a correction to the end of the pipeline.
    pub fn push(&mut self, c: Box<dyn Correction>) {
        self.corrections.push(c);
    }

    /// Number of registered corrections.
    pub fn len(&self) -> usize {
        self.corrections.len()
    }

    /// Returns `true` if no corrections are registered.
    pub fn is_empty(&self) -> bool {
        self.corrections.is_empty()
    }

    /// Apply all corrections in order, returning the final corrected range.
    pub fn apply(&self, range_m: f64, rx_gcrs_km: [f64; 3], sat_gcrs_km: [f64; 3]) -> f64 {
        let mut r = range_m;
        for c in &self.corrections {
            r = c.apply_to_range(r, rx_gcrs_km, sat_gcrs_km);
        }
        r
    }

    /// Names of all registered corrections (for diagnostics).
    pub fn names(&self) -> Vec<&str> {
        self.corrections.iter().map(|c| c.name()).collect()
    }
}

impl Default for CorrectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Built-in corrections ────────────────────────────────────────────────────

/// GNSS antenna phase-centre offset (PCO) correction.
///
/// The PCO vector (in the GCRS frame, km) is projected onto the
/// receiver-to-satellite line-of-sight and subtracted from the raw
/// geometric range.
///
/// # Examples
///
/// ```
/// use spod::observations::corrections::{Correction, PhaseCenterOffset};
///
/// // 1.5 mm offset along the receiver radial direction (X in GCRS here)
/// let pco = PhaseCenterOffset::new([0.0000015, 0.0, 0.0]);
/// let rx  = [7_000.0, 0.0, 0.0_f64];
/// let sat = [26_560.0, 0.0, 0.0_f64];
/// let corrected = pco.apply_to_range(20_000_000.0, rx, sat);
/// // The correction should be about ±1.5 mm along the LOS
/// assert!((corrected - 20_000_000.0).abs() < 0.01);
/// ```
pub struct PhaseCenterOffset {
    /// PCO vector in GCRS frame, kilometres.
    offset_gcrs_km: [f64; 3],
}

impl PhaseCenterOffset {
    /// Construct from a GCRS offset vector given in kilometres.
    pub fn new(offset_gcrs_km: [f64; 3]) -> Self {
        Self { offset_gcrs_km }
    }

    /// Construct from a GCRS offset vector given in metres.
    pub fn from_metres(offset_m: [f64; 3]) -> Self {
        Self {
            offset_gcrs_km: [
                offset_m[0] / 1000.0,
                offset_m[1] / 1000.0,
                offset_m[2] / 1000.0,
            ],
        }
    }
}

impl Correction for PhaseCenterOffset {
    fn name(&self) -> &str {
        "PhaseCenterOffset"
    }

    fn apply_to_range(&self, range_m: f64, rx_gcrs_km: [f64; 3], sat_gcrs_km: [f64; 3]) -> f64 {
        let los = [
            sat_gcrs_km[0] - rx_gcrs_km[0],
            sat_gcrs_km[1] - rx_gcrs_km[1],
            sat_gcrs_km[2] - rx_gcrs_km[2],
        ];
        let los_mag = (los[0] * los[0] + los[1] * los[1] + los[2] * los[2]).sqrt();
        if los_mag == 0.0 {
            return range_m;
        }
        let los_hat = [los[0] / los_mag, los[1] / los_mag, los[2] / los_mag];
        let pco_dot = self.offset_gcrs_km[0] * los_hat[0]
            + self.offset_gcrs_km[1] * los_hat[1]
            + self.offset_gcrs_km[2] * los_hat[2];
        // PCO is subtracted: the antenna phase centre is shifted toward
        // the satellite, reducing the apparent range.
        range_m - pco_dot * 1000.0
    }
}

// ─── Shapiro delay ───────────────────────────────────────────────────────────

/// General-relativistic Shapiro delay for signal propagation near Earth.
///
/// The formula is (Shapiro 1964):
/// `ΔR = (2·GM/c²) · ln((r_rx + r_sat + ρ) / (r_rx + r_sat − ρ))`
///
/// For GPS geometry the correction is typically 5–20 mm.
///
/// # Examples
///
/// ```
/// use spod::observations::corrections::{Correction, ShapiroDelay};
///
/// let sd = ShapiroDelay;
/// let rx  = [6_378.0, 0.0, 0.0_f64]; // km
/// let sat = [26_560.0, 0.0, 0.0_f64];
/// let rho_m = (sat[0] - rx[0]) * 1_000.0; // 20 182 km → 20_182_000 m
/// let corrected = sd.apply_to_range(rho_m, rx, sat);
/// let delay = corrected - rho_m;
/// // Shapiro delay is positive and of order 10 mm.
/// assert!(delay > 0.001 && delay < 0.05, "delay={delay}");
/// ```
pub struct ShapiroDelay;

/// Earth gravitational parameter (m³/s²).
const GM_M3_S2: f64 = 3.986_004_418e14;
/// Speed of light (m/s).
const C_M_S: f64 = 299_792_458.0;
/// Pre-computed 2·GM/c² (m).
const TWO_GM_OVER_C2_M: f64 = 2.0 * GM_M3_S2 / (C_M_S * C_M_S);

impl Correction for ShapiroDelay {
    fn name(&self) -> &str {
        "ShapiroDelay"
    }

    fn apply_to_range(&self, range_m: f64, rx_gcrs_km: [f64; 3], sat_gcrs_km: [f64; 3]) -> f64 {
        let r_rx = (rx_gcrs_km[0] * rx_gcrs_km[0]
            + rx_gcrs_km[1] * rx_gcrs_km[1]
            + rx_gcrs_km[2] * rx_gcrs_km[2])
            .sqrt()
            * 1_000.0; // km → m
        let r_sat = (sat_gcrs_km[0] * sat_gcrs_km[0]
            + sat_gcrs_km[1] * sat_gcrs_km[1]
            + sat_gcrs_km[2] * sat_gcrs_km[2])
            .sqrt()
            * 1_000.0;
        let sum = r_rx + r_sat;
        let den = sum - range_m;
        if den > 0.0 {
            range_m + TWO_GM_OVER_C2_M * ((sum + range_m) / den).ln()
        } else {
            range_m
        }
    }
}

// ─── Earth-tide displacement ─────────────────────────────────────────────────

/// Solid-earth tidal displacement projected onto the receiver-to-satellite
/// line-of-sight.
///
/// The displacement vector must be computed externally (e.g. using the IERS
/// Conventions Step-1 solid-tide formulae with lunar/solar positions) and
/// supplied at construction time.  The correction subtracts the LOS component
/// of the displacement from the range, reflecting the fact that the receiver
/// has been physically displaced by the tide.
///
/// # Examples
///
/// ```
/// use spod::observations::corrections::{Correction, EarthTideDisplacement};
///
/// // 3 cm radial (vertical) uplift
/// let rx  = [6_378.0, 0.0, 0.0_f64]; // km, radial direction = X
/// let sat = [26_560.0, 0.0, 0.0_f64];
/// let etd = EarthTideDisplacement::from_metres(0.03, 0.0, 0.0);
/// let corrected = etd.apply_to_range(20_182_000.0, rx, sat);
/// // 3 cm displacement along LOS → range decreases by ~3 cm
/// assert!((corrected - (20_182_000.0 - 0.03)).abs() < 1e-6);
/// ```
pub struct EarthTideDisplacement {
    /// Station/receiver displacement in GCRS, kilometres.
    displacement_gcrs_km: [f64; 3],
}

impl EarthTideDisplacement {
    /// Construct from a GCRS displacement vector (km).
    pub fn from_km(dx: f64, dy: f64, dz: f64) -> Self {
        Self {
            displacement_gcrs_km: [dx, dy, dz],
        }
    }

    /// Construct from a GCRS displacement vector (m).
    pub fn from_metres(dx_m: f64, dy_m: f64, dz_m: f64) -> Self {
        Self::from_km(dx_m / 1000.0, dy_m / 1000.0, dz_m / 1000.0)
    }

    /// Zero displacement (identity correction).
    pub fn zero() -> Self {
        Self::from_km(0.0, 0.0, 0.0)
    }
}

impl Correction for EarthTideDisplacement {
    fn name(&self) -> &str {
        "EarthTideDisplacement"
    }

    fn apply_to_range(&self, range_m: f64, rx_gcrs_km: [f64; 3], sat_gcrs_km: [f64; 3]) -> f64 {
        let los = [
            sat_gcrs_km[0] - rx_gcrs_km[0],
            sat_gcrs_km[1] - rx_gcrs_km[1],
            sat_gcrs_km[2] - rx_gcrs_km[2],
        ];
        let los_mag = (los[0] * los[0] + los[1] * los[1] + los[2] * los[2]).sqrt();
        if los_mag == 0.0 {
            return range_m;
        }
        let los_hat = [los[0] / los_mag, los[1] / los_mag, los[2] / los_mag];
        let disp_los = self.displacement_gcrs_km[0] * los_hat[0]
            + self.displacement_gcrs_km[1] * los_hat[1]
            + self.displacement_gcrs_km[2] * los_hat[2];
        // A positive displacement toward the satellite reduces the apparent range.
        range_m - disp_los * 1000.0
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapiro_positive_and_small() {
        let rx = [6_378.0, 0.0, 0.0_f64];
        let sat = [26_560.0, 0.0, 0.0_f64];
        let rho_m = (sat[0] - rx[0]) * 1_000.0;
        let corrected = ShapiroDelay.apply_to_range(rho_m, rx, sat);
        let delay = corrected - rho_m;
        assert!(delay > 0.005 && delay < 0.025, "Shapiro delay={delay:.4}m");
    }

    #[test]
    fn earth_tide_zero_is_noop() {
        let rx = [7_000.0, 0.0, 0.0_f64];
        let sat = [26_560.0, 1_000.0, 500.0_f64];
        let rho_m = 19_000_000.0;
        let etd = EarthTideDisplacement::zero();
        assert_eq!(etd.apply_to_range(rho_m, rx, sat), rho_m);
    }

    #[test]
    fn registry_applies_in_order() {
        struct Add(f64);
        impl Correction for Add {
            fn name(&self) -> &str {
                "add"
            }
            fn apply_to_range(&self, r: f64, _: [f64; 3], _: [f64; 3]) -> f64 {
                r + self.0
            }
        }
        let mut reg = CorrectionRegistry::new();
        reg.push(Box::new(Add(1.0)));
        reg.push(Box::new(Add(2.0)));
        assert_eq!(reg.apply(0.0, [0.0; 3], [0.0; 3]), 3.0);
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn pco_along_los_subtracts_offset() {
        // Receiver at origin, satellite along +X.  PCO offset 1 km along +X.
        let pco = PhaseCenterOffset::new([0.001, 0.0, 0.0]); // 1 m in km
        let rx = [0.0, 0.0, 0.0_f64];
        let sat = [1_000.0, 0.0, 0.0_f64];
        let corrected = pco.apply_to_range(1_000_000.0, rx, sat);
        assert!((corrected - (1_000_000.0 - 1.0)).abs() < 1e-9);
    }
}
