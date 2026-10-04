// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! # Earth-fixed to inertial frame-transform provider
//!
//! ## Scientific scope
//!
//! Accurate range and carrier modelling requires moving consistently
//! between terrestrial station coordinates and inertial spacecraft states.
//! This module defines the provider seam for the full ITRF-GCRF chain
//! driven by Earth-orientation inputs and standard precession, nutation,
//! Earth rotation, and polar-motion conventions.
//!
//! The science here is limited to the interface contract: callers supply TT
//! and UT1 epochs because different pieces of the transformation depend on
//! different time scales. The module does not hard-code a specific
//! convention realization.
//!
//! ## Technical scope
//!
//! The public items are `FrameTransformProvider` and `FrameTransformError`.
//! Implementations return `Rotation3` transforms for ITRF-to-GCRF and the
//! inverse direction, leaving the choice of EOP source and astronomical
//! model to the concrete provider.
//!
//! Observation models consume this trait but do not own the realization of
//! the Earth-orientation chain.
//!
//! ## References
//!
//! - IERS Conventions Centre. (2010). IERS Conventions (2010). Verlag des
//!   Bundesamts fur Kartographie und Geodasie.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
use siderust::affn::Rotation3;
use siderust::time::JulianDate;
use thiserror::Error;

/// Error returned by [`FrameTransformProvider`].
#[derive(Debug, Error)]
pub enum FrameTransformError {
    /// The provider could not compute the rotation at the requested epoch.
    #[error("frame transform failed: {0}")]
    Failed(String),
}

/// Provider for frame rotations between Earth-fixed and inertial frames.
///
/// The `at(jd_tt, jd_ut1)` API takes both TT and UT1 because the precession-
/// nutation rotation is parameterised by TT while ERA is parameterised by UT1.
pub trait FrameTransformProvider: Send + Sync {
    /// Rotation from ITRF to GCRF at the given TT/UT1 epoch.
    fn itrf_to_gcrf(
        &self,
        jd_tt: JulianDate,
        jd_ut1: JulianDate,
    ) -> Result<Rotation3, FrameTransformError>;

    /// Rotation from GCRF to ITRF at the given TT/UT1 epoch.
    fn gcrf_to_itrf(
        &self,
        jd_tt: JulianDate,
        jd_ut1: JulianDate,
    ) -> Result<Rotation3, FrameTransformError> {
        Ok(self.itrf_to_gcrf(jd_tt, jd_ut1)?.inverse())
    }
}
