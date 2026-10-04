//! # LISA orbit-file reader and ephemeris provider
//!
//! ## Scientific scope
//!
//! The ESA LISA mission uses three spacecraft in a heliocentric triangular
//! formation.  Their reference trajectories are distributed as CCSDS OEM v2.0
//! files (Orbit Ephemeris Message, ASCII KVN layout) from the ESA LISA orbit
//! repository (<https://github.com/esa/lisa-orbit-files>).  One file per
//! spacecraft is produced; each carries a single segment with state vectors in
//! the `SUN`-centred, EME2000-aligned, TDB-time frame.
//!
//! This module provides:
//!
//! - [`LisaOrbitReader`]: parses a single OEM file (one spacecraft) into a
//!   typed [`LisaOrbit`].
//! - [`LisaOrbitSet`]: holds the three loaded spacecraft orbits together.
//! - [`LisaEphemerisProvider`][]: implements
//!   [`crate::core::providers::EphemerisProvider`] via **cubic Hermite**
//!   interpolation over the tabulated state vectors.
//!
//! ## Interpolation
//!
//! Cubic Hermite interpolation uses the two bracketing tabulated states
//! (position and velocity) to construct a third-degree polynomial that exactly
//! reproduces the endpoint positions and velocities.  Given states
//! `(t₀, p₀, v₀)` and `(t₁, p₁, v₁)`, the normalised parameter
//! `τ = (t − t₀) / (t₁ − t₀)` and the Hermite basis:
//!
//! ```text
//! h₀₀(τ) =  2τ³ − 3τ² + 1
//! h₁₀(τ) =   τ³ − 2τ² + τ    (multiplied by dt = t₁ − t₀)
//! h₀₁(τ) = −2τ³ + 3τ²
//! h₁₁(τ) =   τ³ −  τ²         (multiplied by dt)
//!
//! p(τ) = h₀₀·p₀ + h₁₀·dt·v₀ + h₀₁·p₁ + h₁₁·dt·v₁
//! v(τ) = dp/dt  (derivative of the cubic divided by dt)
//! ```
//!
//! This is exact for constant-velocity (linear) motion and provides a smooth,
//! energy-consistent interpolant for the slowly varying LISA trajectories.
//!
//! ## References
//!
//! - Consultative Committee for Space Data Systems. (2019). Orbit Data
//!   Messages, CCSDS 502.0-B-3.
//! - Martens, W., Joffre, E. (2021). Trajectory Design for the ESA LISA
//!   Mission. *J. Astronaut. Sci.*, 68, 402–443.
//!   <https://doi.org/10.1007/s40295-021-00263-2>

use super::{oem::read_oem, PodIoError};
use crate::core::providers::EphemerisProvider;
use siderust::affn::cartesian;
use siderust::affn::centers::{AffineCenter, ReferenceCenter};
use siderust::affn::frames::EME2000;
use siderust::qtty::unit::Kilometer;
use siderust::tempoch::{JulianDate, Time, TDB};
use std::io::Read;

// ── KmPerSecond local alias ───────────────────────────────────────────────────

/// Velocity unit: kilometres per second.
type KmPerSecond = siderust::qtty::Per<Kilometer, siderust::qtty::unit::Second>;

// ── HeliocentricCenter ────────────────────────────────────────────────────────

/// Heliocentric reference centre for LISA orbit positions (Sun's centre of mass).
///
/// LISA OEM files use `CENTER_NAME = SUN`.  This marker type anchors the
/// type-level centre constraint for [`LisaOrbitPoint::position`].
///
/// # Examples
///
/// ```
/// use siderust_pod::io::lisa::HeliocentricCenter;
/// use siderust::affn::centers::ReferenceCenter;
/// assert_eq!(HeliocentricCenter::center_name(), "Heliocentric");
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct HeliocentricCenter;

impl ReferenceCenter for HeliocentricCenter {
    type Params = ();
    fn center_name() -> &'static str {
        "Heliocentric"
    }
}

impl AffineCenter for HeliocentricCenter {}

// ── Typed position / velocity aliases ─────────────────────────────────────────

/// Heliocentric, EME2000-framed position in kilometres.
pub type LisaPosition = cartesian::Position<HeliocentricCenter, EME2000, Kilometer>;

/// EME2000-framed velocity in km/s.
pub type LisaVelocity = cartesian::Velocity<EME2000, KmPerSecond>;

// ── LisaSpacecraftId ─────────────────────────────────────────────────────────

/// Identifier for one of the three LISA spacecraft.
///
/// The numeric suffix matches the OEM file extension convention:
/// `.oem1` → [`SC1`], `.oem2` → [`SC2`], `.oem3` → [`SC3`].
///
/// The NAIF-style integer codes (`−1001`, `−1002`, `−1003`) are used for
/// [`LisaEphemerisProvider`].
///
/// [`SC1`]: LisaSpacecraftId::SC1
/// [`SC2`]: LisaSpacecraftId::SC2
/// [`SC3`]: LisaSpacecraftId::SC3
///
/// # Examples
///
/// ```
/// use siderust_pod::io::lisa::LisaSpacecraftId;
/// assert_eq!(LisaSpacecraftId::SC1.naif_id(), -1001);
/// assert_eq!(LisaSpacecraftId::from_naif_id(-1002), Some(LisaSpacecraftId::SC2));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LisaSpacecraftId {
    /// LISA spacecraft 1.
    SC1,
    /// LISA spacecraft 2.
    SC2,
    /// LISA spacecraft 3.
    SC3,
}

impl LisaSpacecraftId {
    /// Return the NAIF-style integer body ID for this spacecraft.
    ///
    /// # Examples
    ///
    /// ```
    /// use siderust_pod::io::lisa::LisaSpacecraftId;
    /// assert_eq!(LisaSpacecraftId::SC3.naif_id(), -1003);
    /// ```
    pub fn naif_id(self) -> i32 {
        match self {
            LisaSpacecraftId::SC1 => -1001,
            LisaSpacecraftId::SC2 => -1002,
            LisaSpacecraftId::SC3 => -1003,
        }
    }

    /// Resolve a NAIF-style body ID back to a [`LisaSpacecraftId`], or
    /// `None` if the ID is not a LISA spacecraft.
    ///
    /// # Examples
    ///
    /// ```
    /// use siderust_pod::io::lisa::LisaSpacecraftId;
    /// assert_eq!(LisaSpacecraftId::from_naif_id(-1001), Some(LisaSpacecraftId::SC1));
    /// assert_eq!(LisaSpacecraftId::from_naif_id(399), None);
    /// ```
    pub fn from_naif_id(id: i32) -> Option<Self> {
        match id {
            -1001 => Some(LisaSpacecraftId::SC1),
            -1002 => Some(LisaSpacecraftId::SC2),
            -1003 => Some(LisaSpacecraftId::SC3),
            _ => None,
        }
    }
}

// ── LisaOrbitPoint ────────────────────────────────────────────────────────────

/// A single tabulated state vector from a LISA OEM file.
///
/// # Examples
///
/// ```
/// use siderust_pod::io::lisa::{LisaOrbitReader, LisaSpacecraftId};
///
/// let oem = "\
/// CCSDS_OEM_VERS = 2.0\n\
/// CREATION_DATE  = 2024-01-01T00:00:00\n\
/// ORIGINATOR     = TEST\n\
/// \n\
/// META_START\n\
/// OBJECT_NAME          = LISA-1\n\
/// OBJECT_ID            = -1001\n\
/// CENTER_NAME          = SUN\n\
/// REF_FRAME            = EME2000\n\
/// TIME_SYSTEM          = TDB\n\
/// START_TIME           = 2036-02-12T12:00:00.000\n\
/// STOP_TIME            = 2036-02-12T12:00:00.000\n\
/// META_STOP\n\
/// \n\
/// 2036-02-12T12:00:00.000  100000000.0  50000000.0  10000000.0  10.0  20.0  5.0\n";
///
/// let orbit = LisaOrbitReader::from_str(oem, LisaSpacecraftId::SC1).unwrap();
/// let pt = &orbit.points[0];
/// assert!((pt.position.x().value() - 100000000.0).abs() < 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct LisaOrbitPoint {
    /// Epoch in the Barycentric Dynamical Time (TDB) scale.
    pub epoch: Time<TDB>,
    /// Heliocentric Cartesian position in the EME2000 frame, km.
    pub position: LisaPosition,
    /// Velocity in the EME2000 frame, km/s.
    pub velocity: LisaVelocity,
    /// TDB seconds since J2000.0 (cached for interpolation; not public API).
    pub(crate) epoch_j2000_s: f64,
}

