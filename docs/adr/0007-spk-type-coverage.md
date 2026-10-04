# ADR-0007 — SPK Type coverage: Types 2 + 3 implemented; 9 + 13 deferred

## Status

Accepted.

## Context

NAIF SPICE SPK files use typed segments to store ephemeris data. Types 2 and
3 use Chebyshev representations and cover the JPL DE-series kernels used by
the current `spod` validation path. Types 9 and 13 use Lagrange interpolation
and require a separate, well-validated evaluation kernel.

Low-level DAF/SPK parsing is provided by
`siderust::formats::spice::{daf, spk}`. The value-added `spod::spice`
module owns kernel indexing, segment evaluation, body-chain resolution, and
its service ephemeris adapter.

## Decision

`spod::spice` evaluates SPK Types 2 and 3.

Types 9 and 13 are explicitly unsupported until representative mission
fixtures and validation against authoritative reference states are available.
A query that resolves to one of these segments returns
`SpiceError::UnsupportedDataType { data_type }`.

Other unsupported SPK types fail through the same explicit error path rather
than producing an approximate or silently incorrect state.

## Rationale

Implementing a segment type without authoritative validation risks silent
ephemeris errors. The current Type 2/3 implementation covers the DE440/DE441
workflow required by `spod`; additional types should be added only with
fixtures and regression cases against NAIF/CSPICE or another authoritative
reference.

## Consequences

- JPL DE-series kernels are supported through the current `SpkKernel` path.
- Mission kernels that require unsupported segment types fail explicitly.
- Adding a new segment type extends `SpkSegment`/segment dispatch and adds
  authoritative regression coverage; it does not require a compatibility
  reexport of the underlying Siderust parser namespace.
