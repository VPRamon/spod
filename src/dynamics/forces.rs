// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! POD-layer force-model trait, built-in implementations, and evaluating
//! [`ForceModelRegistry`].
//!
//! ## Scope
//!
//! This module adds a **POD-layer** [`ForceModel`] trait on top of the
//! upstream `siderust::astro::dynamics::forces::ForceModel` trait. The
//! key differences are:
//!
//! * The acceleration method receives an explicit [`Epoch`] argument
//!   (Julian Date) in addition to the Cartesian state, enabling time-varying
//!   models (SRP shadow, analytical Sun position) without a context object.
//! * Force models that need external providers (atmosphere for drag) embed
//!   them as `Arc<dyn …>` fields rather than reading them from a shared
//!   `DynamicsContext`.
//! * Partial derivatives return `Option<AccelPartials>` rather than a
//!   fallible `Result`, making composition cleaner.
//!
//! ## Built-in models
//!
//! | Type | Physics | Analytic partials |
//! |------|---------|-------------------|
//! | [`TwoBodyForce`] | Newtonian two-body (Earth) | ✓ |
//! | [`J2PerturbationForce`] | J2 zonal harmonic (Earth) | ✓ |
//! | [`DragForce`] | Cannonball drag (embedded atmosphere) | — |
//! | [`SolarRadiationPressureForce`] | Cannonball SRP + cylindrical shadow | — |
//!
//! ## Evaluating registry
//!
//! [`ForceModelRegistry`] holds an ordered list of [`Box<dyn ForceModel>`]
//! and evaluates the total acceleration and summed partial derivatives in a
//! single call.
//!
//! # Example
//!
//! ```
//! use std::sync::Arc;
//! use siderust::astro::dynamics::density::ExponentialAtmosphere;
//! use siderust::qtty::{AreaToMass, DragCoefficient, SrpCoefficient};
//! use spod::dynamics::forces::{
//!     DragForce, ForceModelRegistry, J2PerturbationForce, TwoBodyForce,
//! };
//!
//! let reg = ForceModelRegistry::new()
//!     .push(Box::new(TwoBodyForce::earth()))
//!     .push(Box::new(J2PerturbationForce::earth()))
//!     .push(Box::new(DragForce::new(
//!         DragCoefficient::new(2.2),
//!         AreaToMass::new(0.01),
//!         Arc::new(ExponentialAtmosphere::LEO_500KM),
//!     )));
//! assert_eq!(reg.len(), 3);
//! assert!(reg.is_variational());
//! ```

use std::sync::Arc;

use siderust::astro::dynamics::context::DynamicsContextBuilder;
use siderust::astro::dynamics::density::DensityProvider;
use siderust::astro::dynamics::forces::{
    AccelerationModel as UpstreamForceModel, DragForce as UpstreamDragForce, TwoBody, J2,
};
use siderust::astro::dynamics::state::{Acceleration, AccelerationUnit, OrbitState};
use siderust::astro::dynamics::{DynamicsContext, EARTH_J2, GM_EARTH, R_EARTH};
use siderust::coordinates::centers::Geocentric;
use siderust::coordinates::frames::GCRS;
use siderust::qtty::{AreaToMass, DragCoefficient, SrpCoefficient};
use siderust::time::JulianDate;

// ─────────────────────────────────────────────────────────────────────────────
// Type aliases
// ─────────────────────────────────────────────────────────────────────────────

/// Standard geocentric inertial Cartesian orbit state (position, velocity,
/// epoch). Alias for [`OrbitState<Geocentric, GCRS>`].
///
/// # Example
///
/// ```
/// use spod::dynamics::forces::CartesianState;
/// use siderust::astro::dynamics::{Position, Velocity};
/// use siderust::coordinates::frames::GCRS;
/// use siderust::time::JulianDate;
/// let _s: CartesianState = CartesianState::new(
///     JulianDate::new(2_451_545.0).to_j2000s(),
///     Position::<GCRS>::new(7_000.0, 0.0, 0.0),
///     Velocity::<GCRS>::new(0.0, 7.545, 0.0),
/// );
/// ```
pub type CartesianState = OrbitState<Geocentric, GCRS>;

/// Julian Date epoch consumed by POD force models.
///
/// # Example
///
/// ```
/// use spod::dynamics::forces::Epoch;
/// use siderust::time::JulianDate;
/// let _t: Epoch = JulianDate::new(2_451_545.0);
/// ```
pub type Epoch = JulianDate;

// ─────────────────────────────────────────────────────────────────────────────
// Acceleration3
// ─────────────────────────────────────────────────────────────────────────────

/// Three-component inertial acceleration in km/s², GCRS frame.
///
/// Stored as raw `[f64; 3]` (units km/s²) rather than a typed
/// [`siderust::affn`] vector
/// to make it easy to sum contributions from heterogeneous force models.
///
/// # Example
///
/// ```
/// use spod::dynamics::forces::Acceleration3;
/// let a = Acceleration3([1e-6, 0.0, -1e-7]);
/// let b = Acceleration3::zero();
/// let mut c = a;
/// c.add_assign(b);
/// assert!((c.0[0] - 1e-6).abs() < f64::EPSILON);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Acceleration3(pub [f64; 3]);

impl Acceleration3 {
    /// Zero acceleration.
    pub fn zero() -> Self {
        Self([0.0; 3])
    }

    /// Accumulate `other` into `self` element-wise.
    pub fn add_assign(&mut self, other: Acceleration3) {
        for i in 0..3 {
            self.0[i] += other.0[i];
        }
    }

    /// Euclidean magnitude (km/s²).
    pub fn magnitude(&self) -> f64 {
        (self.0[0].powi(2) + self.0[1].powi(2) + self.0[2].powi(2)).sqrt()
    }
}