// ── LisaOrbit ─────────────────────────────────────────────────────────────────

/// Tabulated orbit for one LISA spacecraft, loaded from an OEM file.
///
/// # Examples
///
/// ```
/// use siderust_pod::io::lisa::{LisaOrbitReader, LisaSpacecraftId};
/// let raw = std::fs::read_to_string(
///     concat!(env!("CARGO_MANIFEST_DIR"), "/test-data/lisa/lisa_orbit_sample.oem1")
/// ).unwrap();
/// let orbit = LisaOrbitReader::from_str(&raw, LisaSpacecraftId::SC1).unwrap();
/// assert_eq!(orbit.spacecraft_id, LisaSpacecraftId::SC1);
/// assert_eq!(orbit.points.len(), 10);
/// ```
#[derive(Debug, Clone)]
pub struct LisaOrbit {
    /// Which spacecraft this orbit belongs to.
    pub spacecraft_id: LisaSpacecraftId,
    /// Tabulated state vectors in ascending epoch order.
    pub points: Vec<LisaOrbitPoint>,
}

// ── LisaOrbitReader ───────────────────────────────────────────────────────────

/// Reader for CCSDS OEM v2.0 LISA orbit files.
///
/// Each LISA orbit file covers one spacecraft and contains a single metadata
/// segment with `CENTER_NAME = SUN`, `REF_FRAME = EME2000`, and
/// `TIME_SYSTEM = TDB`.
///
/// # Examples
///
/// ```
/// use siderust_pod::io::lisa::{LisaOrbitReader, LisaSpacecraftId};
///
/// let raw = std::fs::read_to_string(
///     concat!(env!("CARGO_MANIFEST_DIR"), "/test-data/lisa/lisa_orbit_sample.oem1"),
/// ).unwrap();
/// let orbit = LisaOrbitReader::from_str(&raw, LisaSpacecraftId::SC1).unwrap();
/// assert_eq!(orbit.points.len(), 10);
/// ```
pub struct LisaOrbitReader;

impl LisaOrbitReader {
    /// Parse a LISA OEM file from a [`Read`] source.
    ///
    /// The spacecraft identity is provided by the caller (it matches the file
    /// extension: `.oem1` → [`LisaSpacecraftId::SC1`], etc.).
    ///
    /// # Errors
    ///
    /// Returns [`PodIoError`] when the underlying OEM parser fails or when an
    /// epoch value cannot be converted to a finite TDB [`Time`].
    ///
    /// # Examples
    ///
    /// ```
    /// use siderust_pod::io::lisa::{LisaOrbitReader, LisaSpacecraftId};
    /// use std::fs::File;
    ///
    /// let path = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data/lisa/lisa_orbit_sample.oem1");
    /// let f = File::open(path).unwrap();
    /// let orbit = LisaOrbitReader::read(f, LisaSpacecraftId::SC1).unwrap();
    /// assert_eq!(orbit.points.len(), 10);
    /// ```
    pub fn read<R: Read>(
        reader: R,
        spacecraft_id: LisaSpacecraftId,
    ) -> Result<LisaOrbit, PodIoError> {
        let oem_file = read_oem(reader)?;
        Self::convert(oem_file, spacecraft_id)
    }

