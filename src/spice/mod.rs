// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! `siderust-spice` — DAF/SPK kernel reader and `SpiceEphemerisProvider`
//! adapter on top of [`siderust::formats::spice::{daf, spk}`].
//!
//! # Scope
//!
//! This crate fills the gap between the upstream `siderust::formats::spice::{daf, spk}`
//! Type-2 reader and the production-grade SPK feature set required by POD.
//! It re-exports the upstream DAF parser and Type-2 segment reader, and
//! adds:
//!
//! * A unified [`SpkSegment`] enum that supports SPK **Type 2**
//!   (Chebyshev position) and **Type 3** (Chebyshev position+velocity).
//! * Per-record Chebyshev evaluation of position and velocity using the
//!   workspace `cheby` crate (Clenshaw recurrence + analytic derivative).
//! * An indexed [`SpkKernel`] that owns the file bytes, parses every
//!   summary, and chains body-relative segments to compute the state of
//!   any target relative to any kernel-supported center.
//! * [`SpiceEphemerisProvider`], a typed adapter implementing
//!   [`crate::core::providers::EphemerisProvider`]. Center selection
//!   is **explicit** at construction time and is propagated as a typed
//!   `affn` reference center on the returned [`SpiceState`].
//!
//! SPK **Type 9** and **Type 13** (Lagrange interpolation, equal- and
//! unequal-step) are out of scope for this drop. Loading a kernel that
//! contains a Type-9/13 segment succeeds, but a state query that resolves
//! to such a segment returns
//! [`SpiceError::UnsupportedDataType`].
//!
//! # Time semantics
//!
//! All epochs are **TDB seconds past J2000**, matching the convention of
//! NAIF SPK files and [`siderust::tempoch::J2000s`]. The provider trait method
//! exposes the same numeric convention; the typed `siderust::tempoch::EncodedTime`
//! constructor is available via [`SpiceEphemerisProvider::state_at`].
//!
//! # Validation gate
//!
//! Position recovery is byte-identical to NAIF's CSPICE
//! `spkez_c` for the planet body chains in `de440.bsp` / `de441.bsp`.
//! The `de440` feature unlocks an integration test
//! (`tests/de440_validation.rs`) that loads a kernel from disk
//! (`SIDERUST_SPICE_DE_PATH`) and compares against committed reference
//! states under `tests/data/de440_reference.json`.
//!
//! # Examples
//!
//! ```rust
//! use spod::spice::{SpkKernel, SpiceError};
//!
//! // The crate ships no DAF binaries; in production you load a path:
//! //   let kernel = SpkKernel::open("de440.bsp")?;
//! //   let state = kernel.state(/*target=*/3, /*center=*/0, /*et=*/0.0)?;
//! //   assert!(state.position_km[0].is_finite());
//!
//! // For the doc-test we just construct an explicit error to assert
//! // the error type wires into `std::error::Error` correctly.
//! let err: SpiceError = SpiceError::UnsupportedDataType { data_type: 9 };
//! assert!(format!("{err}").contains("Type 9"));
//! # Ok::<_, SpiceError>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod kernel;
mod naif;
mod provider;
mod segment;

pub use error::SpiceError;
pub use kernel::{LoadedSegment, SpkKernel};
pub use naif::{naif_id_for_name, well_known};
pub use provider::{SpiceEphemerisProvider, SpiceState};
pub use segment::{segment_for_summary, ChebSegment, SpkSegment};

/// Re-export of the upstream DAF container parser.
///
/// `siderust::formats::spice::daf` provides a strict, panic-free DAF file parser
/// that this crate uses as the foundation for SPK segment indexing. It
/// is re-exported here so downstream consumers can manipulate raw DAF
/// summaries without depending on `siderust` directly.
pub mod daf {
    pub use siderust::formats::spice::daf::{Daf, Summary};
}

/// Re-export of the upstream SPK Type 2 reader.
///
/// The upstream `siderust::formats::spice::spk` module already exposes a
/// production-quality Type 2 / Type 3 raw reader (`read_type2_segment`,
/// `parse_bsp`) plus the standard NAIF body-id constants. They are
/// re-exported here for callers that prefer the legacy entry points.
pub mod spk {
    pub use siderust::formats::spice::spk::{
        parse_bsp, read_type2_segment, BspSegments, SegmentData, EMB_CENTER, EMB_TARGET,
        MOON_CENTER, MOON_TARGET, SUN_CENTER, SUN_TARGET,
    };
}
