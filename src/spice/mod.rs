// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! SPICE DAF/SPK support and `SpiceEphemerisProvider` service adapter.
//!
//! Low-level DAF parsing and raw SPK readers come directly from
//! [`siderust::formats::spice::{daf, spk}`]. This module adds the value-owned
//! kernel indexing, segment evaluation, body-chain resolution, and service
//! provider integration needed by `spod`.
//!
//! # Scope
//!
//! The module builds on the upstream `siderust::formats::spice::{daf, spk}`
//! readers and adds:
//!
//! * A unified [`SpkSegment`] enum that supports SPK **Type 2**
//!   (Chebyshev position) and **Type 3** (Chebyshev position+velocity).
//! * Per-record Chebyshev evaluation of position and velocity using the
//!   workspace `cheby` crate (Clenshaw recurrence + analytic derivative).
//! * An indexed [`SpkKernel`] that owns the file bytes, parses every
//!   summary, and chains body-relative segments to compute the state of
//!   any target relative to any kernel-supported center.
//! * [`SpiceEphemerisProvider`], a typed adapter implementing
//!   [`crate::service::providers::EphemerisProvider`]. Center selection
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
