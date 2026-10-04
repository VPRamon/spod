# Testing architecture

`spod` tests the product and service boundary. The primary integration path
is the synthetic workflow in
[`tests/service_synthetic.rs`](../../tests/service_synthetic.rs): it loads a
configuration, validates and dispatches it through `Runner`, writes the
service artifact set, and verifies manifest provenance, hashes, and sizes.
It intentionally checks application contracts rather than reproducing
scientific reference values owned by Siderust.

## Test ownership

| Layer | Owned by | Examples |
| --- | --- | --- |
| Service integration | `spod` | configuration validation, workflow dispatch, job lifecycle, artifacts, manifests, provenance |
| Interface integration | `spod` | compiled CLI tests, in-process REST router tests, process and Docker startup smoke tests |
| Scientific unit tests | Siderust | force-model mathematics, propagation numerics, estimation kernels, generic observations, standard formats, and generic QC |
| Product parity | `spod` temporarily | Full Vallado SGP4/SDP4 compatibility required by the product |

The CLI and REST adapters use the same `JobStatus` and `JobError` model.
Detailed behavior belongs in fast in-process tests; shell and Docker checks
only cover process, bind-address, image, and health-boundary behavior.

## Architecture guardrails

`scripts/check_architecture.sh` rejects newly declared local generic
scientific implementation types outside the service boundary. It checks
implementation traits/types rather than filenames, so legitimate orchestration
and upstream Siderust type names remain usable. `scripts/check_dep_graph.sh`
also rejects path and Git dependencies so scientific APIs resolve from the
canonical locked dependency graph.

The local `src/sgp4/` implementation is an explicit exception. Siderust 0.12
does not yet provide equivalent full Vallado/AFSPC semantics, so
[`tests/sgp4_vallado.rs`](../../tests/sgp4_vallado.rs) remains the product
parity safety net for near-Earth, deep-space, and propagation-error cases.
The exception is tracked by [VPRamon/spod#29](https://github.com/VPRamon/spod/issues/29)
and [Siderust/siderust#98](https://github.com/Siderust/siderust/issues/98).

When adding a service workflow, add coverage for its configuration contract,
dispatch and lifecycle behavior, generated artifacts, and provenance. Add
scientific correctness tests upstream instead of copying Siderust's unit
tests into this repository.
