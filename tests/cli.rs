#![allow(missing_docs)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_root(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("spod-cli-{name}-{suffix}"));
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_config(root: &Path, body: &str) -> PathBuf {
    let path = root.join("run.yaml");
    fs::write(&path, body).unwrap();
    path
}

fn valid_config(output_dir: &Path) -> String {
    format!(
        "schema_version: \"1.0.0\"\nrun_id: cli-integration\nworkflow: synthetic\noutput_dir: {output}\ninputs:\n  sp3: null\n  rinex_obs: null\n  rinex_nav: null\n  antex: null\nforces:\n  two_body: true\n  j2: false\n  third_body: false\n",
        output = output_dir.display()
    )
}

fn cli(config: &Path, command: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_spod"))
        .arg(command)
        .arg(config)
        .output()
        .unwrap()
}

#[test]
fn validate_config_accepts_valid_configuration() {
    let root = test_root("validate-success");
    let config = write_config(&root, &valid_config(&root.join("output")));
    let output = cli(&config, "validate-config");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("validates"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn validate_config_reports_categorized_failure() {
    let root = test_root("validate-failure");
    let config = write_config(
        &root,
        &valid_config(&root.join("output")).replace("1.0.0", "9.9.9"),
    );
    let output = cli(&config, "validate-config");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid configuration"));
    assert!(stderr.contains("schema_version"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn run_reports_success_and_writes_manifest() {
    let root = test_root("run-success");
    let output_dir = root.join("output");
    let config = write_config(&root, &valid_config(&output_dir));
    let output = cli(&config, "run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("succeeded"));
    assert!(output_dir.join("run.manifest.json").is_file());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn run_reports_shared_unsupported_error_category() {
    let root = test_root("run-failure");
    let config = write_config(
        &root,
        &valid_config(&root.join("output")).replace("sp3: null", "sp3: unsupported.sp3"),
    );
    let output = cli(&config, "run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported"));
    assert!(stderr.contains("real-data ingestion is not implemented"));
    fs::remove_dir_all(root).unwrap();
}
