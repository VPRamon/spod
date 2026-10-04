//! End-to-end coverage for the service-owned synthetic run boundary.

use chrono::DateTime;
use siderust::pod::run::dataset::DatasetRef;
use siderust::pod::run::manifest::{RunManifest, RUN_MANIFEST_SCHEMA_V1};
use spod::service::{run, RunConfig};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn synthetic_service_produces_valid_manifest_and_products() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("spod-service-test-{}-{suffix}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join("run.yaml");
    let output_dir = root.join("output");
    fs::write(
        &config_path,
        format!(
            "schema_version: 1.0.0\nrun_id: integration-test\ninputs:\n  sp3: null\n  rinex_obs: null\n  rinex_nav: null\n  antex: null\noutput_dir: {}\nforces:\n  two_body: true\n  j2: false\n  third_body: false\n",
            output_dir.display()
        ),
    )
    .unwrap();

    let config: RunConfig = RunConfig::from_yaml_file(config_path.to_str().unwrap()).unwrap();
    let report = run(&config, config_path.to_str().unwrap()).unwrap();
    assert!(report.estimator_iterations > 0);
    assert!(report.reduced_chi2.is_finite());

    let manifest_text = fs::read_to_string(&report.manifest_path).unwrap();
    let manifest: RunManifest = serde_json::from_str(&manifest_text).unwrap();
    assert_eq!(manifest.run_id, "integration-test");
    assert_eq!(manifest.config_sha256.len(), 64);
    assert!(manifest
        .config_sha256
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    assert!(DateTime::parse_from_rfc3339(&manifest.started_at).is_ok());
    assert!(DateTime::parse_from_rfc3339(&manifest.finished_at).is_ok());
    assert!(!manifest.inputs.is_empty());

    let schema: serde_json::Value = serde_json::from_str(RUN_MANIFEST_SCHEMA_V1).unwrap();
    for field in [
        "run_id",
        "tool_version",
        "config_sha256",
        "inputs",
        "outputs",
        "started_at",
        "finished_at",
    ] {
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == field));
    }
    for output in &manifest.outputs {
        let path = if output.path.is_absolute() {
            output.path.clone()
        } else {
            PathBuf::from(&output.path)
        };
        assert!(path.is_file(), "missing output {}", path.display());
        let actual = DatasetRef::from_file(&path, output.kind.clone()).unwrap();
        assert_eq!(actual.bytes, output.bytes);
        assert_eq!(actual.sha256, output.sha256);
    }

    fs::remove_dir_all(root).unwrap();
}
