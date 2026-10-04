//! Explicit outputs produced by a service run.

use serde::Serialize;
use siderust::pod::run::dataset::DatasetRef;
use std::path::Path;

/// The deterministic artifact set produced by the synthetic workflow.
#[derive(Debug, Clone, Serialize)]
pub struct Artifacts {
    /// SP3 orbit product.
    pub orbit_sp3: DatasetRef,
    /// CCSDS OEM orbit product.
    pub orbit_oem: DatasetRef,
    /// Residual records.
    pub residuals_csv: DatasetRef,
    /// Quality-control document.
    pub qc_json: DatasetRef,
    /// Run manifest.
    pub manifest: DatasetRef,
}

impl Artifacts {
    pub(crate) fn from_output_dir(output_dir: &Path) -> Result<Self, std::io::Error> {
        let dataset =
            |path: std::path::PathBuf, kind: &'static str| DatasetRef::from_file(path, kind);
        Ok(Self {
            orbit_sp3: dataset(output_dir.join("products/orbit.sp3"), "orbit-sp3")?,
            orbit_oem: dataset(output_dir.join("products/orbit.oem"), "orbit-oem")?,
            residuals_csv: dataset(output_dir.join("residuals/residuals.csv"), "residuals")?,
            qc_json: dataset(output_dir.join("qc/qc.json"), "qc")?,
            manifest: dataset(output_dir.join("run.manifest.json"), "manifest")?,
        })
    }

    /// Return all generated artifacts in manifest order.
    pub fn all(&self) -> [&DatasetRef; 5] {
        [
            &self.orbit_sp3,
            &self.orbit_oem,
            &self.residuals_csv,
            &self.qc_json,
            &self.manifest,
        ]
    }
}
