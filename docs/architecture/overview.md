# Architecture Overview

`spod` is an application/service built on Siderust's canonical POD APIs.
Its public boundary is the service runner, which owns configuration,
workflow dispatch, job/run orchestration, input validation, artifacts,
and provenance;
reusable dynamics, observations, estimation, QC, products,
and run metadata remain in `siderust::pod`.

The current `synthetic` workflow is the deterministic reference/integration
workflow. Real-data inputs are rejected explicitly until a real-data workflow
is introduced. Full Vallado SGP4/SDP4 remains a temporary local exception
while semantic parity is tracked by spod #29 and Siderust #98.

## Dependency structure

The crate directly depends on `siderust` for reusable POD/scientific
functionality and on `sgp4` for the temporary full Vallado compatibility
exception tracked by spod #29 and Siderust #98. Runtime dependencies support
the service, CLI, and REST surfaces. Lower-level ecosystem crates such as
`qtty`, `tempoch`, `affn`, `cheby`, and `principia` may be transitive
dependencies of Siderust, but are not direct architectural dependencies of
`spod`.

```mermaid
flowchart TD
    SIDERUST[siderust 0.12] --> SPOD[spod service/application]
    SGP4[sgp4 temporary exception] --> SPOD
    RUNTIME[service / CLI / REST dependencies] --> SPOD
    SPOD --> SVC[Runner / CLI / REST]
    SIDERUST --> SCIENCE[siderust::pod]
    SCIENCE --> SVC
```

The dependency check in `scripts/check_dep_graph.sh` rejects path and git
dependencies and verifies the locked graph can be resolved from the
standalone checkout.

## Data flow

A POD run is a deterministic, manifest-tracked transformation from raw
observations and supporting products into an estimated orbit and quality
artefacts.

```mermaid
flowchart LR
    RAW[Raw observations and auxiliary data] --> SVC[Service orchestration]
    SVC --> POD[siderust::pod execution]
    POD --> OUT[Service artifacts and manifest]
```

The CLI and REST binaries are thin front ends over `spod::service::Runner`.
They do not assemble pipelines or call scientific components directly.
Siderust's scientific APIs remain independently testable, while the service
boundary is covered by synthetic end-to-end execution.

## Shared job model

`spod::service` owns the canonical `JobId`, `JobState`, `JobStatus`,
`JobResult`, and `JobError` contracts. The lifecycle is `pending -> running ->
succeeded` or `pending -> running -> failed`; terminal statuses carry either
artifact references or a stable machine-readable error category. REST stores
these snapshots in an in-memory `JobStore` and exposes them as JSON. The CLI
uses the same model synchronously and formats its terminal status for
operators.

The REST adapter is intentionally asynchronous and experimental. It does not
provide durable persistence, scheduling, retries, or distributed workers.
Those concerns are deferred to later service/API work, including the stable
artifact API tracked by #28.
