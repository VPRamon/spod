# spod — Implementation Plan

> **Editable workspace:** `rust/spod/`
> **Untouchable upstream:** `rust/siderust/`, `rust/qtty/`, `rust/tempoch/`, `rust/affn/`, `rust/cheby/`
> **Modifications to foundational crates:** require a separate branch off `main`, written justification, and must be avoided unless strictly necessary. Always prefer adapter layers, wrapper crates, or public provider traits in `spod` first.
> **Attribution rule:** never add Copilot as a co-author anywhere (commits, changelogs, files, docs).

---

## 1. Current-state assessment

### 1.1 What exists in `spod/`

`spod/` is currently a **verbatim fork of the `siderust` crate**, not a POD product workspace:

- `Cargo.toml` declares `name = "siderust"` (a single-crate package) plus a workspace whose only members are `.` and `siderust-ffi`.
- `src/` is the `siderust` source tree (`astro`, `bodies`, `atmosphere`, `calculus`, `coordinates`, `targets`, `archive`, `provenance`, `observatories`, `time.rs`, `interp.rs`, `spectra`, `tables`, etc.). None of it is POD-domain code.
- `siderust-ffi/` is the FFI crate from upstream.
- `memory/` already contains the authoritative inputs:
  - `spod_detailed_design_document.md` (1824 lines — the design contract this plan implements)
  - `research-requirements-tests-cases.md` (FocusPOD/competitor research, 341 lines)
- `doc/` carries upstream architecture/conventions/datasets/frames docs (useful to keep as reference).
- `examples/`, `benches/`, `tests/` are upstream-shaped, not POD-shaped.
- No POD crate exists yet (`siderust-pod-core`, `-dynamics`, `-io`, `-observations`, `-estimation`, `-qc`, `-products`, `-service`, `-cli` are all absent).

**Conclusion:** the M0 milestone must *reset* `spod/` into a real Cargo workspace whose members are the new POD crates, depending on upstream `siderust`/`affn`/`qtty`/`tempoch`/`cheby` as external crates. The cloned `src/` is to be deleted (decision locked with user).

### 1.2 Reusable foundation (no modification)

| Upstream crate | POD-relevant capability already present | Reuse strategy |
|---|---|---|
| `qtty` | `Quantity<U,S>` typed units across `f64`/`f32`; angles, lengths, masses, times, etc. `qtty-core`/`qtty-derive`/`qtty-ffi`. | Use directly. New POD-specific units (carrier cycles, TECU, clock bias, area-to-mass) live in `siderust-pod-core` as `qtty` newtypes around existing dimensions; only file an upstream `qtty` change if a *new dimension* is unavoidable. |
| `tempoch` | `Time<S, Scale>`, `TT/TAI/UTC/UT1/TDB/TCB/TCG/GPS`, `Interval`, `Period`, `eop`, leap seconds, EOP context. | Use directly. POD-run "time context" wraps an explicit `EopDataset` + `LeapSecondDataset` reference (provenance lives in `siderust-pod-core`). |
| `affn` | `Point`/`Vector` algebra with `qtty` units, frame/center markers, rotations, isometries, conics, `Position - Position = Vector`. | Use directly for `OrbitState` geometry, station positions, residual geometry. POD-only frames (RTN/RIC, LVLH, VNC) defined as new marker types in `siderust-pod-core::frames` *implementing* `affn` traits, not modifying `affn`. |
| `cheby` | Chebyshev series, piecewise interpolation, derivatives, spectral tools, binary/serde I/O. | Use directly for SP3 segment representation, propagated-orbit interpolation, and spectral QC. New POD wrappers (e.g. `OrbitSegment`) live in `siderust-pod-core`. |
| `siderust` | VSOP87, ELP2000, optional DE440/DE441; precession, nutation, polar motion, ERA, EOP plumbing; observatories; targets. | Consume through *new public provider traits* (`EphemerisProvider`, `EarthOrientationProvider`, `FrameTransformProvider`) added in `siderust-pod-core::providers` as a thin adapter over current `siderust` APIs. Only escalate to an upstream PR if a needed capability is gated behind private modules. |

### 1.3 Duplication, coupling, architectural risks observed

- **Package-name collision.** `spod/Cargo.toml` declares `name = "siderust"` — this guarantees confusion and would break any consumer that depends on both. Must be renamed in M0.
- **Vendored copy drift risk.** Carrying a full clone of `siderust` source inside `spod/src/` invites accidental edits and silent divergence. Removing it is the only safe option.
- **Scope creep risk.** The design doc explicitly warns against turning `siderust` into a POD monolith and against rebuilding FocusPOD wholesale in v1. The plan honors the M0 → M7 sequencing.
- **Estimator coupling risk.** Without explicit dependency rules, IO/parser code can leak into estimator crates. The plan codifies forbidden edges and enforces them with a CI check (`cargo-deny` + a small graph script).
- **Provider-trait risk.** If POD code reaches into `siderust` private modules, every upstream refactor breaks POD. The plan introduces a thin `siderust-pod-core::providers` adapter that *only* uses public `siderust` items.

---

## 2. Proposed final architecture

### 2.1 Workspace layout

```text
rust/spod/
  Cargo.toml                       # virtual workspace, no [package]
  rust-toolchain.toml
  .cargo/config.toml
  crates/
    siderust-pod-core/             # domain primitives, providers, manifest, errors
    siderust-pod-dynamics/         # forces, propagation, STM, RTN/RIC
    siderust-pod-io/               # SP3, RINEX, ANTEX, EOP, OEM, …
    siderust-pod-observations/     # GNSS/SLR/DORIS/VLBI measurement models
    siderust-pod-estimation/       # WLS/EKF/robust/covariance (faer)
    siderust-pod-qc/               # residuals, RTN/RIC compare, JSON, HTML
    siderust-pod-products/         # SP3/OEM writers, packaging, naming
    siderust-pod-service/          # job/pipeline runner, config, artifacts
    spod/              # thin CLI over service
  examples/
    configs/
      leo_gnss_mvp1.yaml
    fixtures/
      gnss/  time/  gravity/  spacecraft/
  docs/
    architecture/
    conventions/
    requirements/
    formats/
    validation/
  tests/
    e2e/  regression/  synthetic/
  benches/
  scripts/
    check_dep_graph.sh             # CI dependency-direction enforcement
```

The upstream `siderust-ffi` crate that currently lives under `spod/siderust-ffi` is removed from this workspace (it belongs in `rust/siderust/`). POD's own future FFI lives in `crates/spod-ffi/` and is **not** part of MVP-1.

### 2.2 Crate responsibilities

| Crate | Owns | Does not own |
|---|---|---|
| `siderust-pod-core` | `OrbitState`, `SpacecraftState`, `ArcDefinition`, `RunManifest`, `ParameterKind`, `Covariance`, `DatasetRef`, error taxonomy, **provider traits** (`EphemerisProvider`, `EarthOrientationProvider`, `FrameTransformProvider`, `GravityFieldProvider`, `AtmosphereDensityProvider`), POD-specific frame markers (RTN/RIC/LVLH/VNC), config schema primitives. | Numerical algorithms; file parsing; estimator math. |
| `siderust-pod-dynamics` | `ForceModel` trait, `Propagator` trait, two-body/J2/spherical-harmonics/third-body/drag/SRP/relativity/empirical, integrators (RK4 → DOP853), STM/variational, RTN/LVLH/VNC frame transforms. | File I/O; observation models; estimator. |
| `siderust-pod-io` | Parsers/writers for SP3, RINEX OBS/NAV, ANTEX, EOP, OEM (MVP-1) and SINEX/CRD/CPF/ORBEX/CCSDS-OPM/AEM/TDM/OMM/RINEX-DORIS/vgosDB (later). Canonical typed records. Round-trip & strict/permissive parse modes. SHA-256 of input files. | Numerics; estimator; service. |
| `siderust-pod-observations` | `MeasurementModel` trait, GNSS code/carrier prediction + analytic partials, corrections (clock, Sagnac, relativity, antenna phase center, phase wind-up, troposphere, ionosphere), simulation. | File parsing; estimator solver. |
| `siderust-pod-estimation` | Parameter blocks, residual/design-matrix assembly (faer-backed), WLS, nonlinear iteration, robust weighting, covariance extraction, EKF/smoothing later, ambiguity (float now, integer later). | File parsing; observation modeling; service orchestration. |
| `siderust-pod-qc` | Residual statistics, grouping (sat/obs/elev/epoch), orbit overlap, RTN/RIC compare, SLR validation, QC JSON schema, HTML report. | Estimation algorithms; product writing. |
| `siderust-pod-products` | SP3/OEM writers (delegate parsing to `-io`), residual product packaging, manifest/product naming, product validation. | Numerics; service runtime. |
| `siderust-pod-service` | Job model, config load/validate, pipeline stages (ingest → prepare → estimate → qc → products → manifest), artifact layout, hashing, deterministic logging. Future REST/job queue. | Numerical algorithms. |
| `spod` | `clap`-based commands; pure passthrough to `-service` and library APIs; deterministic exit codes. | Any numerical logic. |

### 2.3 Allowed and forbidden dependency directions

```text
ALLOWED:
  qtty, tempoch, affn, cheby   ←— independent foundations
  siderust  →  qtty, tempoch, affn, cheby
  pod-core  →  siderust, qtty, tempoch, affn, cheby
  pod-dynamics →  pod-core, siderust, cheby (for trajectory segments)
  pod-io       →  pod-core
  pod-observations →  pod-core, pod-dynamics (for state propagation context only via traits), siderust
  pod-estimation   →  pod-core, faer
  pod-qc           →  pod-core, pod-products (read-only types), cheby (spectral)
  pod-products     →  pod-core, pod-io (writers reuse parsers)
  pod-service      →  ALL pod-* crates
  pod-cli          →  pod-service (only)

FORBIDDEN:
  siderust          →  pod-*                    (upstream must not know POD)
  qtty/tempoch/affn/cheby  →  siderust or pod-* (foundations stay foundational)
  pod-core          →  pod-service, pod-cli, pod-io, pod-dynamics, pod-observations, pod-estimation, pod-qc, pod-products
  pod-estimation    →  pod-io, pod-observations  (estimator solves algebra; observation models are injected via traits)
  pod-observations  →  pod-io                   (observation models receive canonical typed records, not raw files)
  pod-dynamics      →  pod-io, pod-observations, pod-estimation
  pod-qc            →  pod-estimation           (QC consumes residuals as data)
  pod-cli           →  pod-{core,dynamics,io,observations,estimation,qc,products}  directly  (must go through pod-service)
```

The `scripts/check_dep_graph.sh` CI step parses each crate's `Cargo.toml` and fails if any forbidden edge appears.

### 2.4 Separation-of-concerns guarantees

- **Time** lives in `tempoch` only.
- **Units** live in `qtty` only.
- **Geometry** lives in `affn` only (positions, vectors, rotations, conics).
- **Astronomy** lives in `siderust` only, exposed to POD through provider traits.
- **POD domain semantics** (arcs, parameters, residuals, manifests, run config) live in `siderust-pod-core` only.
- **File formats** live in `siderust-pod-io` only; everything else consumes canonical typed records.
- **Estimator** receives data through traits and parameter blocks; it does not know what GNSS or SLR is.
- **QC** consumes residual records; it does not run estimators.
- **Products** consume estimator outputs; they do not run estimators.
- **Service** orchestrates; CLI presents.

### 2.5 Surface stratification

- **Library-only:** `pod-core`, `pod-dynamics`, `pod-io`, `pod-observations`, `pod-estimation`, `pod-qc`, `pod-products`.
- **Library + service-only:** `pod-service` (Rust API + later REST).
- **CLI-only:** `pod-cli`.
- **Future UI/API:** layered above `pod-service`. No UI/API code in MVP-1 through MVP-3.

---

## 3. Implementation milestones

Milestones are numbered M0–M7. Each lists *goal*, *crates touched*, *concrete tasks*, *expected outputs*, *tests*, *definition of done*, *risks*.

### M0 — Workspace skeleton & dependency lock

- **Goal:** real virtual workspace; package collision removed; dependency-direction CI live; provider traits compile against upstream `siderust`.
- **Crates touched:** all (created), `pod-core` (real code), others (placeholder `lib.rs`).
- **Concrete tasks:**
  - Delete `spod/src/`, `spod/build.rs`, `spod/siderust-ffi/`, upstream-shaped `examples/`/`benches/`/`tests/`, upstream `archive/` data committed under `src/`.
  - Replace root `Cargo.toml` with a `[workspace]` (no `[package]`), `members = ["crates/*"]`, `resolver = "2"`.
  - `cargo new --lib crates/spod-{core,dynamics,io,observations,estimation,qc,products,service}` and `cargo new crates/spod`.
  - Per-crate `Cargo.toml`: AGPL-3.0, version `0.0.0`, `repository`/`license`/`readme` fields, MSRV pin (latest stable −1).
  - Add the Siderust ecosystem crates as released registry dependencies.
  - In `pod-core`: define provider traits (`EphemerisProvider`, `EarthOrientationProvider`, `FrameTransformProvider`, `GravityFieldProvider`, `AtmosphereDensityProvider`) with default impls that wrap public `siderust` APIs.
  - `scripts/check_dep_graph.sh` enforcing §2.3.
  - `.github/workflows/ci.yml`: `fmt`, `clippy -- -D warnings`, `test --workspace`, `check_dep_graph.sh`, `cargo deny check`, `cargo doc --no-deps`.
  - `docs/architecture/{boundaries.md,dependency-rules.md,providers.md}` (short, normative).
  - Move `memory/` to `docs/design/` (keep verbatim; it's the design contract).
- **Expected outputs:** compiling empty workspace; CI green; documented architecture.
- **Tests:** `cargo test --workspace` (placeholder), dep-graph script self-test, doc-build.
- **Definition of done:** all of the above + a short ADR `docs/architecture/ADR-0001-workspace-reset.md` recording the wipe.
- **Risks:** none numerical; risk is bikeshedding the boundary doc — timebox and commit.

### M1 — State, dynamics foundation, RTN/RIC

- **Goal:** typed `OrbitState`/`SpacecraftState`, dynamics context, two-body/J2/third-body forces, RK4 + DOP853 integrators, RTN/RIC frame, Cartesian↔RTN covariance round-trip.
- **Crates touched:** `pod-core`, `pod-dynamics`.
- **Concrete tasks:**
  - `pod-core`: `state.rs` (`OrbitState<F,C,S>`, `SpacecraftState`), `arc.rs`, `spacecraft.rs`, `parameter.rs`, `covariance.rs` (typed 6×6 carrying frame/center/order/units), `frames/{rtn.rs,lvlh.rs,vnc.rs}` defining marker types and `affn` impls.
  - `pod-dynamics`: `forces/{two_body,j2,third_body,composite}.rs`, `integrators/{rk4,dop853}.rs`, `propagation/propagator.rs`, `variational/{stm,partials}.rs` (skeleton with two-body STM only), `frames/rtn.rs` (computes basis from state).
  - Wire `third_body` to `EphemerisProvider`.
- **Expected outputs:** propagation API; covariance transform; reference J2 / two-body acceleration values matching closed-form.
- **Tests:** ST-005 two-body energy conservation, ST-006 J2 reference vector, ST-007 third-body Sun/Moon vs independent calc, ST-003 covariance Cartesian↔RTN round-trip + PSD preservation; STM finite-difference check on two-body.
- **Definition of done:** all listed tests pass at default tolerances, doc page `docs/architecture/dynamics.md` written.
- **Risks:** STM correctness — mitigation: finite-difference validation gate is mandatory; expand to J2 STM in M3, not M1.

### M2 — Format ingestion MVP-1

- **Goal:** parse/write SP3, parse RINEX OBS/NAV (MVP subset), parse ANTEX, EOP bridge into `tempoch::TimeContext`, write OEM.
- **Crates touched:** `pod-io`, small additions in `pod-core` (`DatasetRef`, dataset format enum).
- **Concrete tasks:**
  - `pod-io::sp3` reader+writer with SHA-256, strict/permissive modes, structured diagnostics.
  - `pod-io::rinex::{obs,nav}` MVP subset (GPS L1/L2 P/C, Galileo E1/E5a; broadcast NAV).
  - `pod-io::antex` PCO/PCV reader.
  - `pod-io::eop` adapter producing `tempoch::eop::EopDataset` (without modifying tempoch).
  - `pod-io::ccsds::oem` writer.
- **Expected outputs:** typed records (`Sp3Record`, `RinexObservationEpoch`, `AntexRecord`, `OemRecord`) + provenance.
- **Tests:** ST-010 SP3 round-trip, ST-011 RINEX OBS/NAV parse, ST-012 ANTEX query, OEM write+reparse.
- **Definition of done:** parser regression suite green; fixtures in `examples/fixtures/`.
- **Risks:** RINEX OBS edge cases — scope to GPS+Galileo + plain header for MVP-1; document deferred edge cases.

### M3 — Estimation MVP

- **Goal:** GNSS code+carrier `MeasurementModel`, weighted least-squares solver (faer), covariance extraction, analytic partials validated.
- **Crates touched:** `pod-observations`, `pod-estimation`.
- **Concrete tasks:**
  - `pod-observations::model::MeasurementModel` trait; `gnss::{code,carrier,clock,ambiguity}`; `corrections::{sagnac,relativity,antenna,phase_windup}`.
  - `pod-estimation::{parameter,residual,design_matrix,normal_equations}.rs`; `batch::{weighted_least_squares,nonlinear,convergence}`; `robust::{huber,sigma_edit}`; `covariance.rs`.
  - faer-backed dense Cholesky/QR; condition diagnostics.
  - Analytic-vs-finite-difference Jacobian gate.
- **Expected outputs:** library API `BatchEstimator::solve(...)` returning `EstimationReport`.
- **Tests:** ST-013 code residual, ST-014 carrier residual, ST-015 corrections, ST-016 linear WLS, ST-017 nonlinear synthetic OD, ST-018 convergence report fields, finite-diff Jacobian.
- **Definition of done:** synthetic GNSS arc converges to truth within tolerance.
- **Risks:** ambiguity float-only; document explicitly that integer fixing is post-MVP.

### M4 — POD MVP-1 end-to-end (config-driven)

- **Goal:** `spod run examples/configs/leo_gnss_mvp1.yaml` produces all required artifacts; reproducible; manifest with input hashes.
- **Crates touched:** `pod-service`, `pod-cli`, `pod-products`, `pod-qc`.
- **Concrete tasks:**
  - `pod-service::config` (YAML + `serde` + JSON-Schema export + `validate-config` command); pipeline stages exactly as design §6.8.
  - `pod-products::orbit::{sp3,oem}` writers; `pod-products::residuals` (CSV first, Parquet behind `parquet` feature), `pod-products::manifest` (canonical JSON, sorted keys, fixed float format).
  - `pod-qc::residuals::{summary,grouping}`; `pod-qc::schema` (JSON-Schema published in `docs/validation/`); optional `pod-qc::html` (minimal templated report).
  - `pod-cli` commands: `validate-config`, `run`, `inspect-manifest`, `qc`.
  - End-to-end fixture: synthetic 24 h LEO GNSS arc generated via simulation harness in `tests/synthetic/`.
- **Expected outputs:** the artifact tree of design §17 (`run.manifest.json`, `products/orbit.{sp3,oem}`, `residuals/residuals.csv`, `qc/{qc.json,summary.json,report.html}`, `debug/*`).
- **Tests:** ST-020 reproducibility (rerun = same hashes within configured float tolerance for floating outputs and exact for canonical JSON), ST-023 end-to-end, ST-025 CLI vs Rust-API parity.
- **Definition of done:** the design's §18 checklist (all 9 bullets) passes from clean checkout.
- **Risks:** non-determinism in float formatting / hashmap iteration — mitigation: canonical JSON encoder + sorted iteration + locked rayon seeds.

### M5 — SLR validation (MVP-2)

- **Goal:** validate GNSS-derived orbit against SLR observations; orbit overlap and reference-orbit comparison.
- **Crates touched:** `pod-io` (CRD/CPF), `pod-observations` (SLR range model), `pod-qc` (`orbit_compare`, `slr_validation`).
- **Concrete tasks:** CRD/CPF parsers; SLR range model with station, atmosphere, retroreflector; CLI `validate-slr` and `compare-orbits --frame rtn`.
- **Tests:** ST-021 SLR validation, ST-024 GNSS POD + SLR validation E2E.
- **Definition of done:** CRD/CPF fixture produces a residual report linked to a run manifest; RTN/RIC compare artifact validated against synthetic truth.
- **Risks:** atmosphere model accuracy — start with Mendes-Pavlis; document residual-bias expectations.

### M6 — NRT replay (EKF/smoothing) (MVP-3)

- **Goal:** EKF replay sharing state/parameter definitions with batch; windowed processing; latency metric.
- **Crates touched:** `pod-estimation::sequential`.
- **Concrete tasks:** EKF time/measurement update reusing `ForceModel`/`MeasurementModel`; process-noise model; PSD covariance check; backward smoother; window driver in `pod-service::stages::estimate`.
- **Tests:** ST-022 EKF replay vs batch within tolerance.
- **Definition of done:** stable EKF on the M4 synthetic arc; QC report contains innovation-NIS time series.

### M7 — Productization

- **Goal:** Python bindings (PyO3), REST job API, container image, public-data benchmark corpus, performance dashboards.
- **Crates touched:** new `crates/spod-py/`, `crates/spod-rest/`; `pod-service` worker model.
- **Concrete tasks:** PyO3 wrappers calling the same library APIs; `axum` REST over `pod-service::Runner`; OCI image with deterministic build; public LEO GNSS arc benchmark.
- **Definition of done:** API parity test (CLI ≡ Rust API ≡ Python ≡ REST) green; documented commercial-readiness checklist.

---

## 4. MVP-1 plan: GNSS-only LEO batch POD

### 4.1 Command

```bash
cargo run -p spod -- run examples/configs/leo_gnss_mvp1.yaml
```

### 4.2 Required inputs (per config)

| Input | Format | Source for fixture |
|---|---|---|
| GNSS observations | RINEX OBS (GPS L1/L2 + Galileo E1/E5a) | Simulated (M4) → public IGS later |
| GNSS broadcast NAV | RINEX NAV | Simulated → public BRDC |
| Precise GNSS products | SP3 + (optional) Clock RINEX | IGS final fixture |
| Antenna phase center | ANTEX | IGS `igs14.atx` snapshot |
| EOP | IERS finals2000A bulletin (subset) | Frozen fixture |
| Leap seconds | `leap-seconds.list` (IERS) | Frozen fixture |
| Spacecraft config | YAML (mass, area, Cd, Cr, antenna offsets) | `examples/fixtures/spacecraft/demo-leo.yaml` |
| Gravity field | EGM2008-style `.gfc` (truncated to deg/order ≤ 20 for MVP-1) | Public fixture (truncated copy) |

All inputs are SHA-256-hashed and recorded in `run.manifest.json`.

### 4.3 Required models

- Time/frame: `tempoch` GPS↔UTC↔TAI↔TT↔UT1↔TDB; `siderust` precession/nutation/polar motion via `EarthOrientationProvider`; ITRF↔GCRF via `FrameTransformProvider`.
- Dynamics: two-body, J2, low-deg/order spherical harmonics (≤ 20 from EGM file), Sun + Moon third-body via `EphemerisProvider` (VSOP87 + ELP2000 backend), simple drag with constant density (`AtmosphereDensityProvider::Constant`), cannonball SRP. Optional relativistic correction.
- Observations: GNSS code + carrier; receiver clock as parameter; carrier float ambiguities per arc per signal; Sagnac correction; relativistic correction; antenna phase-center via ANTEX; satellite clock from broadcast NAV (or precise clock if provided).
- Estimation: nonlinear WLS, sigma-edit robust weighting, max 8 iterations, Cholesky on normal equations with condition diagnostic, faer dense backend.
- Outputs: SP3 (own orbit), OEM (CCSDS), residuals CSV, `qc.json`, `run.manifest.json`, optional `report.html`.

### 4.4 Acceptance gates (mirrors design §18)

1. Run completes with exit code 0 and produces all 6 required artifacts.
2. SP3 output reparses with zero diagnostics.
3. OEM output reparses with zero diagnostics.
4. Residual CSV contains both prefit and postfit columns.
5. QC JSON validates against `docs/validation/qc.schema.json`.
6. Estimated synthetic orbit position error ≤ configured threshold (initial target: 5 cm 3D RMS on 24 h synthetic arc; tightened later).
7. Re-run produces identical canonical JSON manifest hash and identical SP3/OEM byte content (modulo timestamps that are recorded as parameters).
8. No POD crate imports a non-`pub` item from `siderust`/`qtty`/`tempoch`/`affn`/`cheby`.
9. Compile-fail tests cover frame/center/unit misuse on the public POD APIs.

---

## 5. Testing and validation plan

### 5.1 Test categories

| Category | Location | What it covers |
|---|---|---|
| Unit | per-crate `src/**/tests` and `tests/` | Numerical kernels, parsers (small), corrections, partials. |
| Compile-fail (`trybuild`) | `crates/siderust-pod-core/tests/compile_fail/` | Adding two `Position`s; using ITRF position where GCRF is required; computing residual without center binding; mixing scalar units. |
| Parser round-trip | `crates/siderust-pod-io/tests/roundtrip/` | SP3, OEM, ANTEX, RINEX MVP. |
| Synthetic truth | `tests/synthetic/` | Simulate orbit + observations → estimate → assert recovered parameters within tolerance. |
| End-to-end | `tests/e2e/` | Run the CLI against a fixture config; diff artifacts against committed expected outputs. |
| Regression | `tests/regression/` | Frozen fixtures of public-data parses; pinned outputs. |
| Performance | `benches/` (criterion) | Per the §6 benchmark targets. |
| Reproducibility | `tests/e2e/reproducibility.rs` | Two consecutive runs produce identical manifest hash. |
| API parity | `tests/e2e/parity.rs` | CLI ≡ Rust API; from M7: + Python + REST. |

### 5.2 Public-data benchmark strategy

- **MVP-1:** synthetic only (deterministic, fully reproducible, no licensing risk).
- **MVP-2:** add one frozen, redistributable LEO GNSS arc + IGS final products (24 h) committed under `examples/fixtures/public/` with explicit license/source notes.
- **MVP-3+:** publish a separate `spod-benchmarks` repo with curated multi-day public arcs and CI-published results dashboards.

### 5.3 Minimum tolerances (initial)

| Quantity | Tolerance |
|---|---|
| Two-body energy drift over 1 orbit (RK4, fixed-step 30 s) | < 1e-9 relative |
| Two-body STM finite-diff vs analytic | < 1e-7 relative |
| Cartesian↔RTN covariance round-trip Frobenius error | < 1e-12 |
| Synthetic LEO 24 h orbit recovery (MVP-1) | ≤ 5 cm 3D RMS |
| Reproducibility: canonical manifest JSON | byte-identical |

---

## 6. Performance plan

### 6.1 Likely hot paths

1. **RINEX OBS parsing** at IGS network scale (1 Hz × tens of stations × tens of satellites).
2. **Spherical-harmonics gravity** evaluation (degree 70+ later).
3. **Force-model + STM RHS** inside the integrator inner loop.
4. **Design-matrix assembly** (n_obs × n_params, possibly 1e5 × 1e3+ later).
5. **Normal-equation Cholesky / SVD** during WLS.
6. **Residual evaluation** at every observation epoch.

### 6.2 Data structures and memory layout

- States as `repr(C)` POD structs with `Vector3<S>` from `affn` (which itself is `repr(C)`).
- Observation batches stored in **struct-of-arrays** (epoch[], sat_id[], obs_value[], sigma[]) to vectorize residual loops.
- Design matrix backed by faer dense storage; sparse upgrade path through faer's sparse module without API change.
- Avoid allocations in inner loops: `SmallVec` for per-step accumulators, pre-sized scratch buffers in `BatchEstimator`.

### 6.3 Static vs dynamic dispatch

- **Static/generic** for inner loops (force model composition, integrator step, residual evaluation) — composed via traits with generic `S = f64` scalar, monomorphized.
- **Dynamic** (`dyn ForceModel`, `dyn MeasurementModel`) for *configuration assembly* in `pod-service`. Convert dyn → generic at the entry of compute kernels (one boxed call per arc/iteration, not per step).
- Document this in `docs/architecture/dispatch.md`.

### 6.4 Profiling and parallelism

- `cargo flamegraph` and `criterion` per benchmark target.
- Parallelize: per-arc estimation (rayon), per-station residual evaluation (rayon `par_iter`), gravity-degree loops only after profiling shows them dominant.
- Pin CI bench machine; track regression with `criterion-cmp` artifacts.

### 6.5 Benchmark targets (MVP-1 acceptance)

| ID | Target |
|---|---|
| PB-001 SP3 parse | ≥ 50 MB/s on CI machine |
| PB-002 RINEX OBS parse | ≥ 200 k observations/s |
| PB-003 24 h LEO propagation (MVP forces) | ≤ 250 ms wall (DOP853, 30 s output) |
| PB-004 GNSS residual eval | ≥ 500 k residuals/s |
| PB-005 Batch LSQ (24 h synthetic) | ≤ 3 s wall, ≤ 200 MB |
| PB-006 QC generation | ≥ 1 M residual records/s |

Baselines locked in `benches/baseline.json`; CI fails on > 15% regression.

---

## 7. Maintainability plan

### 7.1 Documentation

- `docs/architecture/` — boundaries, dependency rules, providers, dispatch, dynamics, estimation.
- `docs/conventions/` — error handling, naming, units, frames, time-scale handling, JSON canonicalization.
- `docs/requirements/` — HR-/LR-/ST- IDs from design doc with status table per release.
- `docs/formats/` — per-format compliance notes (which RINEX subset, which SP3 line types, etc.).
- `docs/validation/` — JSON-Schemas and validation reports.
- `docs/adrs/` — Architecture Decision Records (ADR-0001 = workspace reset, ADR-0002 = faer backend, ADR-0003 = AGPL-3.0, ADR-0004 = provider-trait pattern, …).
- `cargo doc` published in CI for every PR; broken intra-doc links fail CI.

### 7.2 API stability

- Pre-1.0 (MVP-1..MVP-3): `0.x` versions; SemVer-minor bumps may break.
- 1.0 freeze candidates: `pod-core` types and provider traits (the most depended-upon surface). Mark internal items `#[doc(hidden)]` and `pub(crate)`.
- `cargo public-api` snapshot per crate, diffed in CI.

### 7.3 Error handling

- One concrete `Error` enum per crate, built with `thiserror`.
- `pod-core::Error` is the umbrella type; downstream crates re-export their own and convert via `#[from]`.
- All errors carry: dataset id (where relevant), epoch (where relevant), structured machine-readable code.
- Public API never panics on user input; panics are reserved for invariant violations and are documented.

### 7.4 Feature flags

| Crate | Flags |
|---|---|
| `pod-core` | `serde` (default), `schemars` (JSON-Schema export). |
| `pod-io` | `parquet` (off), `compression` (off). |
| `pod-estimation` | `faer-sparse` (off until M5+), `rayon` (default). |
| `pod-qc` | `html` (default), `plotters` (off until M5). |
| `pod-service` | `rest` (off until M7). |
| `pod-cli` | `color` (default). |

No feature gates change *numerical results*. CI matrix tests `--no-default-features` and `--all-features` per crate.

### 7.5 CI gates

- `cargo fmt -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features` (for crates that support it)
- `scripts/check_dep_graph.sh`
- `cargo deny check` (advisory + license + bans)
- `cargo public-api --diff` (warn → fail post-1.0)
- `cargo doc --no-deps -D rustdoc::broken-intra-doc-links`
- Reproducibility test in CI matrix.
- Performance bench job on a pinned runner; warn on > 15% regression.

### 7.6 Dependency audit

- `cargo deny`: deny `gpl-2.0`-only, deny duplicate dependency major versions across the workspace, allow only `apache-2.0`, `mit`, `bsd-3-clause`, `mpl-2.0`, `agpl-3.0`, `unicode-dfs-2016`.
- Lockfile committed.

### 7.7 Coding conventions

- All public types parameterized by `S: Scalar` where useful (default `f64`).
- All public APIs accept `qtty` and `tempoch` types; raw `f64`/`u64` only in serialization and parsers.
- All public functions have rustdoc + an `# Example` or a link to `examples/`.
- No `unsafe` outside vetted FFI shims; `#![forbid(unsafe_code)]` per crate where possible.
- Conventional Commits on every commit (`<type>(<scope>): <summary>`); never include Copilot co-author trailers.

### 7.8 Release / versioning

- Independent SemVer per POD crate; release together via `cargo-release` with workspace-version bumping but separate changelogs.
- `CHANGELOG.md` per crate, Keep-a-Changelog format.
- Pre-1.0: weekly snapshot tag; 1.0 announced after MVP-1 + at least one external user.

---

## 8. Risk register

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R-01 | Scope creep into full FocusPOD parity before MVP-1 ships | High | High | M0–M4 enforce a strict synthetic-only first product; design doc §2.2 non-goals quoted in PR template. |
| R-02 | Pressure to modify `qtty`/`tempoch`/`affn`/`cheby`/`siderust` for short-term POD convenience | High | High | Provider-trait + adapter-first rule; any upstream change requires separate branch + ADR + maintainer approval; CI denies path-dependencies on a feature branch. |
| R-03 | Estimator coupling to GNSS specifics | Medium | High | `MeasurementModel` trait owns the seam; add a non-GNSS dummy (range-rate) impl in M3 to prove generality. |
| R-04 | Non-determinism (HashMap iter, float formatting, parallel reductions) | High | High | Canonical JSON encoder; sorted IDs everywhere; deterministic-reduction option; reproducibility test in CI. |
| R-05 | Covariance semantics drift (frame/center/order ambiguity) | Medium | High | `Covariance` carries frame, center, ordering, and units in its type; round-trip + PSD tests are mandatory gates. |
| R-06 | Performance regressions go unnoticed | Medium | Medium | Pinned-runner bench job with baselines + 15% regression gate. |
| R-07 | Vendored siderust copy creeps back into `spod` | Low | High | M0 deletes it; `scripts/check_dep_graph.sh` rejects any `path = "../../rust/siderust"` not whitelisted; CI scans for `src/astro/`, `src/calculus/` etc. inside `crates/`. |
| R-08 | Float-only ambiguity insufficient for users | Medium | Medium | Document explicitly; carve a clean place in `pod-estimation::ambiguity` for integer fixing in MVP-2+. |
| R-09 | Public-data licensing surprises | Medium | Medium | MVP-1 synthetic-only; vetted public fixtures with explicit `LICENSE` and source URL files in `examples/fixtures/public/`. |
| R-10 | RINEX/SP3/ANTEX edge-case interoperability bugs | High | Medium | Strict + permissive modes; structured diagnostics; expand fixture set per bug report; commit golden fixtures. |
| R-11 | Service/CLI logic leaks into compute crates | Medium | High | Forbidden-edge CI; PR template asks "does this introduce config/IO/CLI knowledge into a compute crate?". |
| R-12 | Provider-trait churn breaks downstream | Medium | Medium | Mark `pod-core::providers` semver-stable target; require ADR for any breaking change. |
| R-13 | AGPL chills commercial adoption | Medium | Medium | Document open-source posture; evaluate dual-licensing (AGPL + commercial) only at M7 productization, with a dedicated ADR. |

---

## 9. Branching and change policy

- `rust/siderust/`, `rust/qtty/`, `rust/tempoch/`, `rust/affn/`, `rust/cheby/`, `rust/NSB/` — **untouched** by the POD plan. CI on the POD branch fails if any file under those paths changes.
- `rust/spod/` — the **only** editable workspace for this plan. All POD work lands here.
- Any change required in a foundational crate must:
  1. Be created on a **separate branch** off `main`, named `foundational/<crate>/<reason>`.
  2. Carry an **ADR** under `rust/spod/docs/adrs/` justifying why an adapter/provider trait was insufficient.
  3. Be reviewed by a maintainer of the affected crate.
  4. Be merged independently of POD work; POD then bumps the upstream version.
- Working branch convention for POD: `pod/<milestone>/<short-topic>`, e.g. `pod/m2/sp3-writer`.
- PR template includes:
  - "No upstream foundational crate modified" checkbox.
  - "No private upstream API imported" checkbox.
  - "No Copilot co-author attribution" checkbox.
  - Forbidden-edges checkbox (auto-checked by CI).

---

## 10. First actionable tasks (ordered)

Each task: target crate(s), expected files, acceptance criteria.

1. **Reset workspace.** `spod`. Delete legacy `src/`, `build.rs`, vendored `siderust-ffi/`, upstream `examples/`/`benches/`/`tests/`, `archive/` data. Replace root `Cargo.toml` with virtual workspace. Move `memory/` → `docs/design/`. *Acceptance:* `cargo metadata` lists no members yet; `git status` is clean after re-add.
2. **Create empty crates.** `crates/spod-{core,dynamics,io,observations,estimation,qc,products,service,cli}` via `cargo new`, AGPL-3.0 in each `Cargo.toml`. *Acceptance:* `cargo build --workspace` succeeds.
3. **Wire workspace deps.** Pin `siderust = "0.7"`, `qtty = "0.7"`, `tempoch = "0.4"`, `affn = "0.7"`, `cheby = "0.2"`, `faer`, `serde`, `thiserror`, `clap`. *Acceptance:* `cargo deny check` passes.
4. **CI skeleton.** `.github/workflows/ci.yml` with fmt/clippy/test/doc/dep-graph/deny. *Acceptance:* CI green on empty workspace.
5. **Dep-graph guard.** `scripts/check_dep_graph.sh` + unit test. *Acceptance:* deliberate forbidden edge fails CI in a draft PR.
6. **ADR-0001..0004.** Workspace reset, faer backend, AGPL-3.0, provider-trait pattern. *Acceptance:* committed under `docs/adrs/`.
7. **Boundaries doc.** `docs/architecture/{boundaries.md,dependency-rules.md,providers.md,dispatch.md}`. *Acceptance:* reviewed.
8. **Provider traits.** `pod-core::providers::{ephemeris,earth_orientation,frame_transform,gravity_field,atmosphere_density}.rs` with default impls wrapping public `siderust`. *Acceptance:* unit tests prove a Sun-position call returns the same value as direct `siderust` call.
9. **Domain primitives.** `pod-core::{state,arc,spacecraft,parameter,covariance,manifest,dataset,error}.rs`. *Acceptance:* unit tests build `OrbitState`, `SpacecraftState`, `RunManifest` instances.
10. **POD frames.** `pod-core::frames::{rtn,lvlh,vnc}.rs` defining marker types and `affn` impls. *Acceptance:* type-level test that `Position<RTN, ...>` cannot be added to `Position<GCRF, ...>`.
11. **Compile-fail harness.** `pod-core/tests/compile_fail/*.rs` via `trybuild`. *Acceptance:* 5 representative misuses fail to compile.
12. **Two-body force.** `pod-dynamics::forces::two_body`. *Acceptance:* ST-005 energy conservation passes.
13. **J2 force.** `pod-dynamics::forces::j2`. *Acceptance:* ST-006 reference vector passes.
14. **RK4 integrator.** `pod-dynamics::integrators::rk4`. *Acceptance:* two-body 1-orbit error bound met.
15. **DOP853 integrator.** `pod-dynamics::integrators::dop853`. *Acceptance:* 24 h LEO propagation finishes within bench target PB-003.
16. **Third-body force.** `pod-dynamics::forces::third_body` calling `EphemerisProvider`. *Acceptance:* ST-007 passes.
17. **Cartesian↔RTN covariance.** `pod-core::covariance` + `pod-dynamics::frames::rtn`. *Acceptance:* ST-003 passes.
18. **STM (two-body).** `pod-dynamics::variational::stm`. *Acceptance:* finite-difference Jacobian agreement < 1e-7.
19. **SP3 reader.** `pod-io::sp3::parser`. *Acceptance:* IGS sample file parses with no diagnostics.
20. **SP3 writer + round-trip test.** `pod-io::sp3::writer`. *Acceptance:* ST-010 passes.
21. **RINEX OBS parser (MVP subset).** `pod-io::rinex::obs`. *Acceptance:* ST-011 passes.
22. **RINEX NAV parser (MVP subset).** `pod-io::rinex::nav`. *Acceptance:* broadcast ephemeris evaluates within tolerance vs SP3.
23. **ANTEX parser.** `pod-io::antex`. *Acceptance:* ST-012 passes.
24. **EOP adapter.** `pod-io::eop` producing a `tempoch::eop::EopDataset` (no upstream change). *Acceptance:* loaded EOP reproduces a known UT1-UTC value.
25. **OEM writer.** `pod-io::ccsds::oem`. *Acceptance:* OEM round-trip via a third-party reader (e.g. CCSDS-validated fixture).
26. **MeasurementModel trait.** `pod-observations::model`. *Acceptance:* trait compiles; doc-tested with a dummy range model.
27. **GNSS code prediction + partials.** `pod-observations::gnss::code`. *Acceptance:* ST-013 passes; finite-diff Jacobian < 1e-6.
28. **GNSS carrier prediction + float ambiguity + partials.** `pod-observations::gnss::carrier`. *Acceptance:* ST-014 passes.
29. **Corrections (Sagnac, relativity, antenna PCO/PCV, phase wind-up).** `pod-observations::corrections::*`. *Acceptance:* ST-015 individual correction tests pass.
30. **WLS solver (faer).** `pod-estimation::batch::weighted_least_squares` + `normal_equations` + `design_matrix`. *Acceptance:* ST-016 linear solve recovers parameters.
31. **Nonlinear iteration + convergence + sigma-edit.** `pod-estimation::batch::{nonlinear,convergence}` + `pod-estimation::robust`. *Acceptance:* ST-017 nonlinear synthetic OD recovers truth.
32. **Covariance extraction & condition diagnostics.** `pod-estimation::covariance`. *Acceptance:* ST-018 fields present.
33. **Synthetic GNSS arc generator.** `tests/synthetic/leo_gnss_24h.rs`. *Acceptance:* generates SP3 + RINEX OBS reproducibly from a seed.
34. **Config schema + loader + validator.** `pod-service::config` + `pod-cli validate-config`. *Acceptance:* `validate-config examples/configs/leo_gnss_mvp1.yaml` succeeds; bad config produces structured error.
35. **Pipeline runner (15 stages).** `pod-service::{pipeline,runner,artifacts,manifest}` + `stages/*`. *Acceptance:* `run` produces all 6 artifacts on the synthetic fixture.
36. **Product writers + naming + manifest hashing.** `pod-products::{orbit::{sp3,oem},residuals,manifest,naming}`. *Acceptance:* product validation passes; manifest contains input + output hashes.
37. **QC residuals + grouping + JSON-Schema + summary.** `pod-qc::{residuals,schema}`. *Acceptance:* `qc.json` validates against committed schema.
38. **HTML report (minimal templated).** `pod-qc::html`. *Acceptance:* `report.html` opens; embeds residual statistics and run metadata.
39. **CLI commands `run` / `inspect-manifest` / `qc`.** `pod-cli`. *Acceptance:* end-to-end ST-023 passes from `cargo run`.
40. **Reproducibility + parity tests.** `tests/e2e/{reproducibility,parity}.rs`. *Acceptance:* ST-020 + ST-025 pass; CI gates on them.

---

## 11. Open decisions still needing human input

| ID | Decision | Default in this plan | Trigger to revisit |
|---|---|---|---|
| ODD-A | Public benchmark satellite for MVP-2 | Sentinel-class LEO public arc with redistributable products | Before M5 starts |
| ODD-B | Parquet adoption for residuals | CSV first, Parquet behind feature | If a real user needs Parquet for scale |
| ODD-C | Python bindings timing | After M4 (per design ODD-006) | If a paying user requires earlier |
| ODD-D | Integer ambiguity resolution | Deferred past MVP-1 | Customer requirement or research collaboration |
| ODD-E | Dual-licensing posture | AGPL-3.0 single license | M7 commercialization push |
| ODD-F | High-degree gravity strategy | EGM2008 truncated to deg/order ≤ 20 in MVP-1; full degree in M5+ | Performance bench shows acceptable cost |
| ODD-G | Atmosphere model beyond constant | NRLMSISE-00 wrapper (or DTM) added in M5 | SLR validation residuals demand it |

---

## 12. Locked decisions

- Workspace will be **wiped and rebuilt** as a virtual workspace; legacy cloned `siderust` code under `spod/src/` will be deleted.
- Linear-algebra backend for the estimator is **`faer`**; `affn` continues to own typed positions/vectors and 3×3 covariance transforms.
- License is **AGPL-3.0** (matches `siderust`).
- Foundational crates (`qtty`, `tempoch`, `affn`, `cheby`, `siderust`) are **read-only** to this plan; any change requires a separate branch and ADR.
- No Copilot co-author attribution anywhere.
