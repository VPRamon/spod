// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Provider bundle trait for dynamic data look-ups inside observation models.

/// Bundled dynamic data sources consumed by observation model implementations.
///
/// The trait is deliberately minimal and object-safe so that observation models
/// can receive it as `&dyn ProviderBundle`.  Concrete implementations are
/// expected to wrap clock-product files, SP3 ephemeris readers, etc.
///
/// Every method has a reasonable default (zero clock bias, `None` state) so
/// that models that do not need a particular service can use
/// [`NullProviderBundle`] or a partial implementation.
///
/// # Examples
///
/// ```
/// use spod::observations::provider_bundle::{ProviderBundle, NullProviderBundle};
///
/// fn check(pb: &dyn ProviderBundle) {
///     assert_eq!(pb.receiver_clock_m(), 0.0);
///     assert!(pb.gnss_satellite_state_gcrs("G01", 2_451_545.0).is_none());
/// }
/// check(&NullProviderBundle);
/// ```
pub trait ProviderBundle: Send + Sync {
    /// GCRS position (km) and velocity (km/s) of a GNSS transmitter at the
    /// given TT Julian date.  Returns `None` if the satellite is unknown or
    /// the epoch is outside the available data window.
    fn gnss_satellite_state_gcrs(&self, prn: &str, epoch_jd: f64) -> Option<([f64; 3], [f64; 3])>;

    /// Satellite clock bias in metres (positive = satellite clock ahead of
    /// GPS time).  Returns `0.0` if no clock product is available.
    fn gnss_satellite_clock_m(&self, prn: &str, epoch_jd: f64) -> f64;

    /// Receiver clock bias in metres.  Returns `0.0` if not estimated.
    fn receiver_clock_m(&self) -> f64;

    /// GCRS position (km) of a ground station at the given TT Julian date.
    /// Used by SLR observation models.  Returns `None` if the station is
    /// unknown.
    fn station_gcrs_km(&self, station_id: &str, epoch_jd: f64) -> Option<[f64; 3]>;
}

// ─── Null implementation ─────────────────────────────────────────────────────

/// A [`ProviderBundle`] that returns zero biases and no satellite/station
/// states.  Useful for unit tests and scenarios where the geometric range
/// is known in advance.
///
/// # Examples
///
/// ```
/// use spod::observations::provider_bundle::{ProviderBundle, NullProviderBundle};
///
/// assert_eq!(NullProviderBundle.receiver_clock_m(), 0.0);
/// assert!(NullProviderBundle.gnss_satellite_state_gcrs("G05", 2_451_545.0).is_none());
/// assert_eq!(NullProviderBundle.gnss_satellite_clock_m("G05", 2_451_545.0), 0.0);
/// assert!(NullProviderBundle.station_gcrs_km("7840", 2_451_545.0).is_none());
/// ```
pub struct NullProviderBundle;

impl ProviderBundle for NullProviderBundle {
    fn gnss_satellite_state_gcrs(
        &self,
        _prn: &str,
        _epoch_jd: f64,
    ) -> Option<([f64; 3], [f64; 3])> {
        None
    }

    fn gnss_satellite_clock_m(&self, _prn: &str, _epoch_jd: f64) -> f64 {
        0.0
    }

    fn receiver_clock_m(&self) -> f64 {
        0.0
    }

    fn station_gcrs_km(&self, _station_id: &str, _epoch_jd: f64) -> Option<[f64; 3]> {
        None
    }
}
