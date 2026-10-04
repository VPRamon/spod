# Product vision and positioning

## Vision

`spod` should make operational Precise Orbit Determination consumable as infrastructure.

A user should describe the mission, input datasets, processing policy, requested products, and quality requirements, and receive a reproducible POD job result without assembling scientific primitives manually.

## Product promise

> Given tracking data, supporting products, and a mission configuration, `spod` produces a controlled, auditable POD run with standard orbit products, diagnostics, quality status, and provenance.

## Why this is different from Siderust

Siderust is a scientific software platform. It provides reusable parsers, propagators, observation models, estimators, QC primitives, and product writers.

`spod` answers a higher-level operational question:

> Run the supported POD workflow for this spacecraft and tell me what happened.

Its product value is composition and operationalization.

## Target architecture

```text
operator / system
      |
  CLI / REST
      |
     spod
  +---+---------------------------+
  | configuration / jobs / QC    |
  | inputs / artifacts / policy  |
  +---+---------------------------+
      |
   Siderust
  +---+---------------------------+
  | formats / POD / astrodynamics|
  +-------------------------------+
```

## What spod should become excellent at

### Reproducible operations

Every run should preserve effective configuration, mission-profile version, exact datasets and hashes, software versions, processing baseline, outputs, and quality decisions.

### Productized workflows

Scientific APIs are flexible; a commercial product needs supported paths such as:

```text
leo-gnss-final
leo-gnss-rapid
orbit-validation
mission-commissioning
historical-reprocessing
```

### Quality gates

The service should distinguish execution success, scientific convergence, and operational quality.

### Automation

Dataset resolution, mission profiles, campaigns, fleet orchestration, retries, and product publication should be product features rather than ad-hoc scripts.

### Deployment

`spod` should support local CLI, container deployment, internal REST service, and eventually managed execution without forking the scientific behavior.

## Primary market wedge

The initial target customer is a LEO operator or ground-segment integrator with onboard GNSS measurements who needs precise orbit products without maintaining a bespoke POD software stack.

The first commercial-grade workflow should therefore be:

> **Onboard GNSS -> precise LEO orbit + QC + provenance.**

## Product extensions

- constellation/fleet processing;
- independent orbit validation;
- commissioning;
- historical reprocessing;
- rapid/final processing classes;
- SLR validation;
- DORIS workflows;
- mission-specific workflows supported by the underlying scientific engine.

## Non-goals

`spod` should not become a general astrodynamics library, a duplicate of Siderust, a desktop mission-analysis suite, or a repository for algorithms with no operational POD use case.

## Differentiation target

> **headless, API-first, reproducible, container-native, operational POD.**

Differentiation should come from workflow reliability, integration, provenance, quality reporting, and deployability, not from owning more implementations of standard scientific algorithms.