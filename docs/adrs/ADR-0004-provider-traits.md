# ADR-0004 — Provider-trait pattern

## Status
Superseded (by direct `siderust` dependencies in each POD crate).

## Context
POD code needs ephemerides, Earth orientation, frame transforms, gravity
fields, and atmosphere densities. Reaching directly into `siderust`
modules would couple POD's compile graph and semantics to upstream's
internal layout and cause churn whenever upstream refactors.

## Original Decision
`siderust-pod-core::providers` was meant to define five small traits:

- `EphemerisProvider`
- `EarthOrientationProvider`
- `FrameTransformProvider`
- `GravityFieldProvider`
- `AtmosphereDensityProvider`

Default implementations would wrap public `siderust` items. All
downstream POD crates were to depend on the *traits*, never on `siderust`
directly for these capabilities.

## Why This Was Superseded
In practice, four of the five provider modules were never more than
one-line `pub use siderust::...` re-exports — no trait was defined, no
abstraction existed. The only real trait (`FrameTransformProvider`) had
no implementations. Every POD call-site already referenced `siderust`
types by their fully-qualified names, so `siderust-pod-core` provided no
isolation in reality.

`siderust-pod-core` was therefore eliminated entirely. Its genuine
content was redistributed:

- `RunManifest` / `DatasetRef` → `siderust-pod-service`
- `FrameTransformProvider` → `siderust-pod-dynamics`
- `ParameterKind` / `Parameter` → `siderust-pod-estimation`
- State types (`Position`, `Velocity`, `OrbitState`, `RTN`) — callers
  import directly from `siderust::astro::dynamics::{state,frames}`.

## Consequences
- Each POD crate takes a direct `siderust` dependency only for the types
  it actually uses.
- If upstream `siderust` renames or restructures an API, the change is
  visible at each use-site rather than hidden behind an alias layer.
- Mock providers for testing can be added per-crate without a shared
  abstraction layer.
