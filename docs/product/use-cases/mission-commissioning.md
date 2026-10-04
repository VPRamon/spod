# Use case: Mission commissioning

## Customer

Mission operations, flight-dynamics, and ground-segment teams during early operations or after major receiver/processing changes.

## Problem

Early mission data can reveal unexpected GNSS availability, timing, receiver clock, metadata, antenna, or processing issues. Teams need a systematic way to determine whether the POD chain is operational.

## Trigger

- launch/early orbit phase;
- GNSS receiver/configuration change;
- antenna/CoM metadata update;
- major ground-processing upgrade;
- anomaly investigation.

## Inputs

- onboard GNSS observations;
- mission profile;
- supporting GNSS/EOP/antenna products;
- optional onboard navigation solution;
- optional external precise orbit;
- optional SLR observations.

## spod responsibilities

- run the supported POD workflow;
- assess data availability and gaps;
- summarize accepted/rejected observations;
- assess time/clock behavior where available;
- evaluate residuals and convergence;
- compare against external/onboard orbit when configured;
- apply commissioning acceptance criteria;
- produce a reproducible report.

## Example findings

```text
GNSS availability       PASS
time tags               PASS
receiver clock          WARNING
carrier residuals       PASS
orbit convergence       PASS
external orbit compare  PASS
SLR validation          NOT_RUN
```

## Output

- normal run products;
- commissioning report;
- issue/finding list with severity and reason;
- complete input/config provenance.

## MVP

A representative commissioning dataset can be processed through the common service and produce actionable PASS/WARNING/FAIL findings without bespoke analysis scripts.

Tracking: [#30](https://github.com/VPRamon/spod/issues/30).