# Use case: Historical reprocessing

## Customer

Science-processing team, operator, or mission ground segment.

## Problem

Improved models, calibration products, reference data, or software versions require historical orbit products to be regenerated. Ad-hoc loops are difficult to resume, audit, and compare.

## Product concepts

- processing baseline;
- mission-profile version;
- software/Siderust version;
- date/arc range;
- input-data policy;
- campaign manifest;
- per-run status;
- campaign-level QC.

## Trigger

A new baseline is approved or a historical correction requires regeneration of a period.

## Example

```text
spod reprocess \
  --mission my-leo \
  --from 2026-01-01 \
  --to 2026-06-30 \
  --baseline pod-v3
```

## spod responsibilities

- expand the campaign deterministically into jobs;
- resolve baseline-specific inputs;
- execute/retry individual jobs;
- skip/reuse already completed compatible runs where policy permits;
- aggregate status and QC;
- preserve provenance per job and for the campaign;
- enable old/new baseline comparison.

## Siderust responsibilities

Scientific execution remains identical to normal POD runs. Campaign orchestration does not belong in Siderust.

## MVP

A date/arc range expands into deterministic jobs, interrupted execution can resume, and a campaign summary reports completion and QC without recomputing all successful jobs.

Tracking: [#25](https://github.com/VPRamon/spod/issues/25).