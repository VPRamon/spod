# ADR-0001 — Pure Rust only; no Python bindings in spod

## Status

Accepted (2026-05-12)

## Context

The siderust ecosystem provides a Python adapter (`python/siderust-py`) that
wraps `siderust` and `siderust-ffi` via PyO3. When planning the maturity path
for `spod`, the option to expose POD functionality directly from this
workspace through Python bindings was considered.

However:

- PyO3/maturin adds a significant build-system surface (virtual environments,
  ABI tags, wheel packaging) orthogonal to the Rust workspace.
- The POD estimation pipeline has deep type-level structure (`qtty`, `tempoch`,
  `affn`) that does not translate cleanly to Python without extensive wrapper
  code.
- Python consumers of POD are better served by consuming the products (SP3, OEM,
  CSV residuals) rather than calling into the estimation pipeline directly.
- The existing `python/siderust-py` adapter is already maintained separately;
  duplicating that maintenance burden inside `spod` would create
  synchronisation lag.

## Decision

`spod` exposes **Rust API only**. No PyO3 crate, no `maturin` build,
no Python type stubs generated from this workspace.

Python users interact with spod outputs through:
- File formats (SP3, OEM, CSV residuals, JSON manifest).
- The REST API exposed by `spod-rest` (OpenAPI-documented).
- The existing `python/siderust-py` adapter for lower-level siderust primitives.

## Consequences

- The `spod` workspace stays free of Python build machinery.
- No `p9-python` todo exists; this decision is final for the 0.x series.
- If Python integration becomes a priority in 1.x, a dedicated
  `spod-py` adapter crate may be created in the `python/` adapter
  tree, not inside this workspace.
