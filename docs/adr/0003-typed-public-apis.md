# ADR-0003 — Typed public APIs (`qtty` / `tempoch` / `affn`); no naked `f64`

## Status

Accepted (Phase 1, reflected in workspace AGENTS.md)

## Context

A common shortcut in scientific Rust code is to use raw `f64` for physical
quantities (angles, distances, time, velocities). This leads to:

- **Silent unit errors.** A function expecting radians silently accepts degrees.
- **Silent frame errors.** A function expecting ICRS silently accepts TEME.
- **Poor discoverability.** Callers must read documentation to understand units;
  the type system gives no help.
- **Impossible refactoring.** Changing a unit requires hunting every call-site.

The siderust ecosystem provides three crates precisely to solve this:

| Crate | Covers |
|---|---|
| `qtty` | Typed physical quantities, unit markers, dimension-safe arithmetic |
| `tempoch` | Typed instants, periods, time scales, epochs |
| `affn` | Typed positions, directions, velocities, frames, centers |

## Decision

All public APIs in `spod` crates use typed `qtty`/`tempoch`/`affn`
types where an equivalent exists. Specifically:

- Angles → `qtty::Angle<Radians>` or `qtty::Angle<Degrees>` (not `f64`).
- Distances → `qtty::Length<Kilometer>` or similar.
- Times / epochs → `tempoch::JulianDate<Utc>` etc. (not `f64` JD, not `chrono::DateTime`).
- Periods → `tempoch::Period` or `qtty::Second` (depending on context).
- Positions / velocities → `affn::cartesian::Position<Frame, Center, Unit>` etc.

**Exceptions**: Fields whose natural representation is a unit-less numeric
(e.g., BSTAR drag term in TLEs, eccentricity, mean motion dot) may remain
`f64` if converting to a typed quantity would require introducing a novel unit
with no standard definition. Each such exception must be documented with a
`// rationale: ...` comment on the field.

The `affn` affine-geometry semantics apply without exception:
- `Position + Position` is not defined.
- `Position - Position` returns a displacement/vector, not another Position.

## Consequences

- Callers get compile-time unit and frame safety.
- IDE completions surface unit information automatically.
- The API surface is larger (more type parameters), which is a worthwhile
  trade for correctness.
- FFI surfaces (`siderust-ffi`, `spod-rest`) must translate to/from
  typed equivalents at the ABI boundary; this is an accepted cost.
