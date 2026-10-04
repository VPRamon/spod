# Dispatch boundary

`spod::service` performs configuration validation, workflow selection, and
artifact/provenance orchestration. Scientific execution is delegated to the
canonical Siderust APIs; `spod` does not define local generic force,
measurement, estimation, or propagation kernels.

The source-hygiene guard in
[`scripts/check_architecture.sh`](../../scripts/check_architecture.sh) checks
for newly declared local scientific implementation traits and types while
allowing service code to compose upstream names. The only current local
scientific exception is `src/sgp4/`, retained for full Vallado/AFSPC parity
until Siderust provides equivalent semantics.
