# Capability map

This document maps product capabilities to their intended long-term owner. The detailed parity inventory is tracked by [#19](https://github.com/VPRamon/spod/issues/19).

## Disposition vocabulary

- **KEEP IN SPOD** - service/application capability.
- **MIGRATE TO SIDERUST** - product capability remains, implementation becomes canonical upstream.
- **UPSTREAM GAP FIRST** - capability must not be deleted until upstream has required semantics.
- **DECISION REQUIRED** - explicit product decision needed.
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
| Force models | scientific primitive | Siderust | MIGRATE TO SIDERUST |
| Propagation / STM | scientific primitive | Siderust | MIGRATE TO SIDERUST |
| WLS / nonlinear estimation | scientific primitive | Siderust | MIGRATE TO SIDERUST |
| Observation models | scientific primitive | Siderust | MIGRATE TO SIDERUST |
| Generic QC statistics | scientific primitive | Siderust | MIGRATE TO SIDERUST |
| SP3/OEM serialization | standard product primitive | Siderust | MIGRATE TO SIDERUST |
| TLE / 3LE / OMM | standard formats | Siderust | MIGRATE TO SIDERUST |
| SP3 / RINEX / ANTEX / SINEX / ORBEX | standard formats | Siderust | MIGRATE TO SIDERUST |
| EOP / CRD / CPF / OEM / OPM / TDM | standard formats | Siderust | MIGRATE TO SIDERUST |
| SPICE / SPK | scientific/format primitive | Siderust | MIGRATE TO SIDERUST |
| Full Vallado SGP4/SDP4 | useful operational capability | canonical Siderust ecosystem backend | UPSTREAM GAP FIRST - [#29](https://github.com/VPRamon/spod/issues/29) |
| Lambert solver | generic mission-design function | ecosystem | DECISION REQUIRED |
| LISA-specific functionality | mission-specific | TBD | DECISION REQUIRED |
| DORIS workflow | potential POD workflow | spod + Siderust | DECISION REQUIRED / likely future |
| Parquet export | product/export capability | Siderust serializer + spod policy | KEEP if used by supported workflows |

## Missing product capabilities

| Capability | Why it belongs in spod | Tracking |
|---|---|---|
| Real onboard-GNSS POD | core commercial workflow | [#20](https://github.com/VPRamon/spod/issues/20) |
| Dataset resolution/acquisition | operational orchestration | [#21](https://github.com/VPRamon/spod/issues/21) |
| Versioned mission profiles | application configuration | [#22](https://github.com/VPRamon/spod/issues/22) |
| Operational QC / run health | service policy | [#23](https://github.com/VPRamon/spod/issues/23) |
| Independent orbit validation | product workflow | [#24](https://github.com/VPRamon/spod/issues/24) |
| Reprocessing campaigns | orchestration | [#25](https://github.com/VPRamon/spod/issues/25) |
| Latency/processing classes | operational policy | [#26](https://github.com/VPRamon/spod/issues/26) |
| Metrics / observability | operations | [#27](https://github.com/VPRamon/spod/issues/27) |
| Stable product/artifact API | service interface | [#28](https://github.com/VPRamon/spod/issues/28) |
| Mission commissioning | product workflow | [#30](https://github.com/VPRamon/spod/issues/30) |
| Fleet/constellation orchestration | scale/operations | [#31](https://github.com/VPRamon/spod/issues/31) |

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

and less like a second scientific namespace such as `spod::forces`, `spod::estimation`, `spod::tle`, `spod::spice`, or `spod::lambert`.

## Important distinction

Removing an old Rust API such as `spod::tle::parse_tle` does not imply removing the ability for a supported `spod` workflow to accept TLE data. The workflow should call `siderust::formats::tle` directly while preserving the product capability.