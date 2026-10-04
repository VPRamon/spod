// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! `spod` — service application for Siderust POD workflows.
//!
//! This crate owns service configuration, orchestration, transport, and
//! artifact handling. Reusable POD science is provided directly by
//! [`siderust::pod`]; `spod` deliberately does not expose a compatibility
//! facade for those APIs.
//!
//! ## Modules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`tle`] | TLE / 3LE / OMM parser and writer |
//! | [`lambert`] | Typed Lambert two-point boundary-value solver |
//! | [`sgp4`] | SGP4/SDP4 propagator producing typed TEME states |
//! | [`spice`] | DAF/SPK kernel reader and SPICE ephemeris provider |
//! | [`io`] | File format parsers/writers: SP3, RINEX, ANTEX, OEM, EOP, … |
//! | [`service`] | Pipeline orchestration and job runner |
//!
//! ## Binary entry-points
//!
//! - `spod` — command-line interface
//! - `spod-rest` — REST API server

#![forbid(unsafe_code)]

pub mod io;
pub mod lambert;
pub mod service;
pub mod sgp4;
pub mod spice;
pub mod tle;
