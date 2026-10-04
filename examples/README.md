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
| `configs/leo_gnss_mvp1.yaml` | Runs the synthetic LEO/GNSS POD reference workflow |

Run it with:

```bash
cargo run --bin spod -- run examples/configs/leo_gnss_mvp1.yaml
```

## Synthetic POD pipeline

The configuration-driven synthetic workflow remains the larger end-to-end demonstration. Its configuration lives at `examples/configs/leo_gnss_mvp1.yaml`:

```bash
cargo run --bin spod -- validate-config examples/configs/leo_gnss_mvp1.yaml
cargo run --bin spod -- run examples/configs/leo_gnss_mvp1.yaml
```

The project is pre-1.0. Real-data examples should be added when their inputs are reproducible, reasonably sized, and exercised by CI.
