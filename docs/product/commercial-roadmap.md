# Product roadmap

This roadmap sequences work by commercial/product dependency rather than by code-module cleanup alone.

## Phase 0 - Protect and recover capability during consolidation

Goal: reduce duplicated code without losing product features.

- [#19](https://github.com/VPRamon/spod/issues/19) product capability parity
- [#9](https://github.com/VPRamon/spod/issues/9) format/orbital-mechanics consolidation under the parity rule
- [#29](https://github.com/VPRamon/spod/issues/29) preserve full SGP4/SDP4 semantics
- [#33](https://github.com/VPRamon/spod/issues/33) restore service-owned human-readable QC reporting
- [#34](https://github.com/VPRamon/spod/issues/34) preserve LISA only as a reference/example use case
- existing architecture issues #10-#13 continue under the same product guardrails

Exit criterion: every major current or recently removed capability is KEEP, MIGRATE, UPSTREAM GAP FIRST, INTENTIONALLY RETIRE, or REFERENCE EXAMPLE.

## Phase 1 - Make the service coherent

Goal: one operational application layer over Siderust.

- consolidate service orchestration (#10);
- unify CLI/REST on the same job model (#11);
- rebuild integration/CI around service behavior (#12);
- make documentation/metadata match the product (#13).

Exit criterion: service concepts rather than scientific modules define the public surface.

## Phase 2 - Build the North Star product

Goal: real onboard-GNSS data to precise orbit product.

- [#20](https://github.com/VPRamon/spod/issues/20) real GNSS POD workflow
- [#21](https://github.com/VPRamon/spod/issues/21) dataset resolution/acquisition
- [#22](https://github.com/VPRamon/spod/issues/22) mission profiles
- [#23](https://github.com/VPRamon/spod/issues/23) operational QC/run health
- [#28](https://github.com/VPRamon/spod/issues/28) stable product/artifact API

Exit criterion: a representative LEO mission can run repeatedly from real tracking data and produce standard products, QC, provenance, and a useful human-readable report.

## Phase 3 - Prove value around the core

Goal: useful commercial workflows that do not require replacing the customer's full operations stack.

- [#24](https://github.com/VPRamon/spod/issues/24) independent orbit/SLR validation
- [#30](https://github.com/VPRamon/spod/issues/30) mission commissioning

Exit criterion: `spod Validate`/commissioning can produce actionable reports on representative external datasets.

## Phase 4 - Operational scale

Goal: make the product suitable for repeated mission/fleet operations.

- [#25](https://github.com/VPRamon/spod/issues/25) historical reprocessing
- [#26](https://github.com/VPRamon/spod/issues/26) latency classes
- [#27](https://github.com/VPRamon/spod/issues/27) observability
- [#31](https://github.com/VPRamon/spod/issues/31) fleet/constellation orchestration

Exit criterion: many jobs can be operated, inspected, retried, compared, and summarized without bespoke shell automation.

## Reference extensions

Mission-specific demonstrations such as LISA can be maintained as examples that exercise the extension model without enlarging the core product surface.

## Product metrics

Useful future metrics include:

- successful run rate;
- percentage of runs requiring manual intervention;
- median and tail processing latency;
- percentage of runs with complete provenance;
- QC pass/warning/fail distribution;
- input-resolution failure rate;
- reprocessing resume success;
- fleet processing completion rate;
- API/job stability across releases.

Scientific accuracy metrics remain mission/workflow specific and should be defined by mission profiles and validation requirements.