    /// Parse a LISA OEM file from a string slice.
    ///
    /// # Errors
    ///
    /// Same as [`LisaOrbitReader::read`].
    ///
    /// # Examples
    ///
    /// ```
    /// use siderust_pod::io::lisa::{LisaOrbitReader, LisaSpacecraftId};
    ///
    /// let oem = "\
    /// CCSDS_OEM_VERS = 2.0\n\
    /// CREATION_DATE  = 2024-01-01T00:00:00\n\
    /// ORIGINATOR     = TEST\n\
    /// \n\
    /// META_START\n\
    /// OBJECT_NAME          = LISA-1\n\
    /// OBJECT_ID            = -1001\n\
    /// CENTER_NAME          = SUN\n\
    /// REF_FRAME            = EME2000\n\
    /// TIME_SYSTEM          = TDB\n\
    /// START_TIME           = 2036-02-12T12:00:00.000\n\
    /// STOP_TIME            = 2036-02-12T12:00:00.000\n\
    /// META_STOP\n\
    /// \n\
    /// 2036-02-12T12:00:00.000  100000000.0  50000000.0  10000000.0  10.0  20.0  5.0\n";
    ///
    /// let orbit = LisaOrbitReader::from_str(oem, LisaSpacecraftId::SC1).unwrap();
    /// assert_eq!(orbit.points.len(), 1);
    /// ```
    pub fn from_str(s: &str, spacecraft_id: LisaSpacecraftId) -> Result<LisaOrbit, PodIoError> {
        Self::read(s.as_bytes(), spacecraft_id)
    }

    fn convert(
        oem_file: crate::io::oem::OemFile,
        spacecraft_id: LisaSpacecraftId,
    ) -> Result<LisaOrbit, PodIoError> {
        // Collect all states across all segments in the file.
        let mut points: Vec<LisaOrbitPoint> = Vec::new();
        for segment in oem_file.segments {
            for state in segment.states {
                let epoch_j2000_s = jd_to_j2000_seconds(state.epoch_jd);
                let epoch = jd_to_time_tdb(state.epoch_jd)?;
                let [x, y, z] = state.position_km;
                let [vx, vy, vz] = state.velocity_km_s;
                let position = LisaPosition::new(
                    siderust::qtty::Kilometers::new(x),
                    siderust::qtty::Kilometers::new(y),
                    siderust::qtty::Kilometers::new(z),
                );
                let velocity = LisaVelocity::new(
                    siderust::qtty::Quantity::<KmPerSecond>::new(vx),
                    siderust::qtty::Quantity::<KmPerSecond>::new(vy),
                    siderust::qtty::Quantity::<KmPerSecond>::new(vz),
                );
                points.push(LisaOrbitPoint {
                    epoch,
                    epoch_j2000_s,
                    position,
                    velocity,
                });
            }
        }
        if points.is_empty() {
            return Err(PodIoError::Format(
                "lisa: OEM file contains no state vectors".into(),
            ));
        }
        // Sort by epoch to guarantee monotonicity for binary-search interpolation.
        points.sort_by(|a, b| a.epoch_j2000_s.partial_cmp(&b.epoch_j2000_s).unwrap());
        Ok(LisaOrbit {
            spacecraft_id,
            points,
        })
    }
}

// ── LisaOrbitSet ─────────────────────────────────────────────────────────────

/// All three LISA spacecraft orbits loaded together.
///
/// # Examples
///
/// ```
/// use siderust_pod::io::lisa::{LisaOrbitReader, LisaOrbitSet, LisaSpacecraftId};
///
/// let root = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data/lisa");
/// let load = |name: &str, sc: LisaSpacecraftId| {
///     let raw = std::fs::read_to_string(format!("{root}/{name}")).unwrap();
///     LisaOrbitReader::from_str(&raw, sc).unwrap()
/// };
/// let set = LisaOrbitSet {
///     sc1: load("lisa_orbit_sample.oem1", LisaSpacecraftId::SC1),
///     sc2: load("lisa_orbit_sample.oem2", LisaSpacecraftId::SC2),
///     sc3: load("lisa_orbit_sample.oem3", LisaSpacecraftId::SC3),
/// };
/// assert_eq!(set.sc1.points.len(), 10);
/// ```
#[derive(Debug, Clone)]
pub struct LisaOrbitSet {
    /// Orbit for spacecraft 1.
    pub sc1: LisaOrbit,
    /// Orbit for spacecraft 2.
    pub sc2: LisaOrbit,
    /// Orbit for spacecraft 3.
    pub sc3: LisaOrbit,
}

