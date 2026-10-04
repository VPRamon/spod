# Use case: Independent orbit validation

## Customer

Satellite operator, ground-segment integrator, mission acceptance team, or external validation provider.

## Problem

The customer already has an operational orbit solution but needs independent evidence of accuracy, continuity, or regression after a processing/software change.

## Product opportunity

This use case can deliver commercial value without requiring the customer to replace its primary POD engine.

## Inputs

- candidate orbit product;
- independent/reference orbit product;
- optional observations/residual data;
- optional SLR data;
- validation profile/thresholds.

## spod responsibilities

- validate/resolve input products;
- align comparable intervals/epochs;
- orchestrate canonical orbit-comparison/QC primitives;
- apply service-level thresholds;
- produce machine-readable and human-readable findings;
- preserve provenance for both solutions.

## Core metrics

Where scientifically valid and supported:

- 3D RMS;
- radial / along-track / cross-track RMS;
- maxima/percentiles;
- coverage/gaps;
- time-series differences;
- residual/SLR statistics when provided.

## Output example

```text
overall              PASS
3D RMS               0.028 m
radial RMS           0.009 m
along-track RMS      0.021 m
cross-track RMS      0.015 m
coverage             PASS
SLR validation       NOT_RUN
```

The exact thresholds are profile-specific.

## Failure/quality semantics

A technically successful comparison may still return WARNING/FAIL because the candidate orbit exceeds configured quality gates.

## MVP

Compare two supported standard orbit products end to end, report RTN/RAC-style statistics where appropriate, generate a reproducible report, and expose it through the common service layer.

Tracking: [#24](https://github.com/VPRamon/spod/issues/24).