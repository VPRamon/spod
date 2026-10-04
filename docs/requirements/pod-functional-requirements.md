# POD Functional Requirements

This document tracks the functional requirements of `spod` against
the M0–M7 milestone structure defined in `docs/design/plan.md`.

Status legend: **Done**, **In progress**, **Planned**, **Deferred**.

## Group: M-level milestone summary

| ID         | Milestone               | Requirement                                                                                       | Status        |
|------------|-------------------------|---------------------------------------------------------------------------------------------------|---------------|
| FR-M0      | Workspace bootstrap     | `spod` compiles cleanly; upstream crates (`siderust`, `affn`, `qtty`, `tempoch`) are the canonical source for general astrodynamics. | Done          |
| FR-M1-01   | GNSS LEO batch          | Ingest SP3, RINEX OBS/NAV, ANTEX, EOP fixtures end-to-end.                                        | In progress   |
| FR-M1-02   | GNSS LEO batch          | Batch WLS with float carrier ambiguities resolves a 24 h LEO arc.                                 | In progress   |
| FR-M1-03   | GNSS LEO batch          | SP3 + OEM + manifest products written reproducibly.                                               | In progress   |
| FR-M2-01   | EKF replay              | EKF replays the same arc as the batch and stays within 3·σ of the batch state at every epoch.     | Planned       |
| FR-M3-01   | SLR validation          | CRD/CPF ingestion, SLR forward model, RTN residual stats within Mendes-Pavlis tolerance.          | Planned       |
| FR-M4-01   | Multi-arc / overlap     | Two consecutive arcs overlap; orbit-overlap RMS reported.                                         | Planned       |
| FR-M5-01   | Service & CLI parity    | CLI ≡ Rust API ≡ REST produce byte-identical artefacts on the same config.                       | Planned       |
| FR-M6-01   | LISA POC                | Three-spacecraft heliocentric POC with simulated MOSA inter-spacecraft ranges.                    | Planned       |
| FR-M7-01   | Hardening for 1.0       | Public-API snapshot gate flips from advisory to gating.                                           | Planned       |

The detailed requirements below are grouped by domain area; each maps to
one or more milestones above.

---

## Group: IO formats — `FR-IO-*`

| ID        | Requirement                                                                                              | Crate                | Status        |
|-----------|----------------------------------------------------------------------------------------------------------|----------------------|---------------|
| FR-IO-01  | Read SP3-d files (header + epoch + P-records).                                                           | `spod::io`    | In progress   |
| FR-IO-02  | Write SP3-d files preserving header verbatim where possible.                                             | `spod::io`    | In progress   |
| FR-IO-03  | Read RINEX 3 OBS files (GPS L1/L2 + Galileo E1/E5a subset).                                              | `spod::io`    | In progress   |
| FR-IO-04  | Read RINEX 3 NAV (GPS broadcast ephemeris).                                                              | `spod::io`    | In progress   |
| FR-IO-05  | Read ANTEX 1.4 PCO values (PCV grids skipped).                                                           | `spod::io`    | In progress   |
| FR-IO-06  | Read IERS EOP C04 series (daily rows + linear interpolation).                                            | `spod::io`    | In progress   |
| FR-IO-07  | Read & write CCSDS OEM (KVN, single-segment write, multi-segment read).                                  | `spod::io`    | In progress   |
| FR-IO-08  | Read ILRS CRD (record types 10 and 11).                                                                  | `spod::io`    | Planned (M3)  |
| FR-IO-09  | Read ILRS CPF (record type 10).                                                                          | `spod::io`    | Planned (M3)  |
| FR-IO-10  | Read TLE / 3LE / OMM (KVN, XML, JSON).                                                                   | `siderust::astro::satellite::tle` | Done          |
| FR-IO-11  | Read DAF/SPK Type 2 and Type 3 segments.                                                                 | `siderust::data::spk`     | Done          |
| FR-IO-12  | Read BLQ ocean-loading coefficients.                                                                     | `spod::io`    | Deferred (Ph5)|

## Group: Force / dynamics models — `FR-DYN-*`

| ID        | Requirement                                                                            | Status      |
|-----------|----------------------------------------------------------------------------------------|-------------|
| FR-DYN-01 | Two-body central gravity.                                                              | Done        |
| FR-DYN-02 | EGM2008 / EGM96 spherical-harmonic gravity field (configurable max degree).            | In progress |
| FR-DYN-03 | Solid Earth + ocean tides (IERS Conventions 2010).                                     | Planned     |
| FR-DYN-04 | Third-body perturbations (Sun, Moon, planets) sourced from `siderust-spice`.           | In progress |
| FR-DYN-05 | Atmospheric drag with NRLMSISE-00 / DTM-2013.                                          | Planned     |
| FR-DYN-06 | Solar radiation pressure: cannonball + box-wing.                                       | In progress |
| FR-DYN-07 | Earth radiation pressure (albedo + IR).                                                | Planned     |
| FR-DYN-08 | Empirical accelerations (constant / once-per-rev RTN).                                 | In progress |
| FR-DYN-09 | Thrust-arc parameterisation with start/stop epochs.                                    | In progress |
| FR-DYN-10 | Variational equations / analytic STM (replaces finite-diff STM, see ADR-0006).         | Planned     |
| FR-DYN-11 | Composite force model with run-time configurable registry.                             | In progress |

## Group: Observation models — `FR-OBS-*`

| ID        | Requirement                                                              | Status          |
|-----------|--------------------------------------------------------------------------|-----------------|
| FR-OBS-01 | GNSS code (pseudorange) measurement model.                               | In progress     |
| FR-OBS-02 | GNSS carrier-phase model with float ambiguities.                         | In progress     |
| FR-OBS-03 | Antenna PCO/PCV correction from ANTEX.                                   | In progress     |
| FR-OBS-04 | Sagnac correction.                                                       | In progress     |
| FR-OBS-05 | Relativistic clock correction.                                           | In progress     |
| FR-OBS-06 | Phase wind-up correction.                                                | In progress     |
| FR-OBS-07 | Tropospheric delay (Saastamoinen + GMF/VMF1 mapping).                    | In progress     |
| FR-OBS-08 | Ionospheric delay (dual-frequency ionosphere-free combination).          | In progress     |
| FR-OBS-09 | SLR range model (centre-of-mass offset, Marini-Murray / Mendes-Pavlis).  | Planned (M3)    |
| FR-OBS-10 | DORIS Doppler measurement model.                                         | Planned (M5)    |
| FR-OBS-11 | VLBI delay model.                                                        | Deferred        |
| FR-OBS-12 | Inter-spacecraft range (LISA POC).                                       | Planned (M6)    |

## Group: Estimation — `FR-EST-*`

| ID        | Requirement                                                                          | Status      |
|-----------|--------------------------------------------------------------------------------------|-------------|
| FR-EST-01 | Batch weighted least squares with `faer` Cholesky / QR backend.                       | In progress |
| FR-EST-02 | Robust weighting (Huber, sigma-edit).                                                | In progress |
| FR-EST-03 | Posterior covariance extraction with parameter labels.                                | In progress |
| FR-EST-04 | Outlier detection and flagging in residual records.                                   | In progress |
| FR-EST-05 | Extended Kalman Filter on the same state vector as the batch.                         | Planned (M2)|
| FR-EST-06 | Process-noise wrappers (RTN / inertial Q matrices).                                   | In progress |
| FR-EST-07 | LAMBDA integer ambiguity resolution.                                                  | Planned (M4)|

## Group: QC and products — `FR-QC-*`

| ID       | Requirement                                                                              | Status        |
|----------|------------------------------------------------------------------------------------------|---------------|
| FR-QC-01 | Per-satellite / per-obs-type / per-elevation residual statistics.                        | In progress   |
| FR-QC-02 | Orbit overlap RMS in RTN/RIC frames.                                                     | Planned (M4)  |
| FR-QC-03 | SLR validation residual report.                                                          | Planned (M3)  |
| FR-QC-04 | JSON QC record + HTML report with inline templates.                                      | In progress   |
| FR-QC-05 | SP3 product writer.                                                                      | In progress   |
| FR-QC-06 | CCSDS OEM product writer.                                                                | In progress   |
| FR-QC-07 | Residual product (CSV, optional Parquet under `parquet` feature).                        | In progress   |
| FR-QC-08 | Canonical JSON manifest with byte-stable hashing.                                        | In progress   |
| FR-QC-09 | Archive layout: predictable per-run directory structure with provenance.                 | In progress   |

## Group: Service, CLI, REST — `FR-SVC-*`

| ID        | Requirement                                                                                  | Status        |
|-----------|----------------------------------------------------------------------------------------------|---------------|
| FR-SVC-01 | YAML `RunConfig` ingestion via `serde_yaml`.                                                 | In progress   |
| FR-SVC-02 | 15-stage pipeline runner with deterministic ordering.                                        | In progress   |
| FR-SVC-03 | `spod` exposes `run`, `validate-config`, `inspect-manifest`, `qc`, `validate-slr`, `compare-orbits`, `lambert`, `propagate`, `lisa-poc`. | In progress |
| FR-SVC-04 | `spod-rest` exposes `POST /jobs`, `GET /jobs/{id}`, `DELETE /jobs/{id}`, `GET /jobs/{id}/products/{file}`. | Planned (M5) |
| FR-SVC-05 | OpenAPI 3.1 schema generated for the REST API.                                               | Planned (M5)  |
| FR-SVC-06 | Bearer-token authentication for REST.                                                        | Planned       |
| FR-SVC-07 | API parity: CLI, library, and REST produce byte-identical artefacts on the same config.      | Planned (M5)  |

## Group: LISA POC — `FR-LISA-*`

See [ADR-0005](../adr/0005-lisa-poc-scope.md) for the scoping of this work.

| ID         | Requirement                                                                              | Status        |
|------------|------------------------------------------------------------------------------------------|---------------|
| FR-LISA-01 | `LisaEphemerisProvider` reads `esa/lisa-orbit-files`.                                    | Planned (M6)  |
| FR-LISA-02 | Three heliocentric ICRS state vectors per epoch (one per spacecraft).                    | Planned (M6)  |
| FR-LISA-03 | `InterSatRange` measurement model in `spod::observations`.                        | Planned (M6)  |
| FR-LISA-04 | Three-spacecraft batch estimation using existing WLS infrastructure.                     | Planned (M6)  |
| FR-LISA-05 | Example `examples/12_lisa_poc.rs` runs end-to-end.                                       | Planned (M6)  |
| FR-LISA-06 | E2E-09 acceptance test passes within ranging-only observability.                         | Planned (M6)  |