impl LisaOrbitSet {
    /// Retrieve the orbit for the given spacecraft.
    ///
    /// # Examples
    ///
    /// ```
    /// use siderust_pod::io::lisa::{LisaOrbitReader, LisaOrbitSet, LisaSpacecraftId};
    ///
    /// let root = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data/lisa");
    /// let load = |name: &str, sc: LisaSpacecraftId| {
    ///     let raw = std::fs::read_to_string(format!("{root}/{name}")).unwrap();
    ///     LisaOrbitReader::from_str(&raw, sc).unwrap()
    /// };
    /// let set = LisaOrbitSet {
    ///     sc1: load("lisa_orbit_sample.oem1", LisaSpacecraftId::SC1),
    ///     sc2: load("lisa_orbit_sample.oem2", LisaSpacecraftId::SC2),
    ///     sc3: load("lisa_orbit_sample.oem3", LisaSpacecraftId::SC3),
    /// };
    /// assert_eq!(set.orbit(LisaSpacecraftId::SC2).spacecraft_id, LisaSpacecraftId::SC2);
    /// ```
    pub fn orbit(&self, sc: LisaSpacecraftId) -> &LisaOrbit {
        match sc {
            LisaSpacecraftId::SC1 => &self.sc1,
            LisaSpacecraftId::SC2 => &self.sc2,
            LisaSpacecraftId::SC3 => &self.sc3,
        }
    }
}

// ── LisaEphemerisProvider ─────────────────────────────────────────────────────

/// Ephemeris provider backed by the three LISA tabulated orbit files.
///
/// Implements [`EphemerisProvider`] using **cubic Hermite interpolation** over
/// the tabulated state vectors (see the [module documentation](self) for the
/// mathematical details).
///
/// # Spacecraft identification
///
/// The `body_naif_id` parameter of [`EphemerisProvider::state`] maps to:
///
/// | NAIF ID | Spacecraft |
/// |---------|-----------|
/// | −1001   | LISA SC1  |
/// | −1002   | LISA SC2  |
/// | −1003   | LISA SC3  |
///
/// # Examples
///
/// ```
/// use siderust_pod::io::lisa::{LisaOrbitReader, LisaOrbitSet, LisaEphemerisProvider, LisaSpacecraftId};
/// use siderust_pod::core::providers::EphemerisProvider;
///
/// let root = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data/lisa");
/// let load = |name: &str, sc: LisaSpacecraftId| {
///     let raw = std::fs::read_to_string(format!("{root}/{name}")).unwrap();
///     LisaOrbitReader::from_str(&raw, sc).unwrap()
/// };
/// let provider = LisaEphemerisProvider::new(LisaOrbitSet {
///     sc1: load("lisa_orbit_sample.oem1", LisaSpacecraftId::SC1),
///     sc2: load("lisa_orbit_sample.oem2", LisaSpacecraftId::SC2),
///     sc3: load("lisa_orbit_sample.oem3", LisaSpacecraftId::SC3),
/// });
/// // Query SC1 at its first epoch (should return exact tabulated value)
/// let pt = provider.state(-1001, 1_139_702_400.0).unwrap();
/// assert!((pt.position.x().value() - 100000000.0).abs() < 1e-3);
/// ```
pub struct LisaEphemerisProvider {
    orbits: LisaOrbitSet,
}

