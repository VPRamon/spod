# Naming Conventions

These conventions apply uniformly across the `spod` crate and its modules.
Deviations are reviewed individually and recorded in the relevant ADR.

## Type suffixes — name by physical concept

Public types end in the role they play, not in implementation detail:

| Suffix      | Meaning                                                | Example                       |
|-------------|--------------------------------------------------------|-------------------------------|
| `*State`    | Mutable / time-evolving snapshot of an entity          | `EkfState`, `OrbitState`      |
| `*Provider` | Trait or struct that supplies data on demand           | `EphemerisProvider`           |
| `*Error`    | Enum returned from fallible operations                 | `LambertError`, `SpiceError`  |
| `*Model`    | Pure mathematical model (force, measurement, …)        | `MeasurementModel`            |
| `*Product`  | Material output of a POD run                           | `Sp3Product`, `OemProduct`    |
| `*Record`   | One row / entry in a parsed file or product            | `Sp3Record`, `RinexObsRecord` |
| `*Config`   | Static input shaping a run                             | `RunConfig`, `EkfConfig`      |
| `*Builder`  | Fluent constructor for a complex value                 | `ManifestBuilder`             |

Avoid generic suffixes such as `*Manager`, `*Helper`, `*Util`, or `*Info`.
They almost always indicate that the concept needs to be split.

## Module names — lowercase singular

```text
src/dynamics/        ✅
src/estimation/      ✅
src/observation/     ✅

src/dynamics_utils/  ❌  (use a sub-module of dynamics)
src/observations/    ❌  (singular)
src/Estimator/       ❌  (lowercase)
```

Sub-modules group by *thing*, not by function: `observations/gnss/`,
`observations/slr/`, `observations/inter_sat/`.

## Trait methods — verb-phrased

| Good                  | Bad                       |
|-----------------------|---------------------------|
| `evaluate(...)`       | `eval(...)` (abbreviated) |
| `propagate(t1, t2)`   | `prop(...)` / `do_prop()` |
| `compute_residual()`  | `residual()` (noun-only)  |
| `position_at(epoch)`  | `position(epoch)` is OK if returning a `Position` directly; prefer `*_at` for time-indexed accessors |

Constructors use `new`, `from_*`, `with_*`. Conversions use `into_*` / `as_*`
following the standard Rust API guidelines.

## Feature flags — lowercase, hyphen-separated

```toml
[features]
default              = ["serde"]
serde                = ["dep:serde", "qtty/serde"]
high-degree-gravity  = []
de440                = []
de441                = []
parquet              = ["dep:arrow", "dep:parquet"]
```

The public crate is named `spod`; its internal areas are modules such as
`spod::io` and `spod::service`. User-facing flags use hyphens.

## Tests

- **Unit tests** live inline:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      // ...
  }
  ```
- **Integration tests** live in `tests/` and are named after what they
  test, not after the implementation file:
  - `tests/sp3_roundtrip.rs` ✅
  - `tests/test_helpers.rs` ❌ (use a `mod` inside an integration test)

## Constants

`UPPER_SNAKE_CASE`. Constants with units must be a typed `qtty` value, not
a bare `f64`:

```rust
use qtty::si::Length;
pub const EARTH_RADIUS: Length = Length::from_metres(6_378_137.0);
```

## File names

- Rust source files: `snake_case.rs`.
- Markdown documents: `kebab-case.md`.
- Configs and fixtures: keep the upstream natural form (e.g. `igs1234.sp3`,
  `eopc04.62-now`).