impl From<Acceleration<GCRS, siderust::astro::dynamics::state::AccelerationUnit>>
    for Acceleration3
{
    fn from(a: Acceleration<GCRS, AccelerationUnit>) -> Self {
        Self([a.x().value(), a.y().value(), a.z().value()])
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AccelPartials
// ─────────────────────────────────────────────────────────────────────────────

/// Partial derivatives of the acceleration with respect to position
/// (`d_acc_d_pos`, units s⁻²) and velocity (`d_acc_d_vel`, units s⁻¹).
///
/// These 3×3 blocks form the lower half of the variational dynamics matrix
/// `F(t) = [[0, I], [A_r, A_v]]`.
///
/// # Example
///
/// ```
/// use spod::dynamics::forces::AccelPartials;
/// let z = AccelPartials::zero();
/// for row in &z.d_acc_d_pos {
///     for &v in row { assert_eq!(v, 0.0); }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccelPartials {
    /// `∂a/∂r` (3×3, units: s⁻²).
    pub d_acc_d_pos: [[f64; 3]; 3],
    /// `∂a/∂v` (3×3, units: s⁻¹). Zero for conservative (position-only) forces.
    pub d_acc_d_vel: [[f64; 3]; 3],
}

impl AccelPartials {
    /// Zero partial derivatives.
    pub fn zero() -> Self {
        Self {
            d_acc_d_pos: [[0.0; 3]; 3],
            d_acc_d_vel: [[0.0; 3]; 3],
        }
    }

    /// Accumulate `other` into `self` element-wise.
    pub fn add_assign(&mut self, other: &AccelPartials) {
        for i in 0..3 {
            for j in 0..3 {
                self.d_acc_d_pos[i][j] += other.d_acc_d_pos[i][j];
                self.d_acc_d_vel[i][j] += other.d_acc_d_vel[i][j];
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ForceModel trait
// ─────────────────────────────────────────────────────────────────────────────

/// POD-layer force model trait.
///
/// Unlike the upstream `siderust::astro::dynamics::forces::ForceModel`, this
/// trait:
///
/// * Takes an explicit [`Epoch`] (Julian Date) alongside the state.
/// * Returns acceleration directly as [`Acceleration3`], without `Result`.
///   Implementations must handle errors internally (e.g. by returning zero).
/// * Expresses partial derivatives as `Option<AccelPartials>` so the
///   evaluating registry can selectively sum only the models that have them.
///
/// # Example
///
/// ```
/// use spod::dynamics::forces::{
///     AccelPartials, Acceleration3, CartesianState, Epoch, ForceModel,
/// };
/// use siderust::astro::dynamics::{Position, Velocity};
/// use siderust::coordinates::frames::GCRS;
/// use siderust::time::JulianDate;
///
/// struct ConstantDrag;
/// impl ForceModel for ConstantDrag {
///     fn name(&self) -> &str { "constant_drag" }
///     fn acceleration(&self, _s: &CartesianState, _t: Epoch) -> Acceleration3 {
///         Acceleration3([-1e-8, 0.0, 0.0])
///     }
///     fn partials(&self, _s: &CartesianState, _t: Epoch) -> Option<AccelPartials> { None }
/// }
///
/// assert_eq!(ConstantDrag.name(), "constant_drag");
/// assert!(!ConstantDrag.is_variational());
/// ```
pub trait ForceModel: Send + Sync {
    /// Stable human-readable name for this model.
    fn name(&self) -> &str;

    /// Inertial acceleration (km/s², GCRS) at `state` and epoch `t`.
    ///
    /// Implementations that encounter unrecoverable errors (e.g. atmosphere
    /// not available) should return [`Acceleration3::zero()`].
    fn acceleration(&self, state: &CartesianState, t: Epoch) -> Acceleration3;

    /// Analytic partial derivatives of the acceleration w.r.t. position and
    /// velocity, or `None` if this model does not provide them.
    fn partials(&self, state: &CartesianState, t: Epoch) -> Option<AccelPartials>;

    /// Return `true` iff this model provides analytic partial derivatives.
    ///
    /// The default returns `false`. Override in models that implement
    /// [`ForceModel::partials`] to return `Some`.
    fn is_variational(&self) -> bool {
        false
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TwoBodyForce
// ─────────────────────────────────────────────────────────────────────────────

/// Newtonian two-body (point-mass) central force for Earth.
///
/// Delegates acceleration and analytic Jacobians to the upstream
/// [`siderust::astro::dynamics::forces::TwoBody`] model. Provides analytic
/// partial derivatives `∂a/∂r` (the Clohessy–Wiltshire tidal matrix).
///
/// # Example
///
/// ```
/// use spod::dynamics::forces::{CartesianState, Epoch, TwoBodyForce, ForceModel};
/// use siderust::astro::dynamics::{Position, Velocity};
/// use siderust::coordinates::frames::GCRS;
/// use siderust::time::JulianDate;
///
/// let f = TwoBodyForce::earth();
/// let s = CartesianState::new(
///     JulianDate::new(2_451_545.0).to_j2000s(),
///     Position::<GCRS>::new(7_000.0, 0.0, 0.0),
///     Velocity::<GCRS>::new(0.0, 7.545, 0.0),
/// );
/// let a = f.acceleration(&s, JulianDate::new(2_451_545.0));
/// assert!(a.magnitude() > 0.0);
/// assert!(f.is_variational());
/// ```
pub struct TwoBodyForce {
    inner: TwoBody,
}

impl TwoBodyForce {
    /// Construct with Earth's standard gravitational parameter (GM = 398 600.4418 km³/s²).
    pub fn earth() -> Self {
        Self {
            inner: TwoBody::new(GM_EARTH),
        }
    }
}

impl ForceModel for TwoBodyForce {
    fn name(&self) -> &str {
        "two_body"
    }

    fn acceleration(&self, state: &CartesianState, _t: Epoch) -> Acceleration3 {
        let ctx = DynamicsContext::empty();
        match UpstreamForceModel::acceleration(&self.inner, state, &ctx) {
            Ok(a) => Acceleration3::from(a),
            Err(_) => Acceleration3::zero(),
        }
    }

    fn partials(&self, state: &CartesianState, _t: Epoch) -> Option<AccelPartials> {
        let ctx = DynamicsContext::empty();
        let fp = UpstreamForceModel::partials(&self.inner, state, &ctx).ok()?;
        Some(AccelPartials {
            d_acc_d_pos: *fp.d_acc_d_pos.as_array(),
            d_acc_d_vel: *fp.d_acc_d_vel.as_array(),
        })
    }

    fn is_variational(&self) -> bool {
        true
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// J2PerturbationForce
// ─────────────────────────────────────────────────────────────────────────────

/// J2 zonal harmonic oblateness perturbation for Earth.
///
/// Provides analytic partial derivatives `∂a_J2/∂r` from the upstream
/// [`siderust::astro::dynamics::forces::J2`] implementation. The full
/// STM for a two-body + J2 propagation is obtained by composing
/// [`TwoBodyForce`] and [`J2PerturbationForce`] in a [`ForceModelRegistry`].
///
/// # Example
///
/// ```
/// use spod::dynamics::forces::{
///     CartesianState, ForceModel, J2PerturbationForce,
/// };
/// use siderust::astro::dynamics::{Position, Velocity};
/// use siderust::coordinates::frames::GCRS;
/// use siderust::time::JulianDate;
///
/// let f = J2PerturbationForce::earth();
/// let s = CartesianState::new(
///     JulianDate::new(2_451_545.0).to_j2000s(),
///     Position::<GCRS>::new(7_000.0, 0.0, 0.0),
///     Velocity::<GCRS>::new(0.0, 7.545, 0.0),
/// );
/// assert!(f.is_variational());
/// assert!(f.partials(&s, JulianDate::new(2_451_545.0)).is_some());
/// ```
pub struct J2PerturbationForce {
    inner: J2,
}

impl J2PerturbationForce {
    /// Construct with Earth's J2 = 1.0826257 × 10⁻³.
    pub fn earth() -> Self {
        Self {
            inner: J2::new(GM_EARTH, R_EARTH, EARTH_J2),
        }
    }
}

impl ForceModel for J2PerturbationForce {
    fn name(&self) -> &str {
        "j2"
    }

    fn acceleration(&self, state: &CartesianState, _t: Epoch) -> Acceleration3 {
        let ctx = DynamicsContext::empty();
        match UpstreamForceModel::acceleration(&self.inner, state, &ctx) {
            Ok(a) => Acceleration3::from(a),
            Err(_) => Acceleration3::zero(),
        }
    }

    fn partials(&self, state: &CartesianState, _t: Epoch) -> Option<AccelPartials> {
        let ctx = DynamicsContext::empty();
        let fp = UpstreamForceModel::partials(&self.inner, state, &ctx).ok()?;
        Some(AccelPartials {
            d_acc_d_pos: *fp.d_acc_d_pos.as_array(),
            d_acc_d_vel: *fp.d_acc_d_vel.as_array(),
        })
    }

    fn is_variational(&self) -> bool {
        true
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DragForce
// ─────────────────────────────────────────────────────────────────────────────

/// Cannonball atmospheric drag force.
///
/// Unlike the upstream model, which reads the atmosphere from a shared
/// [`DynamicsContext`], this POD-layer version **embeds** its atmosphere
/// density provider as an `Arc<dyn DensityProvider>`.
///
/// ```text
/// a_drag = −½ · Cd · (A/m) · ρ(h) · |v_rel|_SI · v_rel
/// ```
///
/// Partial derivatives are not provided (analytic drag Jacobians are
/// force-model–specific and model-dependent; use the finite-difference
/// harness in [`crate::dynamics::variational`] when needed).
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use siderust::astro::dynamics::density::ExponentialAtmosphere;
/// use siderust::qtty::{AreaToMass, DragCoefficient};
/// use spod::dynamics::forces::{CartesianState, DragForce, ForceModel};
/// use siderust::astro::dynamics::{Position, Velocity};
/// use siderust::coordinates::frames::GCRS;
/// use siderust::time::JulianDate;
///
/// let f = DragForce::new(
///     DragCoefficient::new(2.2),
///     AreaToMass::new(0.01),
///     Arc::new(ExponentialAtmosphere::LEO_500KM),
/// );
/// let s = CartesianState::new(
///     JulianDate::new(2_451_545.0).to_j2000s(),
///     Position::<GCRS>::new(6_871.0, 0.0, 0.0),
///     Velocity::<GCRS>::new(0.0, 7.612, 0.0),
/// );
/// let a = f.acceleration(&s, JulianDate::new(2_451_545.0));
/// // Drag opposes velocity → x-component should be ~0, y < 0
/// assert!(!f.is_variational());
/// ```
pub struct DragForce {
    cd: DragCoefficient,
    area_to_mass: AreaToMass,
    atmosphere: Arc<dyn DensityProvider + Send + Sync>,
}

impl DragForce {
    /// Construct with explicit drag coefficient, area-to-mass ratio, and
    /// atmosphere density provider.
    pub fn new(
        cd: DragCoefficient,
        area_to_mass: AreaToMass,
        atmosphere: Arc<dyn DensityProvider + Send + Sync>,
    ) -> Self {
        Self {
            cd,
            area_to_mass,
            atmosphere,
        }
    }

    /// Drag coefficient.
    pub fn cd(&self) -> DragCoefficient {
        self.cd
    }

    /// Area-to-mass ratio (m²/kg).
    pub fn area_to_mass(&self) -> AreaToMass {
        self.area_to_mass
    }
}

impl ForceModel for DragForce {
    fn name(&self) -> &str {
        "drag"
    }

    fn acceleration(&self, state: &CartesianState, _t: Epoch) -> Acceleration3 {
        let ctx = DynamicsContextBuilder::new()
            .with_atmosphere(Arc::clone(&self.atmosphere))
            .build();
        let upstream = UpstreamDragForce::new(self.cd, self.area_to_mass);
        match UpstreamForceModel::acceleration(&upstream, state, &ctx) {
            Ok(a) => Acceleration3::from(a),
            Err(_) => Acceleration3::zero(),
        }
    }

    fn partials(&self, _state: &CartesianState, _t: Epoch) -> Option<AccelPartials> {
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SolarRadiationPressureForce
// ─────────────────────────────────────────────────────────────────────────────

/// Solar radiation pressure constant at 1 AU (N/m²).
const P0_N_M2: f64 = 4.560e-6;

/// 1 AU in km.
const AU_KM: f64 = 149_597_870.7;

/// Earth equatorial radius for shadow geometry (km, WGS-84).
const R_EARTH_SHADOW_KM: f64 = 6_378.137;

/// Low-precision analytic Sun position in GCRS (km) from Julian Date.
///
/// Accuracy is approximately 0.01° in ecliptic longitude, adequate for
/// shadow determination in SRP modelling.
///
/// Formulae from Montenbruck & Gill, *Satellite Orbits*, Appendix A.
fn sun_position_gcrs_km(jd: JulianDate) -> [f64; 3] {
    let t: f64 = (jd.value() - 2_451_545.0) / 36_525.0;
    let l0_deg: f64 = 280.460 + 36_000.771 * t;
    let m_deg: f64 = 357.529_1 + 35_999.050_3 * t;
    let m: f64 = m_deg.to_radians();
    let lam_deg: f64 = l0_deg + 1.914_666 * m.sin() + 0.019_994 * (2.0 * m).sin();
    let lam: f64 = lam_deg.to_radians();
    let eps: f64 = (23.439_4_f64 - 0.013_0 * t).to_radians();
    [
        AU_KM * lam.cos(),
        AU_KM * eps.cos() * lam.sin(),
        AU_KM * eps.sin() * lam.sin(),
    ]
}

/// Cylindrical shadow factor ν for the cannonball SRP model.
///
/// Returns `0.0` if the satellite is inside Earth's cylindrical shadow
/// (anti-sun side AND within the projected Earth radius), `1.0` otherwise.
///
/// Reference: Montenbruck & Gill §3.4.
fn cylindrical_shadow_nu(r_sat: [f64; 3], r_sun: [f64; 3]) -> f64 {
    let sun_mag = (r_sun[0].powi(2) + r_sun[1].powi(2) + r_sun[2].powi(2)).sqrt();
    if sun_mag == 0.0 {
        return 1.0;
    }
    let s = [r_sun[0] / sun_mag, r_sun[1] / sun_mag, r_sun[2] / sun_mag];
    let dot = r_sat[0] * s[0] + r_sat[1] * s[1] + r_sat[2] * s[2];
    // Satellite on the sun side → fully illuminated.
    if dot > 0.0 {
        return 1.0;
    }
    // Perpendicular distance from the anti-sun axis.
    let px = r_sat[0] - dot * s[0];
    let py = r_sat[1] - dot * s[1];
    let pz = r_sat[2] - dot * s[2];
    let perp_sq = px.powi(2) + py.powi(2) + pz.powi(2);
    if perp_sq < R_EARTH_SHADOW_KM.powi(2) {
        0.0
    } else {
        1.0
    }
}

/// Cannonball solar radiation pressure with cylindrical Earth shadow model.
///
/// The Sun position is computed from the supplied epoch using a low-precision
/// analytic formula (Montenbruck & Gill, Appendix A), accurate to ~0.01°.
/// The shadow model follows Montenbruck & Gill §3.4 (cylindrical).
///
/// ```text
/// a_srp = ν · Cr · P₀ · (AU² / |r_sat − r_sun|²) · (A/m) / 1000   [km/s²]
/// ```
///
/// # Example
///
/// ```
/// use siderust::qtty::{AreaToMass, SrpCoefficient};
/// use spod::dynamics::forces::{
///     CartesianState, ForceModel, SolarRadiationPressureForce,
/// };
/// use siderust::astro::dynamics::{Position, Velocity};
/// use siderust::coordinates::frames::GCRS;
/// use siderust::time::JulianDate;
///
/// let f = SolarRadiationPressureForce::new(SrpCoefficient::new(1.5), AreaToMass::new(0.02));
/// let jd = JulianDate::new(2_451_545.0);
/// let s = CartesianState::new(
///     jd.to_j2000s(),
///     // Place satellite at x=+7000 km (same side as Sun in this epoch)
///     Position::<GCRS>::new(7_000.0, 0.0, 0.0),
///     Velocity::<GCRS>::new(0.0, 7.545, 0.0),
/// );
/// let a = f.acceleration(&s, jd);
/// // Magnitude is finite (not NaN)
/// assert!(a.0.iter().all(|v| v.is_finite()));
/// assert!(!f.is_variational());
/// ```
pub struct SolarRadiationPressureForce {
    cr: SrpCoefficient,
    area_to_mass: AreaToMass,
}

impl SolarRadiationPressureForce {
    /// Construct with reflectivity coefficient `Cr` and area-to-mass ratio.
    ///
    /// The cylindrical Earth shadow model is always active. To suppress
    /// shadow, use `Cr = 0.0`.
    pub fn new(cr: SrpCoefficient, area_to_mass: AreaToMass) -> Self {
        Self { cr, area_to_mass }
    }

    /// SRP reflectivity coefficient.
    pub fn cr(&self) -> SrpCoefficient {
        self.cr
    }

    /// Area-to-mass ratio (m²/kg).
    pub fn area_to_mass(&self) -> AreaToMass {
        self.area_to_mass
    }
}

impl ForceModel for SolarRadiationPressureForce {
    fn name(&self) -> &str {
        "srp_cylindrical"
    }

    fn acceleration(&self, state: &CartesianState, t: Epoch) -> Acceleration3 {
        let r_sat = [
            state.position.x().value(),
            state.position.y().value(),
            state.position.z().value(),
        ];
        let r_sun = sun_position_gcrs_km(t);
        let nu = cylindrical_shadow_nu(r_sat, r_sun);
        if nu == 0.0 {
            return Acceleration3::zero();
        }
        // Vector from Sun to satellite (direction of photon flux arriving at s/c).
        let dx = r_sat[0] - r_sun[0];
        let dy = r_sat[1] - r_sun[1];
        let dz = r_sat[2] - r_sun[2];
        let r = (dx.powi(2) + dy.powi(2) + dz.powi(2)).sqrt();
        if r == 0.0 {
            return Acceleration3::zero();
        }
        // a_srp = ν · Cr · P₀ · (AU² / r²) · (A/m) / 1000  [km/s²]
        let mag =
            nu * self.cr.value() * P0_N_M2 * (AU_KM * AU_KM / (r * r)) * self.area_to_mass.value()
                / 1_000.0;
        let inv_r = 1.0 / r;
        Acceleration3([mag * dx * inv_r, mag * dy * inv_r, mag * dz * inv_r])
    }

    fn partials(&self, _state: &CartesianState, _t: Epoch) -> Option<AccelPartials> {
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ForceModelRegistry (evaluating kind)
// ─────────────────────────────────────────────────────────────────────────────

/// Runtime evaluating registry of POD force models.
///
/// Holds an ordered list of [`Box<dyn ForceModel>`] objects and evaluates
/// the total acceleration and summed partial derivatives in a single call.
/// Unlike the factory registry in [`crate::dynamics::registry`] (which maps string
/// keys to factories), this registry holds live model instances ready to
/// be evaluated against a state.
///
/// # Example
///
/// ```
/// use spod::dynamics::forces::{
///     ForceModelRegistry, J2PerturbationForce, TwoBodyForce,
/// };
/// use siderust::astro::dynamics::{Position, Velocity};
/// use siderust::coordinates::frames::GCRS;
/// use siderust::time::JulianDate;
///
/// let reg = ForceModelRegistry::new()
///     .push(Box::new(TwoBodyForce::earth()))
///     .push(Box::new(J2PerturbationForce::earth()));
///
/// assert_eq!(reg.len(), 2);
/// assert!(reg.is_variational());
///
/// let s = spod::dynamics::forces::CartesianState::new(
///     JulianDate::new(2_451_545.0).to_j2000s(),
///     Position::<GCRS>::new(7_000.0, 0.0, 0.0),
///     Velocity::<GCRS>::new(0.0, 7.545, 0.0),
/// );
/// let jd = JulianDate::new(2_451_545.0);
/// let acc = reg.total_acceleration(&s, jd);
/// assert!(acc.magnitude() > 0.0);
/// ```
pub struct ForceModelRegistry {
    models: Vec<Box<dyn ForceModel>>,
}

impl Default for ForceModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ForceModelRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    /// Append a force model and return `self` for builder-style chaining.
    pub fn push(mut self, model: Box<dyn ForceModel>) -> Self {
        self.models.push(model);
        self
    }

    /// Number of registered models.
    pub fn len(&self) -> usize {
        self.models.len()
    }

    /// `true` iff the registry holds no models.
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    /// `true` iff at least one registered model provides analytic partial
    /// derivatives.
    pub fn is_variational(&self) -> bool {
        self.models.iter().any(|m| m.is_variational())
    }

    /// Evaluate the sum of all registered accelerations at `state` and epoch `t`.
    ///
    /// Models that encounter errors return [`Acceleration3::zero()`]; they do
    /// not propagate failures.
    ///
    /// # Example
    ///
    /// ```
    /// use spod::dynamics::forces::{ForceModelRegistry, TwoBodyForce};
    /// use siderust::astro::dynamics::{Position, Velocity};
    /// use siderust::coordinates::frames::GCRS;
    /// use siderust::time::JulianDate;
    ///
    /// let reg = ForceModelRegistry::new().push(Box::new(TwoBodyForce::earth()));
    /// let s = spod::dynamics::forces::CartesianState::new(
    ///     JulianDate::new(2_451_545.0).to_j2000s(),
    ///     Position::<GCRS>::new(7_000.0, 0.0, 0.0),
    ///     Velocity::<GCRS>::new(0.0, 7.545, 0.0),
    /// );
    /// let a = reg.total_acceleration(&s, JulianDate::new(2_451_545.0));
    /// assert!(a.magnitude() > 0.5e-3); // ≈ 8.15e-3 km/s² for LEO
    /// ```
    pub fn total_acceleration(&self, state: &CartesianState, t: Epoch) -> Acceleration3 {
        let mut acc = Acceleration3::zero();
        for m in &self.models {
            acc.add_assign(m.acceleration(state, t));
        }
        acc
    }

    /// Sum the partial derivatives of all variational models.
    ///
    /// Returns `None` if no model provides analytic partials.
    ///
    /// # Example
    ///
    /// ```
    /// use spod::dynamics::forces::{ForceModelRegistry, TwoBodyForce};
    /// use siderust::astro::dynamics::{Position, Velocity};
    /// use siderust::coordinates::frames::GCRS;
    /// use siderust::time::JulianDate;
    ///
    /// let reg = ForceModelRegistry::new().push(Box::new(TwoBodyForce::earth()));
    /// let s = spod::dynamics::forces::CartesianState::new(
    ///     JulianDate::new(2_451_545.0).to_j2000s(),
    ///     Position::<GCRS>::new(7_000.0, 0.0, 0.0),
    ///     Velocity::<GCRS>::new(0.0, 7.545, 0.0),
    /// );
    /// let p = reg.total_partials(&s, JulianDate::new(2_451_545.0));
    /// assert!(p.is_some());
    /// ```
    pub fn total_partials(&self, state: &CartesianState, t: Epoch) -> Option<AccelPartials> {
        if !self.is_variational() {
            return None;
        }
        let mut partials = AccelPartials::zero();
        for m in &self.models {
            if let Some(p) = m.partials(state, t) {
                partials.add_assign(&p);
            }
        }
        Some(partials)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use siderust::astro::dynamics::density::ExponentialAtmosphere;
    use siderust::astro::dynamics::{Position, Velocity};
    use siderust::coordinates::frames::GCRS;
    use siderust::time::JulianDate;

    use super::*;

    fn leo_state() -> CartesianState {
        CartesianState::new(
            JulianDate::new(2_451_545.0).to_j2000s(),
            Position::<GCRS>::new(6_871.0, 0.0, 0.0),
            Velocity::<GCRS>::new(0.0, 7.612, 0.0),
        )
    }

    const JD_J2000: f64 = 2_451_545.0;

    fn jd_j2000() -> Epoch {
        JulianDate::new(JD_J2000)
    }

    #[test]
    fn two_body_acceleration_has_expected_sign() {
        let f = TwoBodyForce::earth();
        let a = f.acceleration(&leo_state(), jd_j2000());
        // At [r, 0, 0], gravity points in −x direction.
        assert!(
            a.0[0] < 0.0,
            "x-component of two-body accel must be negative"
        );
        assert!(a.0[1].abs() < 1e-12, "y-component must be zero by symmetry");
        assert!(a.0[2].abs() < 1e-12, "z-component must be zero by symmetry");
    }

    #[test]
    fn two_body_partials_available() {
        let f = TwoBodyForce::earth();
        let p = f.partials(&leo_state(), jd_j2000());
        assert!(p.is_some(), "TwoBodyForce must provide analytic partials");
    }

    #[test]
    fn j2_adds_out_of_plane_component() {
        let s = CartesianState::new(
            jd_j2000().to_j2000s(),
            // Off-equatorial position
            Position::<GCRS>::new(6_371.0, 0.0, 500.0),
            Velocity::<GCRS>::new(0.0, 7.612, 0.0),
        );
        let f = J2PerturbationForce::earth();
        let a = f.acceleration(&s, jd_j2000());
        // J2 has a z-component for off-equatorial positions.
        assert!(
            a.0[2].abs() > 1e-15,
            "J2 z-accel must be non-zero off-equator"
        );
    }

    #[test]
    fn j2_partials_available() {
        let f = J2PerturbationForce::earth();
        let p = f.partials(&leo_state(), jd_j2000());
        assert!(
            p.is_some(),
            "J2PerturbationForce must provide analytic partials"
        );
    }

    #[test]
    fn drag_produces_nonzero_deceleration() {
        let f = DragForce::new(
            DragCoefficient::new(2.2),
            AreaToMass::new(0.01),
            Arc::new(ExponentialAtmosphere::LEO_500KM),
        );
        // State at ~500 km altitude
        let s = CartesianState::new(
            jd_j2000().to_j2000s(),
            Position::<GCRS>::new(6_871.0, 0.0, 0.0),
            Velocity::<GCRS>::new(0.0, 7.612, 0.0),
        );
        let a = f.acceleration(&s, jd_j2000());
        assert!(
            a.magnitude() > 0.0,
            "drag at 500 km must yield non-zero deceleration"
        );
        assert!(!f.is_variational(), "drag has no analytic partials");
    }

    #[test]
    fn srp_returns_finite_acceleration() {
        let f = SolarRadiationPressureForce::new(SrpCoefficient::new(1.5), AreaToMass::new(0.02));
        let a = f.acceleration(&leo_state(), jd_j2000());
        assert!(
            a.0.iter().all(|v| v.is_finite()),
            "SRP acceleration must be finite"
        );
    }

    #[test]
    fn registry_total_acceleration_sums_contributions() {
        let two_body = TwoBodyForce::earth();
        let a_single = two_body.acceleration(&leo_state(), jd_j2000());

        let reg = ForceModelRegistry::new().push(Box::new(TwoBodyForce::earth()));
        let a_reg = reg.total_acceleration(&leo_state(), jd_j2000());
        assert!(
            (a_single.0[0] - a_reg.0[0]).abs() < 1e-20,
            "Registry must sum contributions faithfully"
        );
    }

    #[test]
    fn registry_total_partials_none_when_no_variational_models() {
        let reg = ForceModelRegistry::new().push(Box::new(SolarRadiationPressureForce::new(
            SrpCoefficient::new(1.5),
            AreaToMass::new(0.02),
        )));
        assert!(
            reg.total_partials(&leo_state(), jd_j2000()).is_none(),
            "No variational models → total_partials must be None"
        );
    }

    #[test]
    fn registry_total_partials_some_with_variational_models() {
        let reg = ForceModelRegistry::new()
            .push(Box::new(TwoBodyForce::earth()))
            .push(Box::new(J2PerturbationForce::earth()));
        let p = reg.total_partials(&leo_state(), jd_j2000());
        assert!(p.is_some(), "TwoBody + J2 → total_partials must be Some");
    }

    #[test]
    fn cylindrical_shadow_in_shadow() {
        // Satellite behind Earth (anti-sun side, within shadow cylinder)
        let r_sat = [-7_000.0_f64, 0.0, 0.0];
        // Sun at +x direction
        let r_sun = [AU_KM, 0.0, 0.0];
        assert_eq!(
            cylindrical_shadow_nu(r_sat, r_sun),
            0.0,
            "satellite directly behind Earth must be in shadow"
        );
    }

    #[test]
    fn cylindrical_shadow_in_sunlight() {
        // Satellite on sun side
        let r_sat = [7_000.0_f64, 0.0, 0.0];
        let r_sun = [AU_KM, 0.0, 0.0];
        assert_eq!(
            cylindrical_shadow_nu(r_sat, r_sun),
            1.0,
            "satellite on sun side must be fully illuminated"
        );
    }
}
