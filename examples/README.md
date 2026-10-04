# Examples

The runnable examples are intentionally small, ordered, and focused on the public APIs a downstream Rust user should reach for first.

## Example style

The examples follow the same typed-boundary rule as the library:

- Keep physical quantities as `qtty` values instead of immediately extracting `f64`.
- Keep positions and vectors tagged with their `affn` frame/center types.
- Keep epochs in `tempoch`/Siderust time types.
- Use `.value()` only when crossing an intentional raw-numeric boundary, such as an estimator kernel, FFI, or a file format that is defined in scalar storage.
- Prefer a short example that demonstrates one workflow over a large example that reimplements library functionality.

This makes the examples useful as copyable application code while still showing where the lower-level numerical interfaces belong.

## Runnable examples

| Example | What it demonstrates |
| --- | --- |
| `01_typed_two_body_propagation` | Builds a typed GCRS LEO state and propagates it with the siderust two-body model through the common `Integrator` interface |
| `02_short_arc_wls` | Assembles and solves a small POD-like weighted least-squares problem, then wraps solver-space corrections back into typed quantities |
| `03_lambert_earth_to_mars` | Solves a typed heliocentric Lambert transfer without unpacking the returned velocities into raw components |
| `04_sgp4_from_tle` | Parses a TLE and propagates typed TEME position/velocity states with the natural SGP4 minutes-since-epoch API |

Run them with:

```bash
cargo run --example 01_typed_two_body_propagation
cargo run --example 02_short_arc_wls
cargo run --example 03_lambert_earth_to_mars
cargo run --example 04_sgp4_from_tle
```

## Synthetic POD pipeline

The configuration-driven synthetic workflow remains the larger end-to-end demonstration. Its configuration lives at `examples/configs/leo_gnss_mvp1.yaml`:

```bash
cargo run --bin spod -- validate-config examples/configs/leo_gnss_mvp1.yaml
cargo run --bin spod -- run examples/configs/leo_gnss_mvp1.yaml
```

The project is pre-1.0. Real-data examples should be added when their inputs are reproducible, reasonably sized, and exercised by CI.
