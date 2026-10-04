//! # Residual CSV writer
//!
//! ## Scientific scope
//!
//! Residual time series are a standard diagnostic product in POD, allowing
//! analysts to inspect measurement fit quality by epoch, sensor, and
//! observable type. This module serializes those diagnostics into a simple
//! flat table for downstream plotting or audit workflows.
//!
//! It assumes residuals and sigmas have already been computed by the
//! service and QC layers. The writer does not reinterpret or normalize the
//! measurements.
//!
//! ## Technical scope
//!
//! Two CSV surfaces are offered:
//!
//! - **Legacy**: [`ResidualRow`] + [`write_residuals_csv`] — a simple
//!   in-memory batch writer for backwards compatibility.
//!
//! - **Streaming**: [`ResidualRecord`] + [`ResidualCsvWriter`] — a
//!   streaming writer using the standard POD column format
//!   `(epoch_jd_tt, obs_type, satellite, residual_m, sigma_m, rejected)`.
//!   The writer flushes to the sink after each record for crash-safety.
//!
//! Formatting is intentionally simple and stable; statistical aggregation
//! remains the responsibility of QC modules.
//!
//! ## References
//!
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). *Statistical Orbit
//!   Determination*. Elsevier Academic Press.
//! - Vallado, D. A. (2013). *Fundamentals of Astrodynamics and Applications*
//!   (4th ed.). Microcosm Press.

use super::error::PodProductsError;
use serde::{Deserialize, Serialize};
use std::io::Write;

pub use siderust::pod::product::residuals_csv::ResidualRecord;

/// One residual row (legacy API).
///
/// For new code prefer [`ResidualRecord`] and [`ResidualCsvWriter`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualRow {
    /// Epoch (Julian Date, TT scale).
    pub jd_tt: f64,
    /// Measurement type tag (e.g. `"code-G01"` or `"phase-G05"`).
    pub kind: String,
    /// Measured value (metres).
    pub measured_m: f64,
    /// Predicted value (metres).
    pub predicted_m: f64,
    /// Residual (measured − predicted), metres.
    pub residual_m: f64,
    /// Standard deviation used (metres).
    pub sigma_m: f64,
}

/// Write a residuals CSV file (legacy batch API).
///
/// # Examples
///
/// ```
/// use spod::products::residuals_csv::{ResidualRow, write_residuals_csv};
///
/// let rows = vec![ResidualRow {
///     jd_tt: 2_451_545.0,
///     kind: "code-G01".into(),
///     measured_m: 20_000_000.0,
///     predicted_m: 20_000_001.0,
///     residual_m: -1.0,
///     sigma_m: 1.0,
/// }];
/// let mut buf = Vec::new();
/// write_residuals_csv(&mut buf, &rows).unwrap();
/// let text = String::from_utf8(buf).unwrap();
/// assert!(text.starts_with("jd_tt,kind,"));
/// ```
pub fn write_residuals_csv<W: Write>(
    w: &mut W,
    rows: &[ResidualRow],
) -> Result<(), std::io::Error> {
    writeln!(w, "jd_tt,kind,measured_m,predicted_m,residual_m,sigma_m")?;
    for r in rows {
        writeln!(
            w,
            "{:.9},{},{:.6},{:.6},{:.6},{:.6}",
            r.jd_tt, r.kind, r.measured_m, r.predicted_m, r.residual_m, r.sigma_m
        )?;
    }
    Ok(())
}

// ─── Streaming API ──────────────────────────────────────────────────────────

/// Streaming CSV writer for POD residuals.
///
/// Writes one [`ResidualRecord`] at a time and flushes the underlying sink
/// after every record so that a crash preserves all records written so far.
///
/// The header is written on construction.
///
/// # Examples
///
/// ```
/// use spod::products::residuals_csv::{ResidualCsvWriter, ResidualRecord};
///
/// let mut buf = Vec::<u8>::new();
/// let mut w = ResidualCsvWriter::new(&mut buf).unwrap();
/// w.write_record(&ResidualRecord {
///     epoch_jd_tt: 2_451_545.0,
///     obs_type: "C1C".into(),
///     satellite: "G01".into(),
///     residual_m: -0.15,
///     sigma_m: 0.30,
///     rejected: false,
/// }).unwrap();
/// w.flush().unwrap();
/// let text = String::from_utf8(buf).unwrap();
/// assert!(text.lines().next().unwrap().starts_with("epoch_jd_tt,"));
/// assert_eq!(text.lines().count(), 2);
/// ```
pub struct ResidualCsvWriter<W: Write> {
    writer: W,
}

impl<W: Write> ResidualCsvWriter<W> {
    /// Create a new writer, immediately writing the CSV header line.
    ///
    /// # Errors
    ///
    /// Returns [`PodProductsError::Io`] if writing or flushing the header
    /// to `writer` fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use spod::products::residuals_csv::ResidualCsvWriter;
    ///
    /// let mut buf = Vec::<u8>::new();
    /// let w = ResidualCsvWriter::new(&mut buf).unwrap();
    /// drop(w);
    /// assert!(String::from_utf8(buf).unwrap().starts_with("epoch_jd_tt,"));
    /// ```
    pub fn new(mut writer: W) -> Result<Self, PodProductsError> {
        writeln!(
            writer,
            "epoch_jd_tt,obs_type,satellite,residual_m,sigma_m,rejected"
        )?;
        writer.flush()?;
        Ok(Self { writer })
    }

    /// Write one residual record and flush the sink.
    ///
    /// The flush after each record guarantees that a process crash cannot
    /// silently drop already-written residuals.
    ///
    /// # Errors
    ///
    /// Returns [`PodProductsError::Io`] if the write or flush fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use spod::products::residuals_csv::{ResidualCsvWriter, ResidualRecord};
    ///
    /// let mut buf = Vec::<u8>::new();
    /// let mut w = ResidualCsvWriter::new(&mut buf).unwrap();
    /// w.write_record(&ResidualRecord {
    ///     epoch_jd_tt: 2_451_545.0,
    ///     obs_type: "L1C".into(),
    ///     satellite: "E11".into(),
    ///     residual_m: 0.02,
    ///     sigma_m: 0.01,
    ///     rejected: false,
    /// }).unwrap();
    /// assert_eq!(String::from_utf8(buf).unwrap().lines().count(), 2);
    /// ```
    pub fn write_record(&mut self, r: &ResidualRecord) -> Result<(), PodProductsError> {
        writeln!(
            self.writer,
            "{:.9},{},{},{:.6},{:.6},{}",
            r.epoch_jd_tt, r.obs_type, r.satellite, r.residual_m, r.sigma_m, r.rejected,
        )?;
        self.writer.flush()?;
        Ok(())
    }

    /// Manually flush the underlying writer.
    ///
    /// Normally not needed because [`write_record`][Self::write_record] already
    /// flushes, but useful when the caller wants an explicit sync point.
    ///
    /// # Examples
    ///
    /// ```
    /// use spod::products::residuals_csv::ResidualCsvWriter;
    ///
    /// let mut buf = Vec::<u8>::new();
    /// let mut w = ResidualCsvWriter::new(&mut buf).unwrap();
    /// w.flush().unwrap();
    /// ```
    pub fn flush(&mut self) -> Result<(), PodProductsError> {
        self.writer.flush().map_err(PodProductsError::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_writes_header_plus_rows() {
        let rows = vec![ResidualRow {
            jd_tt: 2_451_545.0,
            kind: "code-G01".into(),
            measured_m: 20_000_000.0,
            predicted_m: 20_000_001.0,
            residual_m: -1.0,
            sigma_m: 1.0,
        }];
        let mut buf = Vec::new();
        write_residuals_csv(&mut buf, &rows).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("jd_tt,kind,"));
        assert!(text.lines().nth(1).unwrap().contains("code-G01"));
    }

    #[test]
    fn streaming_writer_header() {
        let mut buf = Vec::<u8>::new();
        ResidualCsvWriter::new(&mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert_eq!(
            text.trim(),
            "epoch_jd_tt,obs_type,satellite,residual_m,sigma_m,rejected"
        );
    }

    #[test]
    fn streaming_writer_record_count() {
        let mut buf = Vec::<u8>::new();
        let mut w = ResidualCsvWriter::new(&mut buf).unwrap();
        for i in 0..3 {
            w.write_record(&ResidualRecord {
                epoch_jd_tt: 2_451_545.0 + i as f64,
                obs_type: "C1C".into(),
                satellite: "G01".into(),
                residual_m: 0.0,
                sigma_m: 1.0,
                rejected: false,
            })
            .unwrap();
        }
        let text = String::from_utf8(buf).unwrap();
        // 1 header + 3 data rows
        assert_eq!(text.lines().count(), 4);
    }

    #[test]
    fn streaming_writer_rejected_field() {
        let mut buf = Vec::<u8>::new();
        let mut w = ResidualCsvWriter::new(&mut buf).unwrap();
        w.write_record(&ResidualRecord {
            epoch_jd_tt: 2_451_545.0,
            obs_type: "P2".into(),
            satellite: "G03".into(),
            residual_m: 10.0,
            sigma_m: 1.0,
            rejected: true,
        })
        .unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(
            text.contains(",true"),
            "rejected=true must appear in output"
        );
    }
}
