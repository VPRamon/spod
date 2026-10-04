# spod Product Wiki

This wiki defines what `spod` is as a product, who it is for, which workflows it should provide, and how those capabilities relate to Siderust.

> **spod is operational Precise Orbit Determination infrastructure built on Siderust.**

Siderust owns reusable scientific and numerical primitives. `spod` turns those primitives, external datasets, mission configuration, and operational policy into reproducible POD jobs, products, quality reports, and service interfaces.

## North Star

```text
onboard GNSS observations
        +
precise supporting products
        +
mission configuration
        |
        v
      spod
        |
        +-- resolve and validate inputs
        +-- construct the POD problem
        +-- orchestrate Siderust science
        +-- estimate and assess convergence
        +-- apply operational QC policy
        +-- write standard products
        +-- record full provenance
        |
        v
precise orbit + QC + artifacts + manifest
```

The North Star milestone is tracked in [#20](https://github.com/VPRamon/spod/issues/20).

## Product boundary

### Siderust owns

- physical and astrodynamics models;
- propagation and numerical algorithms;
- estimators;
- observation models;
- coordinate/time/unit primitives;
- standard file-format parsers and writers;
- reusable scientific QC primitives;
- reusable product serialization.

### spod owns

- job and workflow definitions;
- mission configuration;
- input resolution and acquisition policy;
- orchestration of Siderust components;
- artifact layout and publication;
- operational QC policy;
- provenance and reproducibility;
- CLI and REST interfaces;
- fleet/reprocessing orchestration;
- observability;
- deployment and operational behavior.

The rule is not `if Siderust implements it, spod stops supporting it`.

> **If Siderust implements the science, spod should consume it instead of duplicating it while preserving the product capability.**

See [Feature parity policy](feature-parity-policy.md).

## Wiki map

- [Product vision and positioning](product-vision.md)
- [Personas and buyers](personas-and-buyers.md)
- [Capability map](capability-map.md)
- [Feature parity policy](feature-parity-policy.md)
- [Commercial model and packaging](commercial-model.md)
- [Commercial/product roadmap](commercial-roadmap.md)
- [Use cases](use-cases/README.md)

## Primary use cases

| Use case | Primary customer | Product value | Tracking |
|---|---|---|---|
| Operational onboard-GNSS POD | LEO/EO operator | Precise orbit products without building a POD stack | [#20](https://github.com/VPRamon/spod/issues/20) |
| Fleet / constellation POD | Constellation operator | Repeatable processing across many spacecraft | [#31](https://github.com/VPRamon/spod/issues/31) |
| Independent orbit validation | Operator / integrator | Independent confidence and quality gate | [#24](https://github.com/VPRamon/spod/issues/24) |
| Mission commissioning | Mission operations | Validate GNSS tracking and POD readiness | [#30](https://github.com/VPRamon/spod/issues/30) |
| Historical reprocessing | Science / operations | Reproduce historical products under new baselines | [#25](https://github.com/VPRamon/spod/issues/25) |
| Multi-latency operations | Operational ground segment | Rapid vs final processing policy | [#26](https://github.com/VPRamon/spod/issues/26) |

## Product enablers

- product capability parity and migration discipline - [#19](https://github.com/VPRamon/spod/issues/19)
- dataset resolution and acquisition - [#21](https://github.com/VPRamon/spod/issues/21)
- mission profiles - [#22](https://github.com/VPRamon/spod/issues/22)
- operational QC / run health - [#23](https://github.com/VPRamon/spod/issues/23)
- observability - [#27](https://github.com/VPRamon/spod/issues/27)
- stable product/artifact API - [#28](https://github.com/VPRamon/spod/issues/28)
- SGP4/SDP4 parity preservation - [#29](https://github.com/VPRamon/spod/issues/29)

## Product principles

1. **Capabilities are more important than module paths.**
2. **No silent scientific downgrade.**
3. **Every run is auditable.**
4. **CLI and REST are interfaces to the same product.**
5. **Operational policy belongs in spod; reusable science belongs in Siderust.**
6. **Automation is a feature, not an external shell script.**
7. **Quality is an output.** A generated orbit is not enough; the service must explain whether it is fit for its configured operational purpose.

## Current status

`spod` is an engineering preview. The deterministic synthetic workflow proves the service boundary and artifact/provenance model, but the product is not commercially complete until a real-data workflow is operational.

The key transition is:

```text
synthetic reference workflow
        ->
real onboard GNSS data -> operational precise orbit product
```