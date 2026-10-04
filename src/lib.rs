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
//! | [`service`] | Pipeline orchestration, transport, and job runner |
//! | [`sgp4`] | Temporary full Vallado SGP4/SDP4 compatibility layer; removal tracked by spod #29 |
//!
//! ## Binary entry-points
//!
//! - `spod` — command-line interface
//! - `spod-rest` — REST API server

#![forbid(unsafe_code)]

pub mod service;
/// Full Vallado-style SGP4/SDP4 retained until upstream semantic parity.
pub mod sgp4;
