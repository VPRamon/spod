# Capability map

This document maps product capabilities to their intended long-term owner. The detailed parity inventory is tracked by [#19](https://github.com/VPRamon/spod/issues/19).

## Disposition vocabulary

- **KEEP IN SPOD** - service/application capability.
- **MIGRATE TO SIDERUST** - product capability remains, implementation becomes canonical upstream.
- **UPSTREAM GAP FIRST** - capability must not be deleted until upstream has required semantics.
- **INTENTIONALLY RETIRE** - capability is deliberately not part of the core product.
- **REFERENCE EXAMPLE** - mission-specific capability demonstrated outside the core product surface.
- **ADD TO SPOD** - missing product/application capability.

## Existing capabilities

| Capability | Product relevance | Intended owner | Disposition |
|---|---|---|---|
| Run configuration | core | spod | KEEP IN SPOD |
| Job execution | core | spod | KEEP IN SPOD |
| Artifact layout | core | spod | KEEP IN SPOD |
| Run provenance/manifests | operational policy around reusable primitives | spod + Siderust | KEEP IN SPOD |
| CLI | core interface | spod | KEEP IN SPOD |
| REST API | core interface | spod | KEEP IN SPOD |
| Docker/deployment | operations | spod | KEEP IN SPOD |
| Synthetic reference workflow | integration/reference | spod + Siderust | KEEP IN SPOD |
| Human-readable QC report | product presentation/artifact | spod | KEEP IN SPOD / RESTORE - [#33](https://github.com/VPRamon/spod/issues/33) |
| Force models | scientific primitive | Siderust | MIGRATE TO SIDERUST |
| Propagation / STM | scientific primitive | Siderust | MIGRATE TO SIDERUST |
| WLS / nonlinear / sequential estimation | scientific primitive | Siderust | MIGRATE TO SIDERUST |
| Observation models | scientific primitive | Siderust | MIGRATE TO SIDERUST |
| Generic QC statistics, orbit comparison, SLR validation | scientific primitive | Siderust | MIGRATE TO SIDERUST; expose through spod workflows |
| SP3/OEM serialization | standard product primitive | Siderust | MIGRATE TO SIDERUST |
| Run-manifest JSON schema | reusable run contract | Siderust | MIGRATE TO SIDERUST; canonical schema already exists upstream |
| TLE / 3LE / OMM | standard formats | Siderust | MIGRATE TO SIDERUST |
| SP3 / RINEX / ANTEX / SINEX / ORBEX | standard formats | Siderust | MIGRATE TO SIDERUST |
| EOP / CRD / CPF / OEM / OPM / TDM | standard formats | Siderust | MIGRATE TO SIDERUST |
| SPICE / SPK | scientific/format primitive | Siderust | MIGRATE TO SIDERUST |
| Full Vallado SGP4/SDP4 | useful operational capability | canonical Siderust ecosystem backend | UPSTREAM GAP FIRST - [#29](https://github.com/VPRamon/spod/issues/29) |
| Lambert solver | generic mission-design function | ecosystem | INTENTIONALLY RETIRE from core spod unless a concrete operational workflow is justified |
| LISA-specific functionality | mission-specific reference | example/reference layer | REFERENCE EXAMPLE - [#34](https://github.com/VPRamon/spod/issues/34) |
| DORIS parser | standard format | Siderust | MIGRATE TO SIDERUST; not a spod feature by itself |
| DORIS POD workflow | mission/product workflow | spod + Siderust | future product decision |
| de440 feature | validation/development mechanism | tests/upstream | not a product capability |
| Parquet export | optional artifact format | Siderust serializer + spod policy | KEEP only when exposed by a supported workflow |

## Missing product capabilities

| Capability | Why it belongs in spod | Tracking |
|---|---|---|
| Real onboard-GNSS POD | core commercial workflow | [#20](https://github.com/VPRamon/spod/issues/20) |
| Dataset resolution/acquisition | operational orchestration | [#21](https://github.com/VPRamon/spod/issues/21) |
| Versioned mission profiles | application configuration | [#22](https://github.com/VPRamon/spod/issues/22) |
| Operational QC / run health | service policy | [#23](https://github.com/VPRamon/spod/issues/23) |
| Independent orbit/SLR validation | product workflow | [#24](https://github.com/VPRamon/spod/issues/24) |
| Human-readable QC artifact | application/reporting | [#33](https://github.com/VPRamon/spod/issues/33) |
| Reprocessing campaigns | orchestration | [#25](https://github.com/VPRamon/spod/issues/25) |
| Latency/processing classes | operational policy | [#26](https://github.com/VPRamon/spod/issues/26) |
| Metrics / observability | operations | [#27](https://github.com/VPRamon/spod/issues/27) |
| Stable product/artifact API | service interface | [#28](https://github.com/VPRamon/spod/issues/28) |
| Mission commissioning | product workflow | [#30](https://github.com/VPRamon/spod/issues/30) |
| Fleet/constellation orchestration | scale/operations | [#31](https://github.com/VPRamon/spod/issues/31) |
| LISA reference example | extension/reference use case | [#34](https://github.com/VPRamon/spod/issues/34) |

## Target public concept model

Long-term `spod` concepts should look more like:

```text
MissionProfile
RunConfig
Job
JobStatus
Workflow
ResolvedInputs
RunResult
Artifact
QcReport
RunManifest
Campaign
Fleet
```

and less like a second namespace of generic force models, estimators, standard-format parsers, SPICE internals, or mission-design algorithms.

## Important distinction

Removing an old Rust API such as a local TLE parser does not imply removing the ability for a supported `spod` workflow to accept TLE data. The workflow should call the canonical Siderust implementation directly while preserving the product capability.
