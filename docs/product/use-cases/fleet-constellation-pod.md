# Use case: Fleet / constellation POD

## Customer

Operator of many similar LEO spacecraft.

## Problem

A single-spacecraft POD script does not scale to tens or hundreds of spacecraft. Configuration drifts, failures need isolated retries, and operators need both per-spacecraft and fleet-level visibility.

## Trigger

A processing cycle, data arrival, or scheduled campaign requires POD runs for a set of spacecraft/arcs.

## Inputs

- fleet definition;
- spacecraft membership;
- mission profile/version per spacecraft;
- time/arc selector;
- processing class;
- concurrency/retry policy.

## spod responsibilities

- expand a fleet request deterministically into individual POD jobs;
- preserve each job as an independent reproducible run;
- control concurrency and retries;
- expose fleet and per-spacecraft status;
- aggregate QC without hiding individual results;
- support safe processing-baseline rollout.

## Siderust responsibilities

None of the fleet logic belongs in Siderust. Each spacecraft job consumes the same reusable Siderust scientific engine as a single run.

## Outputs

- per-spacecraft run artifacts and manifests;
- fleet/campaign manifest;
- aggregate success/failure summary;
- fleet QC summary;
- retryable list of failed or blocked jobs.

## Operational requirements

- one failing spacecraft must not invalidate successful runs for others;
- retries should be idempotent;
- fleet status must not require parsing filesystem state;
- configuration version must be visible per spacecraft;
- high-cardinality telemetry must be designed carefully.

## MVP

A fleet definition with multiple spacecraft expands into deterministic jobs, executes using the common service runner, and produces an aggregate status/QC summary.

Tracking: [#31](https://github.com/VPRamon/spod/issues/31).