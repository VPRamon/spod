//! End-to-end coverage for the service-owned synthetic run boundary.

use chrono::DateTime;
use siderust::pod::run::dataset::DatasetRef;
use siderust::pod::run::manifest::{RunManifest, RUN_MANIFEST_SCHEMA_V1};
use spod::service::config::{ForcesConfig, InputsConfig};
use spod::service::{run, RunConfig, ServiceError, Workflow};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn base_config(output_dir: &Path) -> RunConfig {
    RunConfig {
        schema_version: "1.0.0".into(),
        run_id: "integration-test".into(),
        workflow: Workflow::Synthetic,
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

fn test_root(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("spod-service-{name}-{suffix}"));
    fs::create_dir_all(&root).unwrap();
    root
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
    assert_eq!(report.workflow, Workflow::Synthetic);
    assert_eq!(report.provenance.inputs.len(), 1);
    for artifact in report.artifacts.all() {
        assert!(
            artifact.path.is_file(),
            "missing {}",
            artifact.path.display()
        );
    }

    let manifest_text = fs::read_to_string(&report.artifacts.manifest.path).unwrap();
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
    let expected_kinds = ["orbit-oem", "orbit-sp3", "qc", "residuals"];
    assert_eq!(manifest.outputs.len(), expected_kinds.len());
    for (output, expected_kind) in manifest.outputs.iter().zip(expected_kinds) {
        assert_eq!(output.kind, expected_kind);
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
        assert!(matches!(err, ServiceError::Unsupported { .. }));
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
    assert!(matches!(err, ServiceError::Unsupported { .. }));
    assert!(
        err.to_string().contains("third_body"),
        "diagnostic did not identify the unsupported third-body force: {err}"
    );
}

#[test]
fn invalid_configuration_fails_before_execution() {
    let output_dir = std::env::temp_dir().join("spod-invalid-config-test");
    let mut cfg = base_config(&output_dir);
    cfg.schema_version = "9.9.9".into();

    let err = run(&cfg, "unused-config.yaml").unwrap_err();
    assert!(matches!(err, ServiceError::Configuration { .. }));
}

#[test]
fn configuration_validation_rejects_empty_run_id_and_disabled_two_body() {
    let output_dir = std::env::temp_dir().join("spod-invalid-semantic-config-test");

    let mut empty_id = base_config(&output_dir);
    empty_id.run_id = "  ".into();
    assert!(matches!(
        empty_id.validate().unwrap_err(),
        ServiceError::Configuration { .. }
    ));

    let mut no_two_body = base_config(&output_dir);
    no_two_body.forces.two_body = false;
    assert!(matches!(
        no_two_body.validate().unwrap_err(),
        ServiceError::Configuration { .. }
    ));
}

#[test]
fn configuration_rejects_unknown_yaml_fields() {
    let root = test_root("unknown-field");
    let config_path = root.join("run.yaml");
    fs::write(
        &config_path,
        "schema_version: 1.0.0\nrun_id: unknown-field\nworkflow: synthetic\ninputs:\n  sp3: null\n  rinex_obs: null\n  rinex_nav: null\n  antex: null\noutput_dir: output\nforces:\n  two_body: true\n  j2: false\n  third_body: false\nunexpected: true\n",
    )
    .unwrap();

    let error = RunConfig::from_yaml_file(&config_path).unwrap_err();
    assert!(matches!(error, ServiceError::Configuration { .. }));
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("unknown field"));
    assert!(diagnostic.contains("unexpected"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn relative_output_directory_is_resolved_from_configuration_file() {
    let root = test_root("relative-output");
    let config_path = root.join("nested/run.yaml");
    let config = base_config(Path::new("products"));
    assert_eq!(
        config.output_dir_relative_to(&config_path),
        root.join("nested/products")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn filesystem_failures_are_artifact_errors() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "spod-artifact-error-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join("run.yaml");
    let output_path = root.join("output-file");
    fs::write(&config_path, "configuration").unwrap();
    fs::write(&output_path, "not a directory").unwrap();

    let err = run(&base_config(&output_path), &config_path).unwrap_err();
    assert!(matches!(err, ServiceError::Artifact { .. }));

    fs::remove_dir_all(root).unwrap();
}
