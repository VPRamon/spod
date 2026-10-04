# Changelog

All notable changes to this workspace are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html) per
crate (independent versioning).

## [Unreleased]

### Migration

* Renamed the project to `spod`, including the Cargo package/library, the
  `spod` and `spod-rest` binaries, repository URLs, and Docker image examples.
* Renamed REST configuration variables to `SPOD_REST_BIND` and
  `SPOD_REST_OUT`. The former `SIDERUST_POD_REST_BIND` and
  `SIDERUST_POD_REST_OUT` names remain supported as deprecated fallbacks.
  Update scripts and deployments to use the `SPOD_*` namespace.

### Workspace

* Consolidated the former multi-crate implementation into the current
  `spod` package and migrated its Siderust ecosystem dependencies to
  released crates.io versions.
* Added a `[workspace.lints]` table (`unsafe_code = forbid`,
  `missing_docs = deny`, `clippy::all = deny`,
  `clippy::{todo,unimplemented,dbg_macro} = deny`,
  `rustdoc::broken_intra_doc_links = deny`) and wired every workspace
  member to inherit it via `[lints] workspace = true`. The full
  workspace now builds clean under `cargo clippy --all-targets -- -D
  warnings` and `cargo doc -- -D rustdoc::broken-intra-doc-links`.
* CI scripts under `scripts/`:
  * `check_no_todos.sh` (gates `todo!`/`unimplemented!`/TODO/FIXME/XXX
    comments — currently passes with zero hits in the source tree).
  * `snapshot_public_api.sh` (commits `crates/<crate>/api.snapshot`
    baselines for all 14 library crates via `cargo public-api`; runs
    in `--update` mode locally and as a diff in CI).
* `.github/workflows/ci.yml`: extended the `build-test` job with
  default / no-default-features / all-features test matrices and the
  new `todo-sweep` step, and added an advisory `public-api` job that
  diffs against the committed snapshots.
* Per-crate library-hygiene baseline: every flagged `missing_docs`
  warning across `siderust-{tle,dynamics,lambert,sgp4,spice}` and
  `spod-{core,dynamics}` was resolved (120 warnings → 0)
  with informative rustdoc covering items, fields, variants, and
  units/encoding.
* `spod-estimation::WlsSolverError`: new `Other(String)` variant
  + `WlsSolverError::other()` constructor so callers can wrap upstream
  propagation/STM failures during normal-equation assembly.
* `spod-service::pipeline`: scoped `#[allow(deprecated)]`
  around the legacy `finite_diff_stm_series` call site (upstream notes
  explicitly preserve the series API for batch-LS use, since
  `propagate_stm` only returns Φ at the terminal epoch).

### Fixed

* `spod-observations::gnss::sagnac_km`: convert the typed
  `OMEGA_EARTH_RAD_S` (`InverseSeconds`) to its scalar value before the
  km/(km·s⁻¹) division, restoring `cargo check` after the upstream
  `siderust` typed-constant migration.
* `spod-service`: pipeline & synthetic-arc generation updated
  to the new five-argument `rk4_propagate_series`/`finite_diff_stm_series`
  signatures (which now take a `&DynamicsContext` and return
  `Result<_, DynamicsError>`); errors are wrapped through
  `WlsSolverError::Other`. EKF replay test migrated to the
  `EncodedTime`-based `OrbitState::new` and to
  `StateCovariance::diagonal_from_sigmas` with typed
  `Kilometers`/`KmPerSecond` arguments.

### Added

* `spod_io::oem::{read_oem, OemFile, OemSegment}` — CCSDS OEM
  KVN reader with full round-trip support (`write_oem` ↔ `read_oem`),
  multi-segment parsing, tolerant skipping of `COVARIANCE_*` and
  `MAN_*` sub-blocks, structured `PodIoError::Format` diagnostics with
  line numbers, and a documented inverse `iso8601_to_jd`. Closes the
  long-standing `todo!()` in `tests/functional/oem.rs` (replaced by a
  proper round-trip functional test).

* `spod-core::manifest`: shipped JSON-Schema (draft-07) for the
  `RunManifest` and QC report formats as `schema/run_manifest.v1.json` and
  `schema/qc.v1.json`, embedded at compile time as `RUN_MANIFEST_SCHEMA_V1`
  and `QC_SCHEMA_V1` constants. Two new tests assert the embedded manifest
  schema matches the actual serialised field set and that the QC schema
  declares `schema = "qc.v1"`. Total `spod-core` test count: 8.
