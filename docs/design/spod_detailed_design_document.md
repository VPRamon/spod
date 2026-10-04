# spod Detailed Design Document

## 0. Document status

**Document type:** Detailed design document
**Target project:** spod / satellite precision-orbit-determination stack
**Primary goal:** Define a concrete architecture and implementation plan for building a FocusPOD-class, Rust-native POD and geodesy product on top of the existing Siderust ecosystem.
**Status:** Draft v0.1
**Audience:** Siderust maintainers, Rust engineers, astrodynamics/POD engineers, validation engineers, and future product owners.

This document assumes the current Siderust organization contains these foundational projects:

- `qtty`: strongly typed physical quantities and units.
- `tempoch`: high-precision time scales, epochs, intervals, EOP and generated time data.
- `affn`: typed frames, centers, affine geometry, Cartesian/spherical/ellipsoidal positions, conics, rotations, translations, and frame/center derivation.
- `cheby`: Chebyshev approximation, interpolation, calculus, quadrature, piecewise series, spectral tools, and compact trajectory-like representations.
- `siderust`: astronomy, ephemerides, Earth orientation, coordinate transforms, observatories, solar/lunar/stellar calculations, targets, data/provenance, spectra/tables, and FFI.
- `NSB`: a domain application proving the model of building specialized applications beside the core Siderust crates.

The design conclusion is that POD must be implemented as a **new sibling product/workspace**, not by expanding the main `siderust` crate into an operations monolith.

---

## 1. Executive summary

The Siderust ecosystem already has a strong scientific substrate: typed units, typed frames/centers, high-precision time handling, ephemerides, Earth orientation, Chebyshev interpolation, and astronomy/observability tools. What it lacks is the product layer required for FocusPOD-class competitiveness: standard POD/geodesy file ingestion, force-model orchestration, measurement modeling, partial derivatives, batch and sequential estimation, ambiguity handling, covariance products, QC reports, reproducible run manifests, and operational execution surfaces.

The proposed design introduces a new product family:

```text
siderust-pod-core
siderust-pod-dynamics
siderust-pod-io
siderust-pod-observations
siderust-pod-estimation
siderust-pod-qc
siderust-pod-products
siderust-pod-service
spod
```

The first competitive target is **GNSS-only LEO POD with float carrier ambiguities, batch weighted least squares, deterministic product generation, and QC artifacts**. The second target is **SLR-validated GNSS POD**. The third target is **near-real-time replay using EKF/sequential estimation**. DORIS, VLBI, GNSS network processing, normal-equation stacking, web UI, RBAC, and distributed operations are later phases.

The central architecture rule is:

```text
qtty       = physical correctness
tempoch    = temporal correctness
affn       = geometric correctness
cheby      = interpolation and spectral correctness
siderust   = astronomical and ephemeris correctness
spod-* = POD, geodesy, estimation, products, QC, and operations
```

---

## 2. Goals and non-goals

## 2.1 Goals

### G-001 — Create a FocusPOD-class open/Rust-native POD stack

The system shall provide a credible path toward modern precise orbit determination and space geodesy workflows, including GNSS, SLR, DORIS, and eventually VLBI.

### G-002 — Preserve Siderust as a reusable scientific kernel

The existing `siderust` crate shall remain focused on astronomy, ephemerides, time/frame integration, observatories, and physical models. POD workflow, file-format, estimator, and service complexity shall be placed in dedicated crates.

### G-003 — Make POD runs reproducible by construction

Every run shall generate a manifest containing input hashes, software versions, model choices, config hash, solver settings, and output product hashes.

### G-004 — Make time, frame, center, and unit mistakes hard

The design shall use `qtty`, `tempoch`, and `affn` to encode physical units, time scales, frames, and centers explicitly wherever feasible.

### G-005 — Support both library and product use

The same computation kernels shall be callable from Rust library APIs, CLI workflows, Python bindings later, and service/job APIs later.

### G-006 — Build public validation credibility

The system shall include a benchmark corpus, synthetic truth cases, regression tests, and public-data test paths for GNSS, SP3, ANTEX, EOP, OEM, and SLR formats.

---

## 2.2 Non-goals for the first release

The first release shall **not** attempt to provide:

- Full GMV FocusPOD parity.
- Full FocusSuite-style mission operations.
- Web UI.
- OAuth2/OpenID/RBAC.
- Distributed Kubernetes operations.
- VLBI.
- DORIS.
- Full GNSS network processing.
- Normal-equation stacking.
- Integer ambiguity resolution.
- Collision avoidance / CAM workflows.
- General mission-control integration.

These are later phases. The first release must prove the POD kernel and reproducibility discipline.

---

## 3. Competitive baseline

## 3.1 FocusPOD baseline

A FocusPOD-class product publicly advertises or implies the following capability families:

- GNSS-based LEO orbit and clock determination.
- SLR processing and validation.
- DORIS processing.
- VLBI processing.
- Weighted least squares and extended Kalman filtering.
- Standard geodesy/POD formats: RINEX, SP3, SINEX, ORBEX, vgosDB, CRD, CPF, ANTEX.
- Modern force and environmental models.
- Residual analysis.
- Combined orbit generation.
- Orbit/clock comparison.
- Geographic and spectral error analysis.
- Sensor-performance analysis.
- Simulation of orbit, clock, attitude, and observations.
- Cloud/service-oriented operation.

## 3.2 Other competing products

The wider competitive set includes:

- Ansys ODTK: strong OD estimation, covariance realism, sequential filtering, many measurement models.
- FreeFlyer: mission design/operations, batch LSQ, EKF, SRIF, UKF, scripting, OD, simulation.
- Orekit: open-source Java astrodynamics and OD framework.
- GMAT: open-source mission analysis/navigation and OD tool.
- Tudat: open-source astrodynamics and estimation research platform.
- GROOPS: open geodesy, gravity-field recovery, GNSS and LEO OD workflows.
- Bernese GNSS Software: high-precision multi-GNSS/geodetic processing.
- GipsyX: JPL geodetic/POD processing with GNSS, SLR, DORIS, Kalman filtering, and ambiguity resolution.

## 3.3 Strategic positioning for Siderust

Siderust should not attempt to win by matching every operational feature immediately. The near-term differentiator should be:

> A Rust-native, type-safe, reproducible, public-validation-driven POD kernel with clean architecture and transparent benchmark cases.

---

## 4. Current Siderust ecosystem assessment

## 4.1 Existing strengths

### `qtty`

Strengths:

- Strongly typed dimensions and units.
- Rich physical quantity catalog.
- Serialization and FFI support.
- `no_std`/feature-conscious design.

POD relevance:

- Prevent unit mistakes in force models, observations, covariance, clocks, and products.

Required extensions:

- Carrier phase cycles.
- TEC / TECU.
- Clock bias and drift types.
- Area-to-mass ratio.
- Radiation pressure.
- Bias and scale-factor representation.

### `tempoch`

Strengths:

- Time scales.
- Tagged times.
- Intervals.
- EOP infrastructure.
- Generated time data.
- Runtime/update tooling.

POD relevance:

- Exact time-scale handling is a core POD requirement.

Required extensions:

- GPS time and GNSS system time integration.
- POD run time-context provenance.
- Explicit EOP and leap-second dataset identity.
- Time-window iteration utilities for arcs and observations.

### `affn`

Strengths:

- Typed frames and centers.
- Cartesian/spherical/ellipsoidal positions.
- Conics.
- Rotations, translations, isometries.
- Compile-time derivation support.

POD relevance:

- Provides the basis for safe orbit states, station positions, observation geometry, and covariance transforms.

Required extensions:

- Orbit-state geometry primitives.
- RTN/RIC, LVLH, VNC frames.
- Covariance metadata and transforms.
- State-vector ordering metadata.

### `cheby`

Strengths:

- Chebyshev series.
- Piecewise interpolation.
- Derivatives and integrals.
- Spectral tools.
- Binary and serde I/O.

POD relevance:

- Precise orbit interpolation.
- Ephemeris interpolation.
- SP3-derived segment representation.
- Spectral QC.
- Efficient trajectory compression.

Required extensions:

- POD-oriented trajectory segment wrappers.
- Error-bounded interpolation for orbit products.
- Position/velocity consistency checks.

### `siderust`

Strengths:

- Ephemerides: VSOP87, ELP2000, optional JPL DE backends.
- Earth rotation, EOP, precession, nutation, polar motion.
- Coordinate transforms.
- Observatories and topocentric calculations.
- Targets and trackable abstractions.
- Data/provenance infrastructure.
- FFI bindings.

POD relevance:

- Source of astronomical and Earth-orientation services for POD dynamics and observation modeling.

Required extensions:

- Public provider traits.
- Stable ephemeris and frame-transform interfaces.
- Avoid exposing downstream crates to private module paths.

---

## 5. Target architecture

## 5.1 Layered architecture

```text
+------------------------------------------------------------------+
|                          spod-ui                         |
|                       optional later web layer                    |
+------------------------------------------------------------------+
|                         siderust-pod-service                     |
|              jobs, configs, manifests, artifacts, API             |
+------------------------------------------------------------------+
|           spod          |   spod-python       |
+------------------------------------------------------------------+
|                        siderust-pod-products                     |
|             SP3, OEM, SINEX-like outputs, residual files          |
+------------------------------------------------------------------+
|       siderust-pod-qc       |       siderust-pod-estimation       |
| residuals, reports, plots   | WLS, EKF, smoothing, covariances    |
+------------------------------------------------------------------+
|      siderust-pod-observations      |      siderust-pod-io         |
| GNSS, SLR, DORIS, VLBI models       | RINEX, SP3, ANTEX, CRD...   |
+------------------------------------------------------------------+
|     siderust-pod-dynamics       |        siderust-pod-core        |
| forces, propagation, STM       | states, params, context, config  |
+------------------------------------------------------------------+
| qtty | tempoch | affn | cheby | siderust                         |
+------------------------------------------------------------------+
```

## 5.2 Dependency rules

Hard dependency rules:

```text
qtty, tempoch, affn, cheby shall not depend on siderust.
siderust may depend on qtty, tempoch, affn, and cheby.
spod-* may depend on qtty, tempoch, affn, cheby, and siderust.
siderust-pod-core shall not depend on siderust-pod-service or CLI crates.
siderust-pod-estimation shall not depend on file-format parsers.
siderust-pod-observations may depend on IO data structures only through canonical models.
siderust-pod-service may depend on all POD crates.
```

Forbidden:

```text
siderust -> spod-*
qtty -> siderust
tempoch -> siderust
affn -> siderust
cheby -> siderust
estimation -> service
observations -> service
core -> service
```

## 5.3 Workspace layout

```text
spod/
  Cargo.toml
  crates/
    siderust-pod-core/
    siderust-pod-dynamics/
    siderust-pod-io/
    siderust-pod-observations/
    siderust-pod-estimation/
    siderust-pod-qc/
    siderust-pod-products/
    siderust-pod-service/
    spod/
  examples/
    configs/
    fixtures/
  docs/
    architecture/
    requirements/
    validation/
    formats/
  tests/
    e2e/
    regression/
    synthetic/
  benches/
```

---

## 6. Crate designs

## 6.1 `siderust-pod-core`

### Purpose

Own POD domain primitives that are not specific to GNSS, SLR, DORIS, VLBI, file formats, or service execution.

### Responsibilities

- OD state representation.
- Spacecraft physical properties.
- Estimation parameter taxonomy.
- Dynamic/observation context types.
- Arc definitions.
- Run configuration schema primitives.
- Provenance primitives.
- Dataset identifiers.
- Error taxonomy.
- Shared constants and conventions.

### Key modules

```text
src/
  lib.rs
  arc.rs
  config.rs
  context.rs
  covariance.rs
  dataset.rs
  error.rs
  manifest.rs
  parameter.rs
  provenance.rs
  spacecraft.rs
  state.rs
  traits.rs
```

### Core types

```rust
pub struct OrbitState<F, C, S = f64> {
    pub epoch: Epoch,
    pub position: Position<F, C, S>,
    pub velocity: Velocity<F, C, S>,
}

pub struct SpacecraftState<F, C, S = f64> {
    pub orbit: OrbitState<F, C, S>,
    pub mass: Option<Mass<S>>,
    pub attitude: Option<AttitudeState<S>>,
    pub properties: Option<SpacecraftProperties<S>>,
}

pub struct ArcDefinition {
    pub id: ArcId,
    pub start: Epoch,
    pub stop: Epoch,
    pub step_hint: Option<Duration>,
}

pub enum ParameterKind {
    InitialPosition,
    InitialVelocity,
    ReceiverClockBias,
    ReceiverClockDrift,
    CarrierAmbiguity,
    DragScale,
    SolarRadiationScale,
    EmpiricalAcceleration,
    StationPosition,
    RangeBias,
    TroposphereZenithDelay,
}

pub struct RunManifest {
    pub run_id: RunId,
    pub config_hash: Sha256,
    pub software: SoftwareManifest,
    pub inputs: Vec<InputManifest>,
    pub outputs: Vec<OutputManifest>,
    pub models: Vec<ModelManifest>,
    pub estimation: Option<EstimationManifest>,
}
```

