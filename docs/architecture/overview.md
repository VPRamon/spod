# Architecture Overview

`spod` is a single-crate precise-orbit-determination toolkit. Its
modules separate domain primitives, dynamics, observations, estimation,
formats, products, quality control, and application orchestration while
sharing one released dependency graph.

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

    POD --> CORE[core]
    POD --> DYN[dynamics]
    POD --> IO[io / formats]
    POD --> OBS[observations]
    POD --> EST[estimation]
    POD --> PROD[products]
    POD --> QC[quality control]
    POD --> SVC[service / CLI / REST]
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
    RAW[Raw observations and auxiliary data] --> IO[Parsing and providers]
    IO --> OBS[Measurement models and corrections]
    IO --> DYN[Force models and propagation]
    OBS --> EST[WLS / Gauss-Newton / EKF]
    DYN --> EST
    EST --> QC[Residual and orbit checks]
    EST --> PROD[SP3 / OEM / residual products]
    QC --> PROD
    PROD --> OUT[Run artefacts and manifest]
```

The CLI and REST binaries are thin front ends over the service module. The
scientific layers remain independently testable, with typed quantities,
time scales, centers, and frames preserved at their boundaries.
