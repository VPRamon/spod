# ADR-0002 — Linear-algebra backend: faer

## Status
Accepted.

## Context
The estimator needs dense matrix factorisations (Cholesky / QR) for
weighted least squares, with a sparse upgrade path. Candidates:
`nalgebra`, `ndarray + lapack`, `faer`.

## Decision
Use `faer` (workspace dependency, version `0.22`) as the dense / sparse
linear algebra backend in `siderust-pod-estimation`. `affn` continues to
own typed positions, vectors, and 3×3 covariance transforms.

## Consequences
- Pure-Rust, no system LAPACK requirement.
- `faer` exposes both dense and sparse APIs, so the same crate covers
  the future high-degree-gravity and large-arc estimators.
- API-stability risk pre-1.0; pinned via the workspace dependency.
