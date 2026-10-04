//! Explicit outputs produced by a service run.

use serde::Serialize;
use siderust::pod::run::dataset::DatasetRef;
use std::path::Path;

/// Canonical on-disk layout for a service run.
#[derive(Debug, Clone)]
pub(crate) struct ArtifactLayout {
    pub(crate) root: std::path::PathBuf,
    pub(crate) orbit_sp3: std::path::PathBuf,
    pub(crate) orbit_oem: std::path::PathBuf,
    pub(crate) residuals_csv: std::path::PathBuf,
    pub(crate) qc_json: std::path::PathBuf,
    pub(crate) manifest: std::path::PathBuf,
}

impl ArtifactLayout {
    pub(crate) fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            orbit_sp3: root.join("products/orbit.sp3"),
            orbit_oem: root.join("products/orbit.oem"),
            residuals_csv: root.join("residuals/residuals.csv"),
            qc_json: root.join("qc/qc.json"),
            manifest: root.join("run.manifest.json"),
        }
    }
}

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
    pub(crate) fn from_layout(layout: &ArtifactLayout) -> Result<Self, std::io::Error> {
        let dataset = |path: &Path, kind: &'static str| DatasetRef::from_file(path, kind);
        Ok(Self {
            orbit_sp3: dataset(&layout.orbit_sp3, "orbit-sp3")?,
            orbit_oem: dataset(&layout.orbit_oem, "orbit-oem")?,
            residuals_csv: dataset(&layout.residuals_csv, "residuals")?,
            qc_json: dataset(&layout.qc_json, "qc")?,
            manifest: dataset(&layout.manifest, "manifest")?,
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
