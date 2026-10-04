# Changelog

All notable changes to this repository are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Renamed the project and Cargo package to `spod`, including the `spod` and
  `spod-rest` binaries, repository metadata, Docker examples, and REST
  configuration variables.
- Consolidated `spod` into a service/application built on the canonical
  `siderust 0.12` POD APIs instead of maintaining a second POD scientific
  implementation.
- The synthetic POD pipeline now uses
  `siderust::pod::propagation::VariationalPropagator` for aligned per-epoch
  states and state-transition matrices.
- CLI runs hash the executed YAML configuration with Siderust `DatasetRef`;
  REST jobs hash a deterministic serialized request representation.
- The service now rejects every real-data input while real-data ingestion is
  not implemented, preventing unused files from being recorded as scientific
  provenance.
- Reusable scientific types and low-level SPICE parser APIs are imported
  directly from Siderust rather than re-exported through compatibility
  façades.
- Raised the MSRV to Rust 1.89 to match the released Siderust dependency graph.

### Fixed

- Run manifests now contain a real configuration SHA-256, canonical hashed
  dataset references, and RFC3339 start/finish timestamps.
- Removed unsupported synthetic carrier-ambiguity injection from the MVP,
  whose estimated parameters are the six state components plus receiver
  clock.
- Synthetic state/STM indexing explicitly maps epoch zero to the identity STM
  and subsequent epochs to the corresponding cumulative variational STM.
- Runtime diagnostics no longer reference deleted implementation-plan
  documents.

### Added

- End-to-end synthetic service coverage from `RunConfig` through Siderust
  propagation/estimation to orbit, residual, QC, and run-manifest artifacts.
- Regression coverage ensuring every unsupported real-data input is rejected
  rather than hashed and ignored.
- CI guards for formatting, Clippy, feature combinations, documentation,
  dependency direction, legacy project namespaces, MSRV, coverage, Docker,
  REST/CLI smoke tests, and supply-chain policy.

### Removed

- Duplicated local POD scientific modules for dynamics, observations,
  estimation, products, quality control, parameters, covariance, and run
  metadata.
- Deprecated finite-difference STM compatibility code and its deprecation
  allowance.
- Deprecated pre-rename REST environment aliases.
- Obsolete compatibility reexports, stale architecture documents, and
  dependencies used only by the removed scientific implementation.
