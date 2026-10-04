// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Unified error type for SPICE kernel operations.

use thiserror::Error;

/// Errors produced by SPK kernel operations.
///
/// All public APIs in this crate return `Result<_, SpiceError>`. The
/// variants are kept small and structured so adapters can branch on the
/// concrete failure mode (out-of-coverage vs. format error vs. missing
/// chain) without string matching.
///
/// # Examples
///
/// ```rust
/// use spod::spice::SpiceError;
///
/// let e = SpiceError::OutOfCoverage {
///     target: 399,
///     center: 0,
///     epoch_tdb_seconds: 0.0,
///     start_tdb_seconds: 1.0,
///     end_tdb_seconds: 2.0,
/// };
/// assert!(format!("{e}").contains("out of coverage"));
/// ```
#[derive(Debug, Error)]
pub enum SpiceError {
    /// I/O error reading a kernel file from disk.
    #[error("SPICE kernel I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The underlying DAF/SPK parser rejected the file.
    #[error("SPICE kernel parse error: {message}")]
    Parse {
        /// Human-readable description from the upstream parser.
        message: String,
    },

    /// No segment in the kernel covers `target → center` at `epoch`.
    #[error(
        "SPICE kernel: state for target={target} center={center} is out of coverage \
         at et={epoch_tdb_seconds} TDB-s; available segment range is \
         [{start_tdb_seconds}, {end_tdb_seconds}] TDB-s"
    )]
    OutOfCoverage {
        /// NAIF body ID of the requested target.
        target: i32,
        /// NAIF body ID of the requested center.
        center: i32,
        /// Epoch in TDB seconds past J2000.
        epoch_tdb_seconds: f64,
        /// Start of the closest segment's coverage window (TDB seconds).
        start_tdb_seconds: f64,
        /// End of the closest segment's coverage window (TDB seconds).
        end_tdb_seconds: f64,
    },

    /// Kernel contains no segment chain from `target` to `center`.
    #[error(
        "SPICE kernel: no chain from target={target} to center={center} \
         (kernel has no segment with this body pair, even via transitive \
         centers)"
    )]
    NoChain {
        /// NAIF body ID of the requested target.
        target: i32,
        /// NAIF body ID of the requested center.
        center: i32,
    },

    /// SPK data type is not implemented in this drop.
    ///
    /// This crate ships SPK Type 2 (Chebyshev position) and Type 3
    /// (Chebyshev position+velocity). Types 9 / 13 (Lagrange equal- and
    /// unequal-step interpolation) and the higher Types are not
    /// implemented; the parser still indexes them so callers can detect
    /// their presence via [`crate::spice::SpkKernel::segments`], but a state
    /// query that resolves to such a segment fails with this error.
    #[error("SPICE kernel: SPK Type {data_type} is not implemented")]
    UnsupportedDataType {
        /// NAIF SPK data type code (e.g. 9 or 13).
        data_type: i32,
    },

    /// Internal invariant violated while decoding a segment.
    ///
    /// Returned when a record's metadata (e.g. `radius`) is non-finite or
    /// non-positive, which would indicate kernel corruption.
    #[error("SPICE kernel: segment record corrupted: {message}")]
    Corrupted {
        /// Human-readable description.
        message: String,
    },
}

impl From<siderust::formats::spice::SpiceError> for SpiceError {
    fn from(err: siderust::formats::spice::SpiceError) -> Self {
        match err {
            siderust::formats::spice::SpiceError::Io(error) => SpiceError::Io(error),
            siderust::formats::spice::SpiceError::Parse { message } => {
                SpiceError::Parse { message }
            }
            siderust::formats::spice::SpiceError::OutOfCoverage {
                target,
                center,
                epoch_tdb_seconds,
                start_tdb_seconds,
                end_tdb_seconds,
            } => SpiceError::OutOfCoverage {
                target,
                center,
                epoch_tdb_seconds,
                start_tdb_seconds,
                end_tdb_seconds,
            },
            siderust::formats::spice::SpiceError::NoChain { target, center } => {
                SpiceError::NoChain { target, center }
            }
            siderust::formats::spice::SpiceError::UnsupportedDataType { data_type } => {
                SpiceError::UnsupportedDataType { data_type }
            }
            siderust::formats::spice::SpiceError::Corrupted { message } => {
                SpiceError::Corrupted { message }
            }
            other => SpiceError::Parse {
                message: other.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_coverage_message_lists_window() {
        let e = SpiceError::OutOfCoverage {
            target: 399,
            center: 0,
            epoch_tdb_seconds: 5.0,
            start_tdb_seconds: 10.0,
            end_tdb_seconds: 20.0,
        };
        let msg = format!("{e}");
        assert!(msg.contains("out of coverage"));
        assert!(msg.contains("399"));
        assert!(msg.contains("10"));
    }

    #[test]
    fn unsupported_data_type_message_contains_code() {
        let e = SpiceError::UnsupportedDataType { data_type: 13 };
        assert!(format!("{e}").contains("Type 13"));
    }
}