impl LisaEphemerisProvider {
    /// Construct a provider from a loaded [`LisaOrbitSet`].
    ///
    /// # Examples
    ///
    /// ```
    /// use siderust_pod::io::lisa::{LisaOrbitReader, LisaOrbitSet, LisaEphemerisProvider, LisaSpacecraftId};
    ///
    /// let root = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data/lisa");
    /// let load = |name: &str, sc: LisaSpacecraftId| {
    ///     let raw = std::fs::read_to_string(format!("{root}/{name}")).unwrap();
    ///     LisaOrbitReader::from_str(&raw, sc).unwrap()
    /// };
    /// let provider = LisaEphemerisProvider::new(LisaOrbitSet {
    ///     sc1: load("lisa_orbit_sample.oem1", LisaSpacecraftId::SC1),
    ///     sc2: load("lisa_orbit_sample.oem2", LisaSpacecraftId::SC2),
    ///     sc3: load("lisa_orbit_sample.oem3", LisaSpacecraftId::SC3),
    /// });
    /// drop(provider);
    /// ```
    pub fn new(orbits: LisaOrbitSet) -> Self {
        Self { orbits }
    }
}

/// Error type for [`LisaEphemerisProvider`].
///
/// # Examples
///
/// ```
/// use siderust_pod::io::lisa::LisaProviderError;
/// let e = LisaProviderError::UnknownBody(-9999);
/// assert!(format!("{e}").contains("-9999"));
/// ```
#[derive(Debug, thiserror::Error)]
pub enum LisaProviderError {
    /// The requested NAIF body ID does not correspond to any LISA spacecraft.
    #[error("unknown LISA body id {0} (expected -1001, -1002, or -1003)")]
    UnknownBody(i32),
    /// The requested epoch lies outside the covered interval.
    #[error("epoch {0:.3} s (TDB J2000) is outside the covered interval [{1:.3}, {2:.3}]")]
    OutOfRange(f64, f64, f64),
}

impl EphemerisProvider for LisaEphemerisProvider {
    /// Interpolated [`LisaOrbitPoint`] state.
    type State = LisaOrbitPoint;
    /// Query error type.
    type Error = LisaProviderError;

    /// Return the interpolated state for a LISA spacecraft at the given epoch.
    ///
    /// `body_naif_id` must be −1001, −1002, or −1003.
    /// `epoch_seconds_tdb` is TDB seconds since J2000.0.
    ///
    /// Uses cubic Hermite interpolation between the two bracketing tabulated
    /// states.  If the epoch exactly coincides with a tabulated epoch, the
    /// tabulated value is returned without interpolation.
    fn state(
        &self,
        body_naif_id: i32,
        epoch_seconds_tdb: f64,
    ) -> Result<LisaOrbitPoint, LisaProviderError> {
        let sc = LisaSpacecraftId::from_naif_id(body_naif_id)
            .ok_or(LisaProviderError::UnknownBody(body_naif_id))?;
        let orbit = self.orbits.orbit(sc);
        hermite_interp(orbit, epoch_seconds_tdb)
    }
}

// ── Hermite interpolation ─────────────────────────────────────────────────────

