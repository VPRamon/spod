// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! # Deterministic run manifest
//!
//! ## Scientific scope
//!
//! Reproducible POD requires more than an orbit solution: it also requires
//! a stable record of which inputs, outputs, and software version produced
//! that solution. This module defines the hash-anchored manifest used to
//! capture that provenance for each run.
//!
//! The science represented by the manifest is indirect. It does not model
//! the orbit itself; instead it records the data dependencies and artifacts
//! from which the orbit and diagnostics can be reproduced.
//!
//! ## Technical scope
//!
//! The public items are `DatasetRef`, `RunManifest`, and `canonical_json`.
//! Callers create dataset references from on-disk files, populate manifest
//! fields, and serialize them into a deterministic JSON ordering suitable
//! for hashing and regression checks.
//!
//! Actual estimation, file parsing, and product generation remain elsewhere
//! in the workspace.
//!
//! ## References
//!
//! - Ben-Kiki, O., Evans, C., & d'Otremont, I. (2021). YAML Ain't Markup
//!   Language (YAML) Version 1.2.2.
//! - Bray, T. (2017). The JavaScript Object Notation (JSON) Data
//!   Interchange Format. RFC 8259.
use sha2::{Digest, Sha256};

/// Reference to a single dataset consumed by a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetRef {
    /// Logical role inside the run (e.g. `"gnss-obs"`, `"antex"`, `"sp3-precise"`).
    pub role: String,
    /// File-format identifier (`"SP3"`, `"RINEX-OBS"`, `"ANTEX"`, …).
    pub format: String,
    /// Resolved on-disk path.
    pub path: String,
    /// Lowercased hex SHA-256 of the file content.
    pub sha256: String,
}

impl DatasetRef {
    /// Compute the SHA-256 of a file and build a `DatasetRef`.
    pub fn from_path(role: &str, format: &str, path: &str) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let mut h = Sha256::new();
        h.update(&bytes);
        Ok(Self {
            role: role.to_string(),
            format: format.to_string(),
            path: path.to_string(),
            sha256: hex::encode(h.finalize()),
        })
    }
}

/// Run manifest: enough to reproduce a run, byte for byte (modulo wallclock
/// timestamps which are themselves recorded as parameters).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunManifest {
    /// Manifest schema version.
    pub schema_version: String,
    /// Configuration file hash (SHA-256, lowercase hex).
    pub config_sha256: String,
    /// Inputs.
    pub inputs: Vec<DatasetRef>,
    /// Output product files, with hashes.
    pub outputs: Vec<DatasetRef>,
    /// Software version (semver of `spod-service`).
    pub software_version: String,
    /// Optional free-form notes.
    pub notes: Option<String>,
}

impl RunManifest {
    /// New empty manifest pinned to a software version.
    pub fn new(software_version: impl Into<String>) -> Self {
        Self {
            schema_version: "1.0.0".into(),
            software_version: software_version.into(),
            ..Default::default()
        }
    }

    /// Compute the canonical SHA-256 of the manifest using the deterministic
    /// formatter in [`canonical_json`].
    pub fn canonical_sha256(&self) -> String {
        let mut h = Sha256::new();
        h.update(canonical_json(self).as_bytes());
        hex::encode(h.finalize())
    }
}

/// Deterministic JSON encoder for [`RunManifest`]. Keys are emitted in fixed
/// order; floats are formatted with `{:.17e}` to be byte-stable across
/// platforms.
pub fn canonical_json(m: &RunManifest) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    push_kv_str(&mut s, "schema_version", &m.schema_version, false);
    push_kv_str(&mut s, "software_version", &m.software_version, false);
    push_kv_str(&mut s, "config_sha256", &m.config_sha256, false);
    s.push_str("  \"inputs\": [\n");
    for (i, d) in m.inputs.iter().enumerate() {
        push_dataset(&mut s, d, i + 1 == m.inputs.len());
    }
    s.push_str("  ],\n");
    s.push_str("  \"outputs\": [\n");
    for (i, d) in m.outputs.iter().enumerate() {
        push_dataset(&mut s, d, i + 1 == m.outputs.len());
    }
    s.push_str("  ],\n");
    match &m.notes {
        Some(n) => push_kv_str(&mut s, "notes", n, true),
        None => s.push_str("  \"notes\": null\n"),
    }
    s.push('}');
    s
}

fn push_kv_str(s: &mut String, k: &str, v: &str, last: bool) {
    s.push_str("  \"");
    s.push_str(k);
    s.push_str("\": ");
    s.push('"');
    s.push_str(&escape(v));
    s.push('"');
    s.push_str(if last { "\n" } else { ",\n" });
}

fn push_dataset(s: &mut String, d: &DatasetRef, last: bool) {
    s.push_str("    { \"role\": \"");
    s.push_str(&escape(&d.role));
    s.push_str("\", \"format\": \"");
    s.push_str(&escape(&d.format));
    s.push_str("\", \"path\": \"");
    s.push_str(&escape(&d.path));
    s.push_str("\", \"sha256\": \"");
    s.push_str(&d.sha256);
    s.push_str("\" }");
    s.push_str(if last { "\n" } else { ",\n" });
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_is_deterministic() {
        let m = RunManifest::new("0.0.0");
        let a = canonical_json(&m);
        let b = canonical_json(&m);
        assert_eq!(a, b);
        assert_eq!(m.canonical_sha256(), m.canonical_sha256());
    }
}
