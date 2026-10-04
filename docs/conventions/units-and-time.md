# Units, Time, and Geometry — When to Use Which Crate

`spod` builds on three typed-primitive crates. Each owns a clearly
delimited concept; APIs in the POD workspace must use the right wrapper for
the right job and avoid bare `f64` whenever a typed alternative exists.

## The three primitive crates

### `qtty` — physical quantities with units

Use `qtty` for **everything that has a physical unit**: distance, velocity,
acceleration, mass, angle, frequency, dimensionless ratios, signal-to-noise,
phase, etc.

```rust,ignore
use qtty::si::{Length, Velocity, Mass, Acceleration};
use qtty::angle::Radians;

pub fn drag_acceleration(
    cd: f64,                 // dimensionless coefficient — bare f64 is OK
    area: qtty::si::Area,    // ✅ typed
    mass: Mass,              // ✅ typed
    rel_v: Velocity,         // ✅ typed
    rho: qtty::si::MassDensity, // ✅ typed
) -> Acceleration { /* ... */ }
```

Bare `f64` in a public signature is acceptable **only** for true
dimensionless ratios (drag coefficient, refraction index, eccentricity)
and even then a `qtty` dimensionless wrapper is preferred when one exists.

### `tempoch` — time

Use `tempoch` for **all time concepts**:

| Concept            | Type                              |
|--------------------|-----------------------------------|
| Instant            | `Epoch<TT>`, `JulianDate<UTC>`, … |
| Duration / period  | `Duration`, `Period`              |
| Interval           | `Interval<UTC>`                   |
| Time-scale change  | `tempoch::convert::*`             |

Never store an "epoch" as `f64 seconds_since_j2000`, `i64 nanoseconds`, or
`String "2024-01-01T00:00:00"` in a public API. Convert at the boundary
once and carry the typed value through.

### `affn` — typed geometry

Use `affn` for **anything with a frame and/or center**:

| Concept       | Type                                                |
|---------------|-----------------------------------------------------|
| Point         | `Position<Frame, Center, Unit>`                     |
| Vector        | `Velocity<Frame, Center, Unit>`, `Direction<Frame>` |
| Difference    | `Displacement<Frame, Unit>`                         |
| Rotation      | `Rotation<FrameA, FrameB>`                          |
| Frame change  | `FrameTransform<FrameA, FrameB>`                    |

Three points to remember:

1. `Position + Position` is a compile error. `Position − Position` yields
   a `Displacement`. (See workspace AGENTS.md and the `affn` crate.)
2. The frame and center markers are part of the type. A function expecting
   a GCRF position cannot accidentally be called with a TEME position.
3. The unit marker rides along: a `Position<Gcrf, Geocenter, Metres>` is
   a different type from `Position<Gcrf, Geocenter, Kilometres>`. Convert
   at the boundary, do not re-scale by hand.

## The rule

> **Public APIs avoid naked `f64`.** If a quantity has units, use `qtty`.
> If it has a time scale, use `tempoch`. If it has a frame or center,
> use `affn`. If it is genuinely dimensionless and has no spatial or
> temporal context, a bare `f64` is acceptable.

This rule is enforced socially in code review and structurally by ADR-0003
([`docs/adr/0003-typed-public-apis.md`](../adr/0003-typed-public-apis.md)).

## Examples

### ✅ Correct

```rust,ignore
use affn::{Position, Velocity};
use affn::frames::{Gcrf, Geocenter};
use qtty::si::{Length, Velocity as VelQ};
use tempoch::{Epoch, TT};

pub fn propagate_to(
    state: &OrbitState,
    target: Epoch<TT>,
) -> Result<OrbitState, Error> { /* ... */ }

pub struct OrbitState {
    pub epoch:    Epoch<TT>,
    pub position: Position<Gcrf, Geocenter, Length>,
    pub velocity: Velocity<Gcrf, Geocenter, VelQ>,
}
```

### ❌ Incorrect

```rust,ignore
// Bare f64 everywhere — no unit, frame, or time-scale information.
pub fn propagate_to(
    state: [f64; 6],
    target_seconds_since_j2000: f64,
) -> [f64; 6];
```

The "incorrect" version cannot distinguish:

- metres vs kilometres,
- TT vs TAI vs UTC vs TDB,
- GCRF vs TEME vs J2000.

Such bugs have caused real mission anomalies. The typed APIs make them
impossible at compile time.
