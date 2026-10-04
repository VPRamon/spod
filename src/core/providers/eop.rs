//! [`EarthOrientationProvider`] trait.
//!
//! Concrete implementations typically wrap `siderust::tempoch::eop` or
//! `siderust::astro::eop`.

use std::error::Error;

/// Minimal Earth-orientation interface used by frame-rotation code.
///
/// All inputs are UTC seconds since J2000 UTC. All outputs are expressed
/// in radians (polar motion) or seconds (UT1-UTC).
///
/// # Examples
///
/// ```
/// use spod::core::providers::EarthOrientationProvider;
///
/// struct ZeroEop;
/// impl EarthOrientationProvider for ZeroEop {
///     type Error = std::io::Error;
///     fn ut1_minus_utc(&self, _t: f64) -> Result<f64, Self::Error> { Ok(0.0) }
///     fn polar_motion_x(&self, _t: f64) -> Result<f64, Self::Error> { Ok(0.0) }
///     fn polar_motion_y(&self, _t: f64) -> Result<f64, Self::Error> { Ok(0.0) }
/// }
///
/// let p = ZeroEop;
/// assert_eq!(p.ut1_minus_utc(0.0).unwrap(), 0.0);
/// ```
pub trait EarthOrientationProvider {
    /// Error type for EOP queries.
    type Error: Error + Send + Sync + 'static;

    /// UT1 - UTC offset (seconds) at the given UTC epoch (seconds since J2000 UTC).
    fn ut1_minus_utc(&self, epoch_seconds_utc: f64) -> Result<f64, Self::Error>;
    /// Polar motion x component (radians) at the given UTC epoch.
    fn polar_motion_x(&self, epoch_seconds_utc: f64) -> Result<f64, Self::Error>;
    /// Polar motion y component (radians) at the given UTC epoch.
    fn polar_motion_y(&self, epoch_seconds_utc: f64) -> Result<f64, Self::Error>;
}