### Design notes

`OrbitState` should probably live in `siderust-pod-core`, while lower-level position, velocity, frame, and center types should come from `affn`. This avoids forcing `affn` to know what an OD arc, estimation parameter, or spacecraft model is.

---

## 6.2 `siderust-pod-dynamics`

### Purpose

Own force models, propagation, variational equations, state transition matrices, and dynamic model composition.

### Responsibilities

- Force-model traits.
- Two-body, J2, high-degree gravity, third-body, drag, SRP.
- Tides and loading later.
- Maneuvers later.
- STM propagation.
- Variational equations.
- Propagator traits and integrator adapters.

### Key modules

```text
src/
  lib.rs
  context.rs
  forces/
    mod.rs
    two_body.rs
    j2.rs
    spherical_harmonics.rs
    third_body.rs
    drag.rs
    srp.rs
    relativity.rs
    empirical.rs
    composite.rs
  gravity/
    coefficients.rs
    model.rs
    normalization.rs
  integrators/
    mod.rs
    rk.rs
    dop853.rs
  propagation/
    mod.rs
    propagator.rs
    events.rs
  variational/
    mod.rs
    stm.rs
    partials.rs
  frames/
    rtn.rs
    lvlh.rs
    vnc.rs
  error.rs
```

### Core traits

```rust
pub trait ForceModel<F, C, S = f64> {
    type Error;

    fn acceleration(
        &self,
        state: &SpacecraftState<F, C, S>,
        ctx: &DynamicsContext<'_, S>,
    ) -> Result<Acceleration<F, S>, Self::Error>;

    fn partials(
        &self,
        state: &SpacecraftState<F, C, S>,
        ctx: &DynamicsContext<'_, S>,
    ) -> Result<ForcePartials<S>, Self::Error>;
}

pub trait Propagator<F, C, S = f64> {
    type Error;

    fn propagate(
        &self,
        initial: &SpacecraftState<F, C, S>,
        target: Epoch,
        ctx: &DynamicsContext<'_, S>,
    ) -> Result<SpacecraftState<F, C, S>, Self::Error>;
}
```

### MVP force models

POD MVP requires:

- Two-body.
- J2.
- Third-body Sun/Moon using `siderust` ephemeris provider traits.
- High-degree gravity interface, even if only low-degree support is implemented initially.
- Simple atmospheric drag with density trait.
- Cannonball SRP.
- Relativistic correction if available within MVP scope.

### Later force models

- Solid Earth tides.
- Ocean tides.
- Pole tide.
- Atmospheric gravity/loading.
- Earth albedo and IR.
- Antenna power thrust.
- Macro-model SRP.
- Empirical accelerations.
- Impulsive and finite burns.

---

## 6.3 `siderust-pod-io`

### Purpose

Own parsing and writing of external POD/geodesy/mission-analysis formats.

### Responsibilities

- Format version detection.
- Strict and permissive parse modes.
- Canonical typed records.
- Round-trip where possible.
- Structured diagnostics for malformed files.
- Provenance hashing.

### Key modules

```text
src/
  lib.rs
  rinex/
    mod.rs
    obs.rs
    nav.rs
    doris.rs
  sp3/
    mod.rs
    parser.rs
    writer.rs
    records.rs
  antex/
    mod.rs
    antenna.rs
    parser.rs
  sinex/
    mod.rs
    station.rs
    solution.rs
  slr/
    mod.rs
    crd.rs
    cpf.rs
  ccsds/
    mod.rs
    oem.rs
    opm.rs
    tdm.rs
    aem.rs
    omm.rs
  orbex/
    mod.rs
  vgosdb/
    mod.rs
  eop/
    mod.rs
  error.rs
```

### MVP formats

MVP-1:

- SP3 read.
- SP3 write.
- RINEX GNSS OBS subset.
- RINEX GNSS NAV subset.
- ANTEX read.
- EOP bridge into `tempoch` context.
- OEM write.

MVP-2:

- SINEX station/solution subset.
- CRD read.
- CPF read.
- OEM read.

Later:

- ORBEX.
- RINEX DORIS.
- vgosDB.
- TDM, OPM, AEM, OMM.

### Canonical record examples

```rust
pub struct Sp3Record<S = f64> {
    pub epoch: Epoch,
    pub satellite: SpaceObjectId,
    pub position: Position<ItrfLike, EarthCenter, S>,
    pub clock: Option<ClockOffset<S>>,
    pub position_sigma: Option<Vector3<S>>,
    pub clock_sigma: Option<S>,
}

pub struct RinexObservationEpoch<S = f64> {
    pub epoch: Epoch,
    pub receiver: ReceiverId,
    pub observations: Vec<GnssObservation<S>>,
}

pub struct GnssObservation<S = f64> {
    pub satellite: GnssSatelliteId,
    pub signal: GnssSignal,
    pub observable: ObservableKind,
    pub value: ObservationValue<S>,
    pub lli: Option<u8>,
    pub snr: Option<S>,
}
```

---

## 6.4 `siderust-pod-observations`

### Purpose

Own measurement models and corrections.

### Responsibilities

- GNSS code/carrier models.
- SLR range model.
- DORIS model later.
- VLBI model later.
- Observation residual construction.
- Analytic partials.
- Bias and nuisance parameter mapping.
- Measurement simulation.

### Key modules

```text
src/
  lib.rs
  model.rs
  corrections/
    mod.rs
    relativity.rs
    sagnac.rs
    phase_windup.rs
    antenna.rs
    ionosphere.rs
    troposphere.rs
  gnss/
    mod.rs
    constellation.rs
    signal.rs
    observable.rs
    code.rs
    carrier.rs
    clock.rs
    ambiguity.rs
    cycle_slip.rs
  slr/
    mod.rs
    station.rs
    range.rs
    bias.rs
  doris/
    mod.rs
  vlbi/
    mod.rs
  simulation/
    mod.rs
    noise.rs
    outage.rs
  error.rs
```

### Core trait

