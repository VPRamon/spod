# Commercial model and packaging

This page describes plausible ways to package `spod`. It is directional, not a pricing commitment.

## Product thesis

The commercial object is not the scientific algorithm itself. The value is supported POD workflows, operational reliability, integration, reproducibility, quality reporting, deployment, and controlled upgrades.

## spod Engine

Self-hosted operational POD service for customers who need to run inside their own infrastructure.

Expected surfaces:

- container image;
- CLI;
- REST API;
- mission profiles;
- artifact integration;
- metrics/logging;
- supported releases and onboarding.

Likely customers include satellite operators, ground-segment integrators, institutional missions, and organizations with data-residency requirements.

## spod Validate

Independent orbit-quality and processing-validation product that can be used even when `spod` is not the primary POD engine.

Core capabilities:

- orbit-vs-reference comparison;
- RTN/RAC statistics;
- residual assessment when available;
- SLR validation when available;
- processing-baseline comparison;
- machine-readable quality status;
- audit report.

Tracked by [#24](https://github.com/VPRamon/spod/issues/24).

This may have lower adoption friction because the customer does not need to replace its operational POD chain.

## spod Cloud

Managed POD execution where a customer submits or streams tracking data and obtains products through an API.

Additional requirements beyond Engine include durable storage, tenancy, auth, usage accounting, durable queues, SLOs, and security/compliance. These are not immediate repository goals.

## Deployment modes

```text
local CLI
   |
container
   |
customer REST service
   |
managed cloud service
```

Scientific behavior should not fork by deployment mode.

## Commercial readiness gates

### Gate 1 - Engineering reference

- deterministic synthetic workflow;
- provenance;
- CI;
- CLI/REST/container.

### Gate 2 - Real-data product proof

- onboard GNSS workflow;
- real standard datasets;
- standard orbit products;
- operational QC;
- documented validation case.

This is the critical transition represented by [#20](https://github.com/VPRamon/spod/issues/20).

### Gate 3 - Operational pilot

- mission profiles;
- dataset resolution;
- stable API;
- structured errors;
- observability;
- repeated daily operation on a representative mission.

### Gate 4 - Production deployment

- controlled upgrades;
- run/campaign recovery;
- resource sizing;
- support procedures;
- security review;
- sufficient external validation for the target accuracy class.

### Gate 5 - Scale

- fleet orchestration;
- reprocessing campaigns;
- multiple latency classes;
- operational dashboards/alerts.

## Possible charging models

Potential packaging could support annual self-hosted license/support, managed service per spacecraft/run, mission onboarding, and validation/commissioning packages.

Commercial value should come from operational product quality rather than duplicating scientific code that belongs in Siderust.