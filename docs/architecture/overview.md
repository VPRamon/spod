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

The crate consumes `qtty`, `tempoch`, `affn`, `cheby`, `principia`, and
`siderust` from crates.io. Siderust owns foundational astronomy and
astrodynamics. Principia owns reusable numerical mechanics. This repository
adds POD-specific composition and applications; it does not vendor or patch
those upstream crates.

```mermaid
flowchart TD
    QTTY[qtty] --> POD[spod]
    TEMPOCH[tempoch] --> POD
    AFFN[affn] --> POD
    CHEBY[cheby] --> POD
    PRINCIPIA[principia] --> POD
    SIDERUST[siderust] --> POD

    POD --> SVC[service / CLI / REST]
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