* `spod-qc::orbit_compare`: RTN comparison adapter migrated to the
  `siderust 0.7` API (`OrbitState::epoch_jd()`, `LocalOrbitalFrame::try_from_state`).
  Existing tests still pass.

### Changed

* `spod-{io,observations,estimation,qc,products}`: migrated to the
  upstream `siderust` API rename (`OrbitState.epoch_tt → epoch`,
  `OrbitState::new(JulianDate, …) → OrbitState::new_at_jd(JulianDate, …)`,
  `StateCovariance::from_stddevs([f64;3], [f64;3]) →
  StateCovariance::diagonal_from_sigmas([Kilometers;3], [KmPerSeconds;3])`,
  `LocalOrbitalFrame::from_state → try_from_state`). Public APIs of pod
  crates unchanged.

* Reserved sibling reusable crates under `rust/`:
  `siderust-dynamics` (variational/STM, force-model composition,
  thrust-arc physics), `siderust-sgp4` (SGP4 propagator producing typed
  TEME states), `siderust-tle` (TLE/3LE/OMM parser),
  `siderust-spice` (DAF/SPK reader extending `siderust::data::{daf,spk}`
  to Type 3/9/13), and `siderust-lambert` (0/N-revolution Lambert
  solver). Each crate ships with a README defining ownership boundaries
  and a `CHANGELOG.md`. Implementation lands in plan Phases 3–5; the
  scaffolds today only declare crate identity, dependencies on `qtty` /
  `tempoch` / `affn[astro]` / `cheby` / `siderust`, an `AGPL-3.0-or-later`
  license, and `#![forbid(unsafe_code)]`. None of these crates depend on
  `spod-*`.
* `spod-core`: new crate scaffolding the POD domain primitives
  referenced by the existing design plan (`error::PodError`,
  `dataset::DatasetRef`, `manifest::RunManifest` with deterministic JSON,
  `parameter::{ParameterKind, Parameter, ParameterOrdering}`,
  `covariance::ParameterCovariance` with symmetry/diagonal validators,
  `providers::{EphemerisProvider, EarthOrientationProvider,
  FrameTransformProvider}` traits). Closes the long-standing gap where
  the workspace plan and changelog referenced this crate but it did not
  exist on disk.
* `spod-dynamics`: new crate scaffolding the POD-specific
  dynamics composition layer (`force_config::ForceModelConfig`,
  `thrust_arc::ThrustArcConfig` emitting estimable parameters,
  `process_noise::ProcessNoiseConfig`). Numerical propagation and
  analytic STM remain in `siderust` (today) and the future reusable
  `siderust-dynamics` crate; this layer only owns POD-specific
  composition.
* `spod-dynamics`: cannonball solar-radiation-pressure force
  (`forces::CannonballSrp`) and exponential-density atmospheric drag
  (`forces::ExponentialDrag`). (audit fixes A-09, A-10)
* `spod-core`: `PodError::NotImplemented` variant for
  capabilities that are scoped to a future milestone.
* `spod-service`: integration test verifying the runner refuses
  real-input runs with a structured `Unsupported` error rather than
  silently propagating-and-ignoring the inputs (audit fix A-03).
* Top-level `SECURITY.md`, `SUPPORT.md`, `CHANGELOG.md`, and
  `deny.toml` for cargo-deny gating in CI.
* `crates/spod-dynamics/benches/propagation.rs`: criterion
  bench skeleton for two-body and two-body+J2 RK4 propagation
  (audit fix A-13 starter).

### Changed

* `spod-service::runner::run`: when real inputs are supplied
  the runner now returns
  `std::io::ErrorKind::Unsupported` carrying the
  `PodError::NotImplemented` message instead of silently invoking the
  legacy propagation-only stub. This closes audit finding C-01.

### Documentation

* `spod/docs/architecture/dependency-rules.md`: extended forbidden
  edges to include reusable foundational crates (`qtty`, `tempoch`,
  `affn`, `cheby`, `siderust`) which must not depend on any
  `spod-*` crate. Enforced by `scripts/check_dep_graph.sh`.
* `plan.md` §13 records the post-M7 technical audit (six 🔴 critical,
  nine 🟠 high, ten 🟡 medium, five 🟢 low findings) and the M8–M12
  remediation roadmap.