```rust
pub trait MeasurementModel<State, Params, Obs, Ctx, S = f64> {
    type Error;

    fn predict(
        &self,
        state: &State,
        params: &Params,
        obs: &Obs,
        ctx: &Ctx,
    ) -> Result<Prediction<S>, Self::Error>;

    fn residual(
        &self,
        state: &State,
        params: &Params,
        obs: &Obs,
        ctx: &Ctx,
    ) -> Result<Residual<S>, Self::Error>;

    fn linearize(
        &self,
        state: &State,
        params: &Params,
        obs: &Obs,
        ctx: &Ctx,
    ) -> Result<LinearizedObservation<S>, Self::Error>;
}
```

### GNSS MVP corrections

- Satellite clock.
- Receiver clock.
- Sagnac correction.
- Relativistic correction.
- Antenna phase center.
- Phase wind-up if feasible in MVP.
- Ionosphere-free combinations later in MVP-2.
- DCB/ISB later.
- Troposphere for ground receivers; LEO receiver POD may not need ground troposphere unless processing ground network observations.

---

## 6.5 `siderust-pod-estimation`

### Purpose

Own estimation algorithms and solver infrastructure.

### Responsibilities

- Parameter blocks.
- Residual vectors.
- Design matrices.
- Normal equations.
- Weighted least squares.
- Robust weighting/outlier rejection.
- Nonlinear iteration.
- Covariance extraction.
- EKF/sequential estimation.
- Smoothing.
- Multi-arc support.
- Normal-equation stacking later.

### Key modules

```text
src/
  lib.rs
  parameter.rs
  residual.rs
  design_matrix.rs
  normal_equations.rs
  solver.rs
  batch/
    mod.rs
    weighted_least_squares.rs
    nonlinear.rs
    convergence.rs
  sequential/
    mod.rs
    ekf.rs
    smoother.rs
  robust/
    mod.rs
    huber.rs
    sigma_edit.rs
  covariance.rs
  multi_arc.rs
  ambiguity.rs
  neq.rs
  error.rs
```

### Required solver backends

MVP:

- Dense QR or SVD for small synthetic cases.
- Normal-equation Cholesky/LDLT for production-like batch cases.
- Condition diagnostics.

Later:

- Sparse normal equations.
- Square-root information filter.
- Block-structured solvers.
- Normal-equation export/import.

### Core outputs

Every estimator run shall produce:

- Converged/not converged flag.
- Iteration count.
- Prefit and postfit RMS.
- Parameter correction norm per iteration.
- Normal matrix condition diagnostics.
- Rejected observation count.
- Estimated parameters.
- Covariance and correlation matrices when available.
- Structured warning list.

---

## 6.6 `siderust-pod-qc`

### Purpose

Own all quality-control and validation artifacts.

### Responsibilities

- Residual statistics.
- Residual grouping.
- Orbit overlap comparison.
- Reference orbit comparison.
- RTN/RIC difference statistics.
- Clock comparison.
- SLR residual validation.
- Spectral diagnostics.
- QC JSON schema.
- HTML report generation.

### Key modules

```text
src/
  lib.rs
  residuals/
    mod.rs
    summary.rs
    grouping.rs
  orbit_compare/
    mod.rs
    interpolation.rs
    rtn.rs
    statistics.rs
  clock_compare.rs
  slr_validation.rs
  spectral.rs
  schema.rs
  html.rs
  error.rs
```

### QC artifacts

MVP-1:

- `qc.json`
- `residuals.csv` or `residuals.parquet`
- `summary.json`
- optional `report.html`

MVP-2:

- `orbit_compare.json`
- `slr_residuals.csv`
- `slr_summary.json`
- `overlap_summary.json`

Later:

- Spectral diagnostics.
- Sensor dashboards.
- Web-browsable artifacts.

---

## 6.7 `siderust-pod-products`

### Purpose

Own product generation and product naming/packaging.

### Responsibilities

- SP3 orbit products.
- OEM orbit products.
- SINEX-like solution/covariance products later.
- Residual product packaging.
- Product metadata.
- Product validation.

### Key modules

```text
src/
  lib.rs
  orbit/
    sp3.rs
    oem.rs
  solution/
    sinex.rs
    covariance.rs
  residuals.rs
  manifest.rs
  naming.rs
  package.rs
  error.rs
```

### Product rules

Every product shall include or reference:

- Run ID.
- Config hash.
- Software version.
- Input product hashes.
- Time coverage.
- Satellite/arc ID.
- Model set.
- Product type.
- Quality status.

---

## 6.8 `siderust-pod-service`

### Purpose

Own orchestration and job-level behavior without embedding heavy web/UI assumptions in the computation crates.

### Responsibilities

- Job model.
- Config loading.
- Artifact directories.
- Run manifests.
- Pipeline stages.
- Event hooks later.
- REST API later.
- Worker model later.

### Key modules

```text
src/
  lib.rs
  job.rs
  pipeline.rs
  config.rs
  artifacts.rs
  manifest.rs
  runner.rs
  stages/
    ingest.rs
    prepare.rs
    estimate.rs
    qc.rs
    products.rs
  error.rs
```

### Pipeline stages

```text
1. Load config
2. Validate config
3. Resolve datasets
4. Hash inputs
5. Build time/EOP context
6. Load spacecraft configuration
7. Load observation data
8. Build dynamic model
9. Build observation model
10. Build estimator
11. Solve
12. Generate products
13. Generate QC
14. Write manifest
15. Validate outputs
```

---

## 6.9 `spod`

### Purpose

Expose product workflows to users and CI.

### Commands

```bash
spod validate-config config.yaml
spod run config.yaml
spod inspect-manifest out/run.manifest.json
spod compare-orbits orbit_a.sp3 orbit_b.sp3 --frame rtn
spod qc out/run.manifest.json
spod validate-slr out/orbit.sp3 slr/*.crd
spod simulate config.yaml
```

### CLI rules

- CLI shall call service/library code.
- CLI shall not implement numerical logic directly.
- CLI outputs shall be deterministic.
- CLI errors shall be structured and actionable.

---

## 7. Data model

## 7.1 Dataset model

```rust
pub struct DatasetRef {
    pub id: DatasetId,
    pub path: PathBuf,
    pub format: DatasetFormat,
    pub sha256: Sha256,
    pub time_coverage: Option<TimeCoverage>,
    pub source: Option<String>,
    pub license: Option<String>,
}
```

Dataset formats:

```rust
pub enum DatasetFormat {
    RinexObs,
    RinexNav,
    Sp3,
    Clock,
    Antex,
    Sinex,
    Crd,
    Cpf,
    Oem,
    Opm,
    Aem,
    Tdm,
    Orbex,
    VgosDb,
    Eop,
    GravityField,
    SpacecraftConfig,
}
```

## 7.2 Observation model

```rust
pub struct ObservationRecord<S = f64> {
    pub id: ObservationId,
    pub epoch: Epoch,
    pub source: ObservationSource,
    pub target: SpaceObjectId,
    pub observable: Observable,
    pub value: ObservationValue<S>,
    pub sigma: Option<S>,
    pub flags: ObservationFlags,
    pub provenance: DatasetId,
}
```

Observable kinds:

```rust
pub enum ObservableKind {
    GnssCode,
    GnssCarrierPhase,
    GnssDoppler,
    SlrRange,
    DorisDoppler,
    VlbiDelay,
    Range,
    RangeRate,
    Azimuth,
    Elevation,
    RightAscension,
    Declination,
}
```

## 7.3 Parameter model

```rust
pub struct ParameterBlock<S = f64> {
    pub id: ParameterBlockId,
    pub kind: ParameterKind,
    pub size: usize,
    pub values: Vec<S>,
    pub prior: Option<Prior<S>>,
    pub bounds: Option<Bounds<S>>,
    pub solve_for: bool,
}
```

Parameter block families:

- State vector.
- Receiver clock.
- Satellite clock.
- Carrier ambiguity.
- Drag coefficient or scale.
- SRP coefficient or scale.
- Empirical accelerations.
- Maneuver parameters.
- Station coordinates.
- Range bias.
- Troposphere.
- EOP adjustments later.

## 7.4 Residual model

```rust
pub struct ResidualRecord<S = f64> {
    pub observation_id: ObservationId,
    pub epoch: Epoch,
    pub observable: ObservableKind,
    pub target: SpaceObjectId,
    pub source: ObservationSource,
    pub prefit: Option<S>,
    pub postfit: Option<S>,
    pub sigma: Option<S>,
    pub weight: Option<S>,
    pub normalized: Option<S>,
    pub status: ResidualStatus,
    pub rejection_reason: Option<RejectionReason>,
}
```

## 7.5 Product model

```rust
pub struct ProductManifest {
    pub product_id: ProductId,
    pub product_type: ProductType,
    pub path: PathBuf,
    pub sha256: Sha256,
    pub coverage: TimeCoverage,
    pub quality: ProductQuality,
}
```

Product types:

- Orbit SP3.
- Orbit OEM.
- Residual table.
- QC summary.
- Run manifest.
- HTML report.
- Covariance product.
- Solution SINEX later.

---

## 8. Configuration design

## 8.1 MVP config example

```yaml
version: 1

run:
  id: leo-gnss-mvp1
  mode: batch_lsq
  output_dir: out/leo-gnss-mvp1

arc:
  start: 2026-01-01T00:00:00Z
  stop: 2026-01-02T00:00:00Z
  step_hint_seconds: 30

spacecraft:
  id: demo-leo
  mass_kg: 1200.0
  area_m2: 12.0
  initial_state:
    epoch: 2026-01-01T00:00:00Z
    frame: GCRF
    center: Earth
    position_m: [7000000.0, 0.0, 0.0]
    velocity_mps: [0.0, 7500.0, 1000.0]

inputs:
  rinex_obs:
    - fixtures/gnss/demo.obs
  sp3:
    - fixtures/gnss/igs.sp3
  antex: fixtures/gnss/igs.atx
  eop: fixtures/time/eop.txt
  leap_seconds: fixtures/time/leap-seconds.txt
  gravity: fixtures/gravity/egm2008.gfc

models:
  dynamics:
    gravity:
      model: egm2008
      degree: 20
      order: 20
    j2: true
    third_body:
      sun: true
      moon: true
    drag:
      enabled: true
      density_model: constant
      estimate_scale: true
    srp:
      enabled: true
      model: cannonball
      estimate_scale: true
  observations:
    gnss:
      code: true
      carrier_phase: true
      ambiguity_mode: float
      antenna_phase_center: true
      sagnac: true
      relativity: true

estimation:
  method: batch_lsq
  max_iterations: 8
  convergence:
    parameter_update_norm: 1.0e-6
    residual_rms_delta: 1.0e-5
  robust:
    method: sigma_edit
    threshold: 5.0
  solve_for:
    - initial_state
    - receiver_clock
    - carrier_ambiguities
    - drag_scale
    - srp_scale

outputs:
  orbit_sp3: true
  orbit_oem: true
  residuals_csv: true
  qc_json: true
  html_report: true
  manifest: true
```

## 8.2 Config validation rules

- `arc.start < arc.stop`.
- Input files must exist unless running in dry-run mode.
- Time systems must be explicit or unambiguously inferred from format.
- Force model dependencies must be satisfied.
- Estimation parameters must be compatible with enabled models.
- Output directory must be writable.
- Unsupported format versions must produce explicit errors.
- Network access shall be disabled by default for reproducibility.

---

## 9. Algorithms

## 9.1 Batch weighted least squares

The batch estimator solves nonlinear least squares by iterating:

```text
1. Propagate nominal state over arc.
2. Predict all observations.
3. Compute prefit residuals.
4. Assemble design matrix H.
5. Assemble weight matrix W.
6. Solve normal equations or QR/SVD system.
7. Apply parameter update.
8. Recompute residuals.
9. Check convergence.
10. Emit covariance and QC artifacts.
```

Required numerical diagnostics:

- Normal matrix condition estimate.
- Rank deficiency flags.
- Parameter update norm.
- RMS by observable type.
- RMS by satellite/station/source.
- Rejection count per iteration.
- Warnings for weakly observable parameters.

## 9.2 EKF / sequential estimator

Later MVP phase. The EKF shall share state and parameter definitions with batch LSQ.

Processing loop:

```text
1. Initialize state and covariance.
2. Time update via propagation and STM.
3. Process observations by epoch or batch window.
4. Apply measurement update.
5. Apply robust gating.
6. Persist state/covariance history.
7. Optionally smooth backward.
```

Required diagnostics:

- Innovation residuals.
- Innovation covariance.
- Normalized innovation squared.
- State covariance positive-semidefinite checks.
- Filter divergence detection.
- Restart/gap handling.

## 9.3 Simulation

Simulation is not optional long term. It is essential for validation.

Simulation modes:

- Orbit truth generation.
- Clock truth generation.
- Observation generation.
- Noise injection.
- Bias injection.
- Cycle-slip injection.
- Outage injection.
- Maneuver injection.
- Monte Carlo.

Simulation outputs shall be compatible with estimation inputs.

---

## 10. Requirements specification

## 10.1 High-level requirements

| ID | Requirement | Priority | Owner | Acceptance |
|---|---|---:|---|---|
| HR-001 | The system shall provide a dedicated POD workspace separate from the main `siderust` crate. | P0 | org | Workspace exists and dependency rules pass CI. |
| HR-002 | The system shall support deterministic GNSS-only LEO batch POD. | P0 | pod | Synthetic 24 h benchmark converges and emits products. |
| HR-003 | The system shall ingest SP3, RINEX OBS, RINEX NAV, ANTEX, EOP, and gravity data for MVP-1. | P0 | io | Fixtures parse into canonical records with provenance. |
| HR-004 | The system shall produce SP3/OEM orbit products, residual tables, QC JSON, and run manifests. | P0 | products/qc | Outputs validate and reparse. |
| HR-005 | The system shall support typed time, frame, center, and unit handling across all POD APIs. | P0 | core | Compile-fail and runtime validation tests pass. |
| HR-006 | The system shall expose force models through composable traits with acceleration and partials. | P0 | dynamics | WLS and propagation use same force-model interfaces. |
| HR-007 | The system shall expose measurement models through traits with prediction, residual, and linearization methods. | P0 | observations | GNSS code/carrier models implement the traits. |
| HR-008 | The system shall support nonlinear weighted least squares with robust outlier handling. | P0 | estimation | Synthetic truth cases recover known parameters. |
| HR-009 | The system shall support covariance generation and RTN/RIC covariance transforms. | P0 | core/dynamics/qc | Round-trip covariance tests pass. |
| HR-010 | The system shall support SLR residual validation for GNSS-derived orbits. | P1 | observations/qc | CRD/CPF fixtures produce residual reports. |
| HR-011 | The system shall support EKF/sequential estimation for NRT replay. | P1 | estimation | EKF replay agrees with batch within tolerance on stable synthetic arc. |
| HR-012 | The system shall support simulation of orbit, clock, and observations. | P1 | core/observations | Simulate-estimate-compare loop is reproducible. |
| HR-013 | The system shall support DORIS observation processing. | P2 | observations/io | DORIS synthetic and fixture workflows pass. |
| HR-014 | The system shall support VLBI-ready abstractions and vgosDB ingestion. | P2 | geodesy/io | VLBI session can build normal equations. |
| HR-015 | The system shall expose library, CLI, and later service/API execution without duplicating numerical logic. | P1 | service/cli | CLI and Rust API outputs are equivalent. |

## 10.2 Low-level requirements

| ID | Requirement | Priority | Owner | Acceptance |
|---|---|---:|---|---|
| LR-001 | `tempoch` shall support UTC, TAI, TT, TDB, UT1, and GPS time in POD contexts. | P0 | tempoch | Leap-boundary tests pass. |
| LR-002 | Every POD run shall bind to explicit EOP and leap-second datasets. | P0 | core/service | Manifest includes dataset IDs/hashes. |
| LR-003 | `affn`/POD core shall represent RTN/RIC and LVLH frames. | P0 | affn/core | Frame transform tests pass. |
| LR-004 | `siderust` shall expose public ephemeris and frame-transform provider traits. | P0 | siderust | POD crates do not import private modules. |
| LR-005 | `siderust-pod-dynamics` shall implement two-body and J2 acceleration. | P0 | dynamics | Reference acceleration tests pass. |
| LR-006 | `siderust-pod-dynamics` shall implement third-body acceleration using Siderust ephemerides. | P0 | dynamics | Sun/Moon tests pass. |
| LR-007 | `siderust-pod-dynamics` shall implement a high-degree gravity interface. | P0 | dynamics | Gravity file loads and truncates degree/order. |
| LR-008 | `siderust-pod-dynamics` shall implement simple drag and cannonball SRP. | P0 | dynamics | Synthetic drag/SRP cases pass. |
| LR-009 | `siderust-pod-io` shall parse and write SP3. | P0 | io | Round-trip test passes. |
| LR-010 | `siderust-pod-io` shall parse RINEX OBS and NAV MVP subsets. | P0 | io | Fixtures parse with no silent loss. |
| LR-011 | `siderust-pod-io` shall parse ANTEX. | P0 | io | Antenna corrections queryable. |
| LR-012 | `siderust-pod-observations` shall implement GNSS pseudorange prediction and partials. | P0 | obs | Synthetic residual/Jacobian tests pass. |
| LR-013 | `siderust-pod-observations` shall implement GNSS carrier-phase prediction with float ambiguity. | P0 | obs | Synthetic residual/Jacobian tests pass. |
| LR-014 | GNSS models shall support satellite clock, receiver clock, Sagnac, relativity, and antenna phase-center corrections. | P0 | obs | Individual correction tests pass. |
| LR-015 | `siderust-pod-estimation` shall implement nonlinear WLS. | P0 | estimation | Synthetic OD converges. |
| LR-016 | WLS shall output convergence report, covariance, residuals, and diagnostics. | P0 | estimation | Required fields present. |
| LR-017 | QC shall group residuals by satellite, observable, epoch, elevation, and rejection status. | P0 | qc | QC schema validates. |
| LR-018 | Products shall include hashes and provenance. | P0 | products | Manifest integrity tests pass. |
| LR-019 | SLR CRD/CPF support shall be implemented for validation. | P1 | io/obs | SLR validation report generated. |
| LR-020 | EKF shall support time update, measurement update, process noise, and smoothing. | P1 | estimation | Replay tests pass. |
| LR-021 | Python bindings shall call the same library APIs as CLI. | P2 | python | Output parity test passes. |
| LR-022 | REST service shall call the same service runner as CLI. | P2 | service | Output parity test passes. |

---

## 11. Test strategy

## 11.1 Test categories

```text
Unit tests:
  small isolated numerical and parser tests

Compile-fail tests:
  frame/unit/center misuse tests

Integration tests:
  multi-crate workflows with fixtures

Synthetic truth tests:
  generated orbit/observation truth cases

Public-data regression tests:
  curated frozen public files

Performance benchmarks:
  parse speed, propagation speed, estimator throughput

End-to-end tests:
  config-driven POD run from inputs to products
```

## 11.2 Test matrix

| Test ID | Requirement coverage | Type | Description | Pass criterion |
|---|---|---|---|---|
| ST-001 | LR-001 | Unit | Time-scale round trips across leap seconds. | Error below configured tolerance. |
| ST-002 | LR-002 | Integration | Run manifest records EOP/leap datasets. | Hashes and dataset IDs present. |
| ST-003 | LR-003/HR-009 | Unit | Cartesian covariance to RTN/RIC and back. | Round-trip error below tolerance and PSD preserved. |
| ST-004 | LR-004 | Architecture | POD crates import public APIs only. | CI dependency/private API audit passes. |
| ST-005 | LR-005 | Unit | Two-body energy conservation. | Energy drift below tolerance over one orbit. |
| ST-006 | LR-005 | Unit | J2 acceleration reference vector. | Difference below tolerance. |
| ST-007 | LR-006 | Unit | Third-body Sun/Moon acceleration. | Matches independent calculation. |
| ST-008 | LR-007 | Integration | Gravity model file load/truncation. | Degree/order truncation behaves deterministically. |
| ST-009 | LR-008 | Unit | Drag and SRP analytic cases. | Acceleration matches expected value. |
| ST-010 | LR-009 | Parser | SP3 parse/write/parse. | Semantic round trip for supported fields. |
| ST-011 | LR-010 | Parser | RINEX OBS/NAV parse. | Epochs, observables, metadata preserved. |
| ST-012 | LR-011 | Parser | ANTEX parse and query. | Expected PCO/PCV records returned. |
| ST-013 | LR-012 | Unit | GNSS pseudorange synthetic residual. | Residual near zero for truth. |
| ST-014 | LR-013 | Unit | GNSS carrier synthetic residual. | Residual near zero with known ambiguity. |
| ST-015 | LR-014 | Unit | Sagnac/relativity/clock/antenna corrections. | Matches reference values. |
| ST-016 | LR-015 | Estimation | Linear WLS known-parameter solve. | Parameters recovered. |
| ST-017 | LR-015 | Estimation | Nonlinear synthetic OD solve. | State recovered within tolerance. |
| ST-018 | LR-016 | Integration | Convergence report generation. | All mandatory fields present. |
| ST-019 | LR-017 | QC | Residual grouping. | Grouped stats match fixture expectations. |
| ST-020 | LR-018 | Reproducibility | Deterministic rerun. | Same input/config yields same manifest/product hashes or tolerance-equivalent numeric outputs. |
| ST-021 | LR-019 | Integration | SLR residual validation. | CRD/CPF fixture produces residual report. |
| ST-022 | LR-020 | Estimation | EKF replay on synthetic arc. | EKF stable and agrees with batch within tolerance. |
| ST-023 | HR-002 | E2E | GNSS-only LEO POD from config. | Orbit/residuals/QC/manifest generated. |
| ST-024 | HR-010 | E2E | GNSS POD + SLR validation. | SLR residual summary generated and linked to manifest. |
| ST-025 | HR-015 | API parity | Rust API vs CLI run. | Equivalent outputs. |

## 11.3 Performance benchmarks

| Benchmark ID | Description | Metric |
|---|---|---|
| PB-001 | SP3 parse throughput. | MB/s and records/s. |
| PB-002 | RINEX OBS parse throughput. | MB/s and observations/s. |
| PB-003 | 24 h LEO propagation with MVP force model. | Runtime and max error vs reference. |
| PB-004 | GNSS residual evaluation throughput. | Observations/s. |
| PB-005 | Batch LSQ solve for synthetic 24 h arc. | Wall time, memory, iterations. |
| PB-006 | QC generation throughput. | Residual records/s. |

---

## 12. Milestones

## 12.1 Milestone M0 — Architecture lock

Deliverables:

- Workspace created.
- Crate-boundary document.
- Dependency rules in CI.
- Provider traits designed.
- Config schema draft.

Exit criteria:

- `cargo test --workspace` passes with placeholder crates.
- Architecture document accepted.
- No dependency cycles.

## 12.2 Milestone M1 — Dynamics and state foundation

Deliverables:

- `OrbitState`.
- `SpacecraftState`.
- Dynamics context.
- Two-body, J2, third-body.
- RTN/RIC frame support.
- Covariance transform MVP.

Exit criteria:

- Two-body/J2/third-body tests pass.
- RTN covariance round-trip passes.

## 12.3 Milestone M2 — Format ingestion MVP

Deliverables:

- SP3 read/write.
- RINEX OBS/NAV subset.
- ANTEX read.
- EOP bridge.
- OEM write.

Exit criteria:

- Parser regression suite passes.
- Product round-trip tests pass.

## 12.4 Milestone M3 — Estimation MVP

Deliverables:

- Measurement-model trait.
- GNSS code/carrier prediction.
- WLS nonlinear estimator.
- Residual and convergence reports.

Exit criteria:

- Synthetic GNSS OD recovers truth.
- Jacobians validated against finite differences.

## 12.5 Milestone M4 — POD MVP-1

Deliverables:

- Config-driven GNSS-only LEO POD.
- SP3/OEM outputs.
- Residual table.
- QC JSON.
- Manifest.
- Optional HTML report.

Exit criteria:

- `spod run examples/configs/leo_gnss_mvp1.yaml` completes.
- All required artifacts generated.
- Re-run reproducibility test passes.

## 12.6 Milestone M5 — SLR validation MVP-2

Deliverables:

- CRD/CPF parse.
- SLR range model.
- SLR residual validation command.
- Orbit overlap/reference comparison.

Exit criteria:

- GNSS-derived orbit can be validated against SLR fixtures.
- RTN/RIC comparison artifacts generated.

## 12.7 Milestone M6 — NRT replay MVP-3

Deliverables:

- EKF.
- Sequential replay.
- Windowed processing.
- Latency metrics.
- State/covariance history.

Exit criteria:

- EKF replay stable.
- EKF vs batch comparison within configured threshold.

## 12.8 Milestone M7 — Productization

Deliverables:

- Python bindings.
- REST API.
- Job queue.
- Artifact browsing API.
- Container image.

Exit criteria:

- CLI, Rust API, Python API, and REST call same kernel.
- API parity tests pass.

---

