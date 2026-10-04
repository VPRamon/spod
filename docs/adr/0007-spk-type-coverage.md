# ADR-0007 — SPK Type coverage: Types 2 + 3 implemented; 9 + 13 deferred

## Status

Accepted (Phase 2.5)

## Context

NAIF SPICE SPK files use typed segments to store ephemeris data. The types
relevant to planetary-science missions are:

| Type | Description | Used by |
|---|---|---|
| 2 | Chebyshev position | JPL DE (de440/de441) |
| 3 | Chebyshev position + velocity | JPL DE (some bodies) |
| 5 | Two-body propagation | Preliminary orbits |
| 9 | Lagrange interpolation, unequal-step | MAVEN, MRO, etc. |
| 13 | Lagrange interpolation, equal-step | Cassini, etc. |
| 21 | Extended modified difference array | SPICE-toolkit |

The Phase 2.5 plan specified Types 2/3/9/13 as targets. Types 9 and 13 use
Lagrange interpolation with different step patterns, requiring a different
evaluation kernel from Types 2 and 3.

## Decision

`siderust-spice` ships **Types 2 and 3** (Chebyshev) as fully-implemented,
tested, and validated against JPL DE440/441.

Types 9 and 13 (Lagrange, unequal/equal step) are **explicitly rejected** at
runtime with `SpiceError::UnsupportedDataType { type_id: 9 }` (or 13). The
error message directs users to the ADR for context.

All other SPK types (1, 4, 5, 6, 7, 8, 10–12, 14–20, 21) are similarly
rejected with `UnsupportedDataType`.

## Rationale

Types 2 and 3 cover 100% of the JPL DE series (de430, de440, de441), which
are the ephemerides used in the spod test suite and the LISA POC.
Implementing Types 9/13 correctly requires the full Lagrange-interpolation
kernel with order selection, which is non-trivial and untested without
mission-specific SPK fixtures.

Implementing them incorrectly would silently produce wrong ephemerides, which
is worse than an explicit `UnsupportedDataType` error that tells the caller
to use a DE-series file instead.

## Consequences

- Users loading mission-specific SPK files (e.g., `cas_2004_v26.bsp`) will
  receive a clear `UnsupportedDataType` error rather than wrong positions.
- When Types 9/13 are implemented, the `UnsupportedDataType` arms become
  reachable evaluation branches; the API surface does not change.
- The `api.snapshot` for `siderust-spice` reflects only Types 2/3 evaluation
  paths until the extension lands.
