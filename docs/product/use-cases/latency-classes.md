# Use case: Rapid and final processing classes

## Customer

Operational ground segment that needs different trade-offs between timeliness and supporting-product quality.

## Problem

Not every orbit product is generated under the same data-availability constraints. A service needs explicit processing policies rather than hidden fallback behavior.

## Product concept

A job declares a processing/latency class. The class controls service policy such as:

- which supporting products are acceptable;
- whether the service waits for a preferred product;
- allowed fallbacks;
- priority;
- retry/wait behavior;
- publication state;
- QC expectations.

Example conceptual classes:

```text
rapid
routine
final
```

Names and timing targets should remain configurable rather than hard-coded.

## spod responsibilities

- represent the class in job configuration;
- map it to dataset-resolution policy;
- make waits/fallbacks explicit;
- record the chosen class and actual concrete inputs in provenance;
- distinguish waiting-for-input from scientific execution;
- expose latency/status metrics.

## Siderust responsibilities

None of the scheduling/input-availability policy belongs in the scientific engine.

## MVP

At least two classes select different input-resolution policies, with deterministic fallback/error behavior and integration coverage.

Tracking: [#26](https://github.com/VPRamon/spod/issues/26).