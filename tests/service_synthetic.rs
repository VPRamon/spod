//! End-to-end coverage for the service-owned synthetic run boundary.

use chrono::DateTime;
use siderust::pod::run::dataset::DatasetRef;
use siderust::pod::run::manifest::{RunManifest, RUN_MANIFEST_SCHEMA_V1};
use spod::service::config::{ForcesConfig, InputsConfig};
use spod::service::{run, RunConfig};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn base_config(output_dir: &Path) -> RunConfig {
    RunConfig {
        schema_version: "1.0.0".into(),
        run_id: "integration-test".into(),
        inputs: InputsConfig {
            sp3: None,
            rinex_obs: None,
            rinex_nav: None,
            antex: None,
        },
        output_dir: output_dir.display().to_string(),
        forces: ForcesConfig {
            two_body: true,
            j2: false,
            third_body: false,
        },
    }
}

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

    assert_eq!(manifest.inputs.len(), 1);
    let config_input = &manifest.inputs[0];
    assert_eq!(config_input.kind, "configuration");
    assert_eq!(config_input.sha256, manifest.config_sha256);
    let actual_config = DatasetRef::from_file(&config_path, "configuration").unwrap();
    assert_eq!(config_input.bytes, actual_config.bytes);
    assert_eq!(config_input.sha256, actual_config.sha256);

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

#[test]
fn synthetic_service_rejects_every_real_data_input() {
    let output_dir = std::env::temp_dir().join("spod-unsupported-input-test");

    for input_name in ["sp3", "rinex_obs", "rinex_nav", "antex"] {
        let mut cfg = base_config(&output_dir);
        match input_name {
            "sp3" => cfg.inputs.sp3 = Some("unused.sp3".into()),
            "rinex_obs" => cfg.inputs.rinex_obs = Some("unused.obs".into()),
            "rinex_nav" => cfg.inputs.rinex_nav = Some("unused.nav".into()),
            "antex" => cfg.inputs.antex = Some("unused.atx".into()),
            _ => unreachable!(),
        }

        let err = run(&cfg, "unused-config.yaml").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
        assert!(
            err.to_string().contains(input_name),
            "diagnostic for {input_name} did not identify the unsupported input: {err}"
        );
    }
}

#[test]
fn synthetic_service_rejects_unsupported_third_body_force() {
    let output_dir = std::env::temp_dir().join("spod-unsupported-third-body-test");
    let mut cfg = base_config(&output_dir);
    cfg.forces.third_body = true;

    let err = run(&cfg, "unused-config.yaml").unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        err.to_string().contains("third_body"),
        "diagnostic did not identify the unsupported third-body force: {err}"
    );
}
