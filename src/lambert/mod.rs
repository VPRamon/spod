//! `spod::lambert` — typed 0/N-revolution Lambert solver.
//!
//! ## Overview
//!
//! This module solves Lambert's two-point boundary-value problem (find
//! the conic transfer connecting two position vectors in a prescribed
//! time of flight) using Izzo's 2014 reformulation with third-order
//! Householder iteration. Both the single-revolution branch (`N = 0`)
//! and the multi-revolution branches (`N ≥ 1`, two solutions per `N`)
//! are implemented.
//!
//! ## Public API
//!
//! - [`lambert`] / [`lambert_n_rev`] — **typed** entry-points operating
//!   on [`siderust::affn::cartesian::Position`] / [`siderust::affn::cartesian::Velocity`]
//!   and [`siderust::qtty::Second`]. These are the documented public interface.
//! - [`solve_lambert`] / [`solve_lambert_n_rev`] — low-level numeric
//!   backend on plain `[f64; 3]` km / km/s arrays. Useful for FFI and
//!   for inner loops in mission-design search where allocating typed
//!   wrappers per evaluation is not desired.
//! - [`LambertError`] — single, unified error enum, re-exported here.
//!
//! ## Examples
//!
//! Earth-centred prograde transfer (Vallado *Fundamentals of Astrodynamics*
//! Ex. 7-5, 4th ed., p. 467):
//!
//! ```
//! use siderust::affn::cartesian::Position;
//! use siderust::affn::centers::ReferenceCenter;
//! use siderust::affn::frames::ICRS;
//! use siderust::qtty::GravitationalParameter;
//! use siderust::qtty::unit::Kilometer;
//! use siderust::qtty::Second;
//! use spod::lambert::{lambert, LambertBranch};
//!
//! let r1 = Position::<(), ICRS, Kilometer>::new(15945.34, 0.0, 0.0);
//! let r2 = Position::<(), ICRS, Kilometer>::new(12214.83899, 10249.46731, 0.0);
//! let tof = Second::new(4_560.0);
//! let mu = GravitationalParameter::new(398_600.4418);
//!
//! let sol = lambert(r1, r2, tof, mu, LambertBranch::Prograde).unwrap();
//! assert!((sol.v1.x().value() - 2.058913).abs() < 1e-3);
//! ```
//!
//! See `examples/03_lambert_earth_to_mars.rs` (workspace root) for a
//! heliocentric Earth → Mars transfer worked end-to-end.
//!
//! ## Out of scope (kept in higher-level crates)
//!
//! * Mission-design optimisation, porkchop search, and transfer-window
//!   enumeration belong in higher-level mission-design tooling.
//! * Frame / center bookkeeping at the algorithm boundary beyond what
//!   the typed entry-points provide.
//! * Any POD coupling.
//!
//! ## References
//!
//! - Izzo, D. (2014). *Revisiting Lambert's Problem*. Celest. Mech. Dyn.
//!   Astron., 121(1):1–15.
//! - Battin, R. H. (1999). *An Introduction to the Mathematics and
//!   Methods of Astrodynamics* (Rev. ed.). AIAA.
//! - Vallado, D. A. (2013). *Fundamentals of Astrodynamics and
//!   Applications* (4th ed.). Microcosm Press.

#![forbid(unsafe_code)]

mod error;
mod izzo;
mod typed;

pub use error::LambertError;
pub use izzo::{
    solve_lambert, solve_lambert_n_rev, LambertBranch, LambertDiagnostics, LambertSolution,
    NRevBranch,
};
pub use typed::{lambert, lambert_n_rev, TypedLambertSolution};