fn hermite_interp(orbit: &LisaOrbit, t: f64) -> Result<LisaOrbitPoint, LisaProviderError> {
    let pts = &orbit.points;
    if pts.is_empty() {
        return Err(LisaProviderError::OutOfRange(t, f64::NAN, f64::NAN));
    }
    let t0 = pts.first().unwrap().epoch_j2000_s;
    let t1 = pts.last().unwrap().epoch_j2000_s;
    if t < t0 || t > t1 {
        return Err(LisaProviderError::OutOfRange(t, t0, t1));
    }

    // Binary search for the left bracket.
    let idx = match pts.binary_search_by(|p| {
        p.epoch_j2000_s
            .partial_cmp(&t)
            .unwrap_or(std::cmp::Ordering::Less)
    }) {
        Ok(i) => return Ok(pts[i].clone()), // exact hit
        Err(i) => i.saturating_sub(1).min(pts.len() - 2),
    };

    let p0 = &pts[idx];
    let p1 = &pts[idx + 1];

    let dt = p1.epoch_j2000_s - p0.epoch_j2000_s;
    let tau = (t - p0.epoch_j2000_s) / dt;

    // Cubic Hermite basis functions.
    let tau2 = tau * tau;
    let tau3 = tau2 * tau;
    let h00 = 2.0 * tau3 - 3.0 * tau2 + 1.0;
    let h10 = tau3 - 2.0 * tau2 + tau;
    let h01 = -2.0 * tau3 + 3.0 * tau2;
    let h11 = tau3 - tau2;

    // Interpolate position components.
    let interp_pos = |a: f64, da: f64, b: f64, db: f64| -> f64 {
        h00 * a + h10 * dt * da + h01 * b + h11 * dt * db
    };

    let x = interp_pos(
        p0.position.x().value(),
        p0.velocity.x().value(),
        p1.position.x().value(),
        p1.velocity.x().value(),
    );
    let y = interp_pos(
        p0.position.y().value(),
        p0.velocity.y().value(),
        p1.position.y().value(),
        p1.velocity.y().value(),
    );
    let z = interp_pos(
        p0.position.z().value(),
        p0.velocity.z().value(),
        p1.position.z().value(),
        p1.velocity.z().value(),
    );

    // Interpolate velocity components (derivative of the cubic / dt).
    let interp_vel = |a: f64, da: f64, b: f64, db: f64| -> f64 {
        let dh00 = 6.0 * tau2 - 6.0 * tau;
        let dh10 = 3.0 * tau2 - 4.0 * tau + 1.0;
        let dh01 = -6.0 * tau2 + 6.0 * tau;
        let dh11 = 3.0 * tau2 - 2.0 * tau;
        (dh00 * a + dh10 * dt * da + dh01 * b + dh11 * dt * db) / dt
    };

    let vx = interp_vel(
        p0.position.x().value(),
        p0.velocity.x().value(),
        p1.position.x().value(),
        p1.velocity.x().value(),
    );
    let vy = interp_vel(
        p0.position.y().value(),
        p0.velocity.y().value(),
        p1.position.y().value(),
        p1.velocity.y().value(),
    );
    let vz = interp_vel(
        p0.position.z().value(),
        p0.velocity.z().value(),
        p1.position.z().value(),
        p1.velocity.z().value(),
    );

    let position = LisaPosition::new(
        siderust::qtty::Kilometers::new(x),
        siderust::qtty::Kilometers::new(y),
        siderust::qtty::Kilometers::new(z),
    );
    let velocity = LisaVelocity::new(
        siderust::qtty::Quantity::<KmPerSecond>::new(vx),
        siderust::qtty::Quantity::<KmPerSecond>::new(vy),
        siderust::qtty::Quantity::<KmPerSecond>::new(vz),
    );

    // Epoch: linearly interpolate between the two bracketing epochs for the
    // return value.  The exact polynomial position/velocity is already captured.
    let epoch_j2000_s = p0.epoch_j2000_s + tau * dt;
    let epoch_jd = j2000_seconds_to_jd(epoch_j2000_s);
    let epoch = jd_to_time_tdb(epoch_jd).unwrap_or(p0.epoch);

    Ok(LisaOrbitPoint {
        epoch,
        epoch_j2000_s,
        position,
        velocity,
    })
}

// ── Time helpers ──────────────────────────────────────────────────────────────

/// J2000.0 epoch as a Julian Date (TT, but negligibly different from TDB here).
const J2000_JD: f64 = 2_451_545.0;

/// Convert a Julian Date (TDB) to TDB seconds since J2000.0.
fn jd_to_j2000_seconds(jd: f64) -> f64 {
    (jd - J2000_JD) * 86_400.0
}

/// Convert TDB seconds since J2000.0 back to Julian Date (TDB).
fn j2000_seconds_to_jd(s: f64) -> f64 {
    s / 86_400.0 + J2000_JD
}

/// Construct a [`Time<TDB>`] from a Julian Date value (TDB scale).
///
/// # Errors
///
/// Returns [`PodIoError::Format`] if `jd` is not finite or the conversion fails.
fn jd_to_time_tdb(jd: f64) -> Result<Time<TDB>, PodIoError> {
    JulianDate::<TDB>::try_new(siderust::qtty::Days::new(jd))
        .map(|enc| enc.to_j2000s())
        .map_err(|e| PodIoError::Format(format!("lisa: cannot convert JD {jd} to TDB Time: {e}")))
}