## 13. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Scope expands into full FocusPOD clone too early. | Project stalls. | Enforce MVP-1/MVP-2/MVP-3 sequence. |
| `siderust` becomes monolithic. | Core crate loses clarity. | Keep POD in sibling workspace. |
| Time/frame duplication appears across crates. | Numerical inconsistency. | Use `tempoch`/`affn` as single owners. |
| Estimator design becomes too GNSS-specific. | DORIS/VLBI later become hard. | Keep generic estimation traits separate from observation crates. |
| Format parsers become ad hoc. | Interoperability failures. | Canonical typed records and golden fixtures. |
| Analytic partials are delayed. | Estimation quality/performance suffers. | Validate analytic partials early against finite differences. |
| Public validation data is hard to curate. | Credibility suffers. | Use synthetic truth first, then public frozen fixtures. |
| Performance lags due to dynamic dispatch. | Operational throughput weak. | Support both dynamic configuration and static/generic optimized paths. |
| Covariance semantics are ambiguous. | QC and estimation errors. | Attach state ordering, frame, center, and units to covariance objects. |
| Service layer pollutes core APIs. | Maintenance burden. | Keep service crate at top of dependency graph. |

---

## 14. Open design decisions

| ID | Decision | Options | Recommendation |
|---|---|---|---|
| ODD-001 | Should `OrbitState` live in `affn` or `siderust-pod-core`? | `affn`, `siderust`, `siderust-pod-core` | Put POD semantics in `siderust-pod-core`; keep geometry primitives in `affn`. |
| ODD-002 | Which linear algebra backend? | `nalgebra`, `faer`, `ndarray`, custom traits | Use backend traits; start with one pragmatic backend. |
| ODD-003 | Which format parser strategy? | hand-written parsers, parser combinators, generated parsers | Hand-written robust parsers for MVP formats. |
| ODD-004 | How strict should file parsing be? | strict only, permissive only, both modes | Support strict and permissive modes with diagnostics. |
| ODD-005 | What is the first public benchmark satellite? | synthetic only, Sentinel-like, public LEO GNSS arc | Synthetic first; public LEO GNSS benchmark second. |
| ODD-006 | Should Python bindings be early? | before MVP-1, after MVP-1, after MVP-2 | After MVP-1 unless external users require earlier. |
| ODD-007 | Should service/API be in first year? | yes/no | Only after CLI and library kernel are stable. |
| ODD-008 | Should DORIS precede EKF? | yes/no | No. EKF/NRT path is more broadly useful. |
| ODD-009 | Should integer ambiguity resolution be MVP? | yes/no | No. Start float; add integer fixing later. |
| ODD-010 | Should `siderust-pod-io` be a general `siderust-formats` crate? | POD-specific vs general | Start POD-specific; extract general formats only after APIs stabilize. |

---

## 15. Recommended first GitHub issues

1. Create `spod` workspace skeleton.
2. Add architecture boundary document.
3. Add dependency graph CI check.
4. Define `OrbitState`, `SpacecraftState`, `ArcDefinition`, and `RunManifest`.
5. Define `EphemerisProvider`, `EarthOrientationProvider`, and `FrameTransformProvider` public traits in `siderust` or adapter crate.
6. Implement two-body dynamics test.
7. Implement J2 acceleration test.
8. Implement RTN/RIC frame basis and covariance round-trip test.
9. Implement SP3 parser skeleton.
10. Implement SP3 writer round-trip test.
11. Implement RINEX OBS MVP parser skeleton.
12. Implement ANTEX parser skeleton.
13. Define measurement-model trait.
14. Implement synthetic GNSS pseudorange model.
15. Implement weighted least-squares linear synthetic test.
16. Implement nonlinear synthetic OD test.
17. Define QC JSON schema.
18. Define run manifest schema.
19. Implement CLI `validate-config`.
20. Implement CLI `run` with synthetic pipeline.

---

## 16. Appendix A — Proposed public module surface

```rust
// siderust-pod-core
pub mod arc;
pub mod config;
pub mod context;
pub mod covariance;
pub mod manifest;
pub mod parameter;
pub mod spacecraft;
pub mod state;

// siderust-pod-dynamics
pub mod forces;
pub mod gravity;
pub mod integrators;
pub mod propagation;
pub mod variational;

// siderust-pod-io
pub mod rinex;
pub mod sp3;
pub mod antex;
pub mod sinex;
pub mod slr;
pub mod ccsds;

// siderust-pod-observations
pub mod gnss;
pub mod slr;
pub mod corrections;
pub mod simulation;

// siderust-pod-estimation
pub mod batch;
pub mod sequential;
pub mod robust;
pub mod covariance;
pub mod multi_arc;

// siderust-pod-qc
pub mod residuals;
pub mod orbit_compare;
pub mod slr_validation;
pub mod spectral;
pub mod html;
```

---

## 17. Appendix B — MVP artifact layout

```text
out/leo-gnss-mvp1/
  run.manifest.json
  config.normalized.yaml
  logs/
    run.log
  products/
    orbit.sp3
    orbit.oem
  residuals/
    residuals.csv
  qc/
    qc.json
    summary.json
    report.html
  debug/
    iterations.json
    parameters.json
    covariance.json
```

---

## 18. Appendix C — Definition of done for MVP-1

MVP-1 is done when this command works from a clean checkout with fixtures installed:

```bash
cargo run -p spod -- run examples/configs/leo_gnss_mvp1.yaml
```

and produces:

```text
out/leo-gnss-mvp1/run.manifest.json
out/leo-gnss-mvp1/products/orbit.sp3
out/leo-gnss-mvp1/products/orbit.oem
out/leo-gnss-mvp1/residuals/residuals.csv
out/leo-gnss-mvp1/qc/qc.json
out/leo-gnss-mvp1/qc/report.html
```

with all of the following true:

- Run is deterministic.
- Manifest contains input hashes.
- SP3 output reparses.
- OEM output reparses.
- Residuals include prefit and postfit values.
- QC JSON validates against schema.
- Estimated synthetic orbit meets configured accuracy threshold.
- No private APIs from existing Siderust crates are imported.
- All frame/unit/center misuse tests fail to compile or fail validation explicitly.

---

## 19. Final recommendation

The next implementation step should not be another feature inside `siderust/src`. It should be the creation of a dedicated `spod` workspace with a minimal, test-first GNSS POD pipeline.

The first build target is:

```text
Synthetic GNSS-only LEO batch POD
  -> typed state
  -> SP3/RINEX/ANTEX fixtures
  -> GNSS code/carrier residuals
  -> nonlinear WLS
  -> orbit product
  -> residual file
  -> QC JSON
  -> manifest
```

After that, the project should add SLR validation, then EKF/NRT replay, then public-data benchmarks, then DORIS/VLBI/geodesy expansion, and only then service/UI/productization.

This path gives Siderust the strongest chance to compete credibly: use its current strengths as a scientific kernel, avoid monolithic scope creep, and build a POD product layer with explicit validation and operational discipline.
