# ADR-0007 — SPK type coverage delegated to Siderust

## Status

Superseded by the Siderust 0.12 format implementation.

## Context

NAIF SPICE SPK files use typed segments to store ephemeris data. Types 2 and
3 use Chebyshev representations and cover the JPL DE-series kernels used by
the current `spod` validation path. Types 9 and 13 use Lagrange interpolation
and require a separate, well-validated evaluation kernel.

Low-level DAF/SPK parsing and evaluation are provided by
`siderust::formats::spice::{daf, spk}`. `spod` does not maintain a duplicate
SPICE implementation or compatibility namespace.

## Decision

Applications needing SPK support should use the canonical Siderust APIs.

## Rationale

Implementing a segment type without authoritative validation risks silent
ephemeris errors. The current Type 2/3 implementation covers the DE440/DE441
workflow required by `spod`; additional types should be added only with
fixtures and regression cases against NAIF/CSPICE or another authoritative
reference.

## Consequences

- Service code does not own reusable SPK parsing or evaluation.
