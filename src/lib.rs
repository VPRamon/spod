// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! `siderust-pod` — Precise Orbit Determination toolkit.
//!
//! This crate provides a complete POD pipeline built on the
//! [`siderust`] and its public scientific re-exports.
//!
//! ## Modules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`tle`] | TLE / 3LE / OMM parser and writer |
//! | [`lambert`] | Typed Lambert two-point boundary-value solver |
//! | [`sgp4`] | SGP4/SDP4 propagator producing typed TEME states |
//! | [`spice`] | DAF/SPK kernel reader and SPICE ephemeris provider |
//! | [`core`] | POD domain primitives: errors, providers, parameters, covariance |
//! | [`io`] | File format parsers/writers: SP3, RINEX, ANTEX, OEM, EOP, … |
//! | [`dynamics`] | Astrodynamics primitives and POD force-model composition |
//! | [`observations`] | Measurement models: GNSS, SLR, LISA (feature-gated) |
//! | [`estimation`] | WLS, Gauss-Newton, EKF estimators |
//! | [`qc`] | Quality-control and validation products |
//! | [`products`] | SP3, OEM, residual CSV/Parquet, manifest writers |
//! | [`service`] | Pipeline orchestration and job runner |
//!
//! ## Binary entry-points
//!
//! - `siderust-pod` — command-line interface
//! - `siderust-pod-rest` — REST API server

#![forbid(unsafe_code)]

pub mod core;
pub mod dynamics;
pub mod estimation;
pub mod io;
pub mod lambert;
pub mod observations;
pub mod products;
pub mod qc;
pub mod service;
pub mod sgp4;
pub mod spice;
pub mod tle;
