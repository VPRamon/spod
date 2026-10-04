//! High-level service orchestration boundary.

use super::artifacts::Artifacts;
use super::error::ServiceError;
use super::job::{RunReport, RunRequest};
use super::pipeline::run_synth;
use super::provenance::RunProvenance;
use super::synth::{generate, SyntheticArcConfig};
use super::workflow::Workflow;
use siderust::astro::dynamics::{OrbitState, Position, Velocity};
use siderust::pod::run::dataset::DatasetRef;

/// Stateless runner for supported service workflows.
#[derive(Debug, Default, Clone, Copy)]
pub struct Runner;

impl Runner {
    /// Execute a request through validation, dispatch, execution, and packaging.
    pub fn run(&self, request: RunRequest) -> Result<RunReport, ServiceError> {
        request.config.validate()?;
        let configured_inputs = [
            ("sp3", request.config.inputs.sp3.as_ref()),
            ("rinex_obs", request.config.inputs.rinex_obs.as_ref()),
            ("rinex_nav", request.config.inputs.rinex_nav.as_ref()),
            ("antex", request.config.inputs.antex.as_ref()),
        ]
        .into_iter()
        .filter_map(|(name, value)| value.as_ref().map(|_| name))
        .collect::<Vec<_>>();
        if !configured_inputs.is_empty() {
            return Err(ServiceError::unsupported(format!(
                "real-data ingestion is not implemented; unsupported configured inputs: {}. \
                 Set all entries under `inputs` to null",
                configured_inputs.join(", ")
            )));
        }

        let config_ref = DatasetRef::from_file(&request.config_path, "configuration")
            .map_err(ServiceError::input)?;
        let provenance = RunProvenance::from_config(config_ref, Vec::new());
        match request.config.workflow {
            Workflow::Synthetic => self.run_synthetic(request, provenance),
        }
    }

    fn run_synthetic(
        &self,
        request: RunRequest,
        provenance: RunProvenance,
    ) -> Result<RunReport, ServiceError> {
        let output_dir = request.config.output_dir_relative_to(&request.config_path);
        let arc = generate(&SyntheticArcConfig::default());
        let t0 = arc.truth_states[0];
        let initial = OrbitState::new(
            t0.epoch,
            Position::new(
                t0.position.x().value() + 0.05,
                t0.position.y().value() - 0.05,
                t0.position.z().value() + 0.05,
            ),
            Velocity::new(
                t0.velocity.x().value() + 5e-5,
                t0.velocity.y().value() - 5e-5,
                t0.velocity.z().value() + 5e-5,
            ),
        );
        let report = run_synth(
            &arc,
            initial,
            0.0,
            &output_dir,
            &request.config.run_id,
            request.config.forces.j2,
            provenance.clone(),
        )
        .map_err(|source| match source {
            super::pipeline::PipelineError::Io(source) => ServiceError::Artifact { source },
            source => ServiceError::Scientific { source },
        })?;
        let artifacts = Artifacts::from_output_dir(&output_dir).map_err(ServiceError::artifact)?;
        Ok(RunReport {
            run_id: request.config.run_id,
            workflow: Workflow::Synthetic,
            final_state: report.estimated_final,
            n_steps: arc.epochs.len(),
            estimator_iterations: report.estimator.iterations,
            reduced_chi2: report.estimator.last.reduced_chi2(),
            artifacts,
            provenance,
        })
    }
}

/// Execute a configuration through the service boundary.
pub fn run(
    config: &super::config::RunConfig,
    config_path: impl Into<std::path::PathBuf>,
) -> Result<RunReport, ServiceError> {
    Runner.run(RunRequest {
        config: config.clone(),
        config_path: config_path.into(),
    })
}
