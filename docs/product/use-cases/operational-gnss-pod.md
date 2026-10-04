# Use case: Operational onboard-GNSS POD

## Customer

LEO/Earth-observation operator or ground-segment integrator.

## Problem

The spacecraft produces onboard GNSS observations. The operator needs precise orbit products with known quality and provenance, but does not want to assemble and maintain a bespoke stack of parsers, force models, estimators, QC scripts, and product writers.

## Trigger

A new observation arc is available for processing.

## Inputs

Typical input contract:

- onboard GNSS observations (for example RINEX);
- precise GNSS orbit/clock products as required;
- antenna calibration;
- EOP;
- mission profile / spacecraft metadata;
- arc start/end;
- processing/estimation policy.

Dataset acquisition may be explicit initially and progressively automated by [#21](https://github.com/VPRamon/spod/issues/21).

## spod responsibilities

- validate the request and mission profile;
- resolve concrete input datasets;
- check availability/coverage and hash inputs;
- translate supported configuration into a Siderust POD problem;
- run the supported estimation workflow;
- apply operational QC policy;
- write and index artifacts;
- report status/errors;
- preserve complete provenance;
- expose the same job through CLI and REST.

## Siderust responsibilities

- standard format parsing;
- dynamics/force models;
- propagation/variational equations;
- GNSS observation models;
- estimation kernels;
- reusable QC primitives;
- standard product serialization.

## Outputs

At minimum:

```text
run.manifest.json
products/orbit.sp3
products/orbit.oem
residuals/residuals.csv
qc/qc.json
```

Future outputs may include covariance/uncertainty and additional product formats.

## Operator result

A successful run should answer both:

1. Did the job execute?
2. Is the resulting orbit operationally acceptable?

Execution success and quality success are separate concepts.

## Quality indicators

Possible indicators include:

- estimator convergence;
- accepted/rejected observation count;
- code/carrier residual statistics;
- reduced chi-square;
- covariance/uncertainty when available;
- orbit continuity;
- reference-orbit comparison when configured;
- SLR validation when configured;
- completeness of required inputs.

Thresholds belong in mission/workflow policy, not in generic Siderust kernels.

## Failure modes

`spod` should distinguish at least:

- invalid configuration;
- unavailable input dataset;
- unsupported data/model;
- parse/coverage failure;
- scientific convergence failure;
- artifact publication failure;
- completed run that fails operational QC.

## MVP

The MVP is complete when one representative real LEO dataset can run end to end repeatedly and produce standard orbit products, residuals, QC, and complete provenance through the same service API used by CLI/REST.

Tracking: [#20](https://github.com/VPRamon/spod/issues/20).