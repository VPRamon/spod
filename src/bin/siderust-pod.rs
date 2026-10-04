//! # POD command-line interface
//!
//! Operator-facing CLI; `println!` is the intended user output channel.
#![allow(clippy::print_stdout)]
//!
//! ## Scientific scope
//!
//! This binary is an operational entry point for batch POD workflows. It
//! does not implement orbital physics directly; instead it exposes
//! validation, execution, and artifact-inspection commands around the
//! deterministic synthetic and configuration-driven flows implemented in
//! the service crate.
//!
//! Its scientific regime is therefore inherited from the selected pipeline
//! path. In the current MVP workflow that means short synthetic GNSS orbit-
//! determination runs and post-run inspection of generated JSON products.
//!
//! ## Technical scope
//!
//! The executable uses `clap` to expose `validate-config`, `run`, `inspect-
//! manifest`, and `qc` style operations. Inputs are filesystem paths to
//! YAML configuration files or generated JSON artifacts, and outputs are
//! terminal status messages or the raw text of previously produced manifest
//! and QC documents.
//!
//! It deliberately does not own schema evolution, force-model selection, or
//! estimation logic; those responsibilities remain in `siderust-pod-
//! service`.
//!
//! ## References
//!
//! - Ben-Kiki, O., Evans, C., & d'Otremont, I. (2021). YAML Ain't Markup
//!   Language (YAML) Version 1.2.2.
//! - Bray, T. (2017). The JavaScript Object Notation (JSON) Data
//!   Interchange Format. RFC 8259.
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "siderust-pod",
    about = "Siderust POD command-line interface",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Validate a run configuration without executing it.
    ValidateConfig {
        /// Path to the YAML run configuration.
        config: String,
    },
    /// Run a configuration end-to-end.
    Run {
        /// Path to the YAML run configuration.
        config: String,
    },
    /// Pretty-print a previously written run manifest.
    InspectManifest {
        /// Path to `run.manifest.json`.
        manifest: String,
    },
    /// Pretty-print a previously written `qc.json`.
    Qc {
        /// Path to `qc.json`.
        qc: String,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    match cli.command {
        Cmd::ValidateConfig { config } => {
            let cfg = siderust_pod::service::RunConfig::from_yaml_file(&config)?;
            cfg.validate().map_err(|e| anyhow::anyhow!(e))?;
            println!("OK: {} validates", config);
        }
        Cmd::Run { config } => {
            let cfg = siderust_pod::service::RunConfig::from_yaml_file(&config)?;
            let report = siderust_pod::service::run(&cfg, &config)?;
            println!(
                "OK: ran {} steps, final epoch JD={}, manifest at {}",
                report.n_steps,
                report
                    .final_state
                    .epoch
                    .to::<siderust::tempoch::JD>()
                    .value(),
                report.manifest_path.display()
            );
        }
        Cmd::InspectManifest { manifest } => {
            let txt = std::fs::read_to_string(&manifest)?;
            println!("{}", txt);
        }
        Cmd::Qc { qc } => {
            let txt = std::fs::read_to_string(&qc)?;
            println!("{}", txt);
        }
    }
    Ok(())
}
