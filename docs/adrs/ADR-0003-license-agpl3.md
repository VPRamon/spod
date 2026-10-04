# ADR-0003 — License: AGPL-3.0

## Status
Accepted.

## Context
Foundational crates (`siderust`, `qtty`, `tempoch`, `affn`, `cheby`) are
AGPL-3.0. POD crates depend on them and must be license-compatible.

## Decision
All `spod-*` crates are licensed under AGPL-3.0.

## Consequences
- License-uniform with the rest of the Siderust ecosystem.
- A future commercial dual-license (AGPL + commercial) decision is
  deferred to M7 productization and will require its own ADR.
