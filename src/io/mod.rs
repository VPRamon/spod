//! # spod I/O
//!
//! ## Scientific scope
//!
//! This crate handles the interchange formats that feed and leave POD
//! workflows: precise or broadcast GNSS products, SLR products, Earth-
//! orientation series, and orbit product files. The scientific content is
//! defined by the external standards being parsed or written rather than by
//! new physics inside the crate.
//!
//! Current coverage is intentionally selective. Each parser or writer
//! supports the subset needed by the MVP POD pipeline and ignores
//! unsupported record families when that is safer than pretending to
//! implement the full standard.
//!
//! ## Technical scope
//!
//! The crate exports format-specific reader and writer modules plus the
//! shared `PodIoError`. Most readers map external text or binary records
//! into typed Rust structs and, where appropriate, into `siderust`
//! orbit/time types such as `OrbitState` and `JulianDate`.
//!
//! Higher-level estimation, validation, and product orchestration are
//! outside the scope of this crate and are handled by sibling service,
//! observations, and products modules.
//!
//! ## References
//!
//! - IERS Conventions Centre. (2010). IERS Conventions (2010). Verlag des
//!   Bundesamts fur Kartographie und Geodasie.
//! - Vallado, D. A. (2013). Fundamentals of Astrodynamics and Applications
//!   (4th ed.). Microcosm Press.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::path::PathBuf;
use thiserror::Error;

pub mod antex;
pub mod cpf;
pub mod crd;
pub mod eop;
pub mod lisa;
pub mod oem;
pub mod omm;
pub mod opm;
pub mod orbex;
pub mod rinex_doris;
pub mod rinex_nav;
pub mod rinex_obs;
pub mod sinex;
pub mod sp3;
pub mod tdm;
pub mod vgosdb;

/// Location inside an input artefact, used by structured diagnostics.
///
/// Fields are 1-based when present (matching the convention of common text
/// editors and the format specifications themselves). `path` is `None` when
/// the input was a buffer rather than a named file.
///
/// # Examples
///
/// ```
/// use spod::io::FileLocation;
/// let loc = FileLocation::new(Some("/data/igs.sp3".into()), Some(42), Some(7));
/// assert_eq!(loc.line, Some(42));
/// assert_eq!(loc.column, Some(7));
/// assert!(loc.path.as_ref().unwrap().ends_with("igs.sp3"));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileLocation {
    /// Filesystem path, when known.
    pub path: Option<PathBuf>,
    /// 1-based line number, when known.
    pub line: Option<usize>,
    /// 1-based column number, when known.
    pub column: Option<usize>,
}

impl FileLocation {
    /// Build a [`FileLocation`] from optional fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use spod::io::FileLocation;
    /// let loc = FileLocation::new(None, Some(1), None);
    /// assert!(loc.path.is_none());
    /// ```
    pub fn new(path: Option<PathBuf>, line: Option<usize>, column: Option<usize>) -> Self {
        Self { path, line, column }
    }

    /// Build a buffer-anchored location with just a line number.
    ///
    /// # Examples
    ///
    /// ```
    /// use spod::io::FileLocation;
    /// let loc = FileLocation::at_line(17);
    /// assert_eq!(loc.line, Some(17));
    /// ```
    pub fn at_line(line: usize) -> Self {
        Self {
            path: None,
            line: Some(line),
            column: None,
        }
    }
}

impl std::fmt::Display for FileLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.path, self.line, self.column) {
            (Some(p), Some(l), Some(c)) => write!(f, "{}:{}:{}", p.display(), l, c),
            (Some(p), Some(l), None) => write!(f, "{}:{}", p.display(), l),
            (Some(p), None, _) => write!(f, "{}", p.display()),
            (None, Some(l), Some(c)) => write!(f, "<input>:{}:{}", l, c),
            (None, Some(l), None) => write!(f, "<input>:{}", l),
            (None, None, _) => write!(f, "<input>"),
        }
    }
}

/// Parse mode for permissive vs. strict ingestion.
///
/// In `Strict` mode, any deviation from the published format spec is a
/// hard error. In `Permissive` mode, recoverable deviations (extra
/// whitespace, unknown header records, truncated trailing blocks) are
/// silently skipped while still yielding a deterministic result.
///
/// Both modes are deterministic — `Permissive` is **not** a synonym for
/// "best-effort guess".
///
/// # Examples
///
/// ```
/// use spod::io::ParseMode;
/// assert_eq!(ParseMode::default(), ParseMode::Strict);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParseMode {
    /// Reject any spec violation.
    #[default]
    Strict,
    /// Recover from non-fatal deviations.
    Permissive,
}

/// Unified IO-layer error type for every format module.
///
/// `Format` retains its free-form string variant for compatibility, while
/// `Located` is the structured variant carrying a [`FileLocation`] and a
/// per-format spec section reference (e.g. `"SP3-d §3.2"`).
///
/// # Examples
///
/// ```
/// use spod::io::{FileLocation, PodIoError};
/// let err = PodIoError::located(
///     "RINEX 3.05 §6.3",
///     FileLocation::at_line(42),
///     "epoch line malformed",
/// );
/// assert!(format!("{err}").contains("RINEX"));
/// ```
#[derive(Debug, Error)]
pub enum PodIoError {
    /// Underlying IO failure.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// Malformed or unsupported file content (free-form).
    #[error("format: {0}")]
    Format(String),
    /// Format or feature not yet supported.
    #[error("unsupported: {0}")]
    Unsupported(String),
    /// Structured format error with file location and spec section.
    #[error("{spec} at {location}: {message}")]
    Located {
        /// Reference to the format-spec section being violated.
        spec: &'static str,
        /// Where the violation was observed.
        location: FileLocation,
        /// Human-readable diagnostic.
        message: String,
    },
}

impl PodIoError {
    /// Construct a structured [`PodIoError::Located`].
    ///
    /// # Examples
    ///
    /// ```
    /// use spod::io::{FileLocation, PodIoError};
    /// let _ = PodIoError::located("OEM v3 §5.2", FileLocation::at_line(3), "missing OBJECT_ID");
    /// ```
    pub fn located<M: Into<String>>(
        spec: &'static str,
        location: FileLocation,
        message: M,
    ) -> Self {
        Self::Located {
            spec,
            location,
            message: message.into(),
        }
    }
}
