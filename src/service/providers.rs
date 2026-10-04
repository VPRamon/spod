//! Service-owned adapters for external scientific data sources.

use std::error::Error;

/// Minimal ephemeris query interface used by service adapters.
///
/// The trait is intentionally kept at the service boundary: reusable POD
/// models and algorithms use the provider traits in `siderust::pod`.
pub trait EphemerisProvider {
    /// Provider-specific state representation.
    type State;
    /// Provider-specific query error.
    type Error: Error + Send + Sync + 'static;

    /// Return a body state at TDB seconds since J2000.
    fn state(&self, body_naif_id: i32, epoch_seconds_tdb: f64) -> Result<Self::State, Self::Error>;
}
