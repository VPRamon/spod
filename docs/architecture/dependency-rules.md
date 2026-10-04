# Dependency rules

`spod` is a single service/application crate built on the canonical Siderust
0.12 APIs. The dependency direction is intentionally small:

```text
spod service/application
        |
        v
  siderust::pod and siderust::formats
```

Runtime crates such as Axum, Tokio, Clap, and Serde support the application
boundary. They do not provide or replace POD scientific ownership.

## Allowed exceptions

The direct `sgp4` dependency and `src/sgp4/` implementation are temporary
product-parity exceptions. They provide full Vallado/AFSPC semantics that are
not yet available in Siderust 0.12. Their removal is gated by spod #29 and
Siderust #98, with the fixtures in `tests/sgp4_vallado.rs` preserved until
semantic parity is demonstrated.

## Forbidden drift

- Path or Git dependencies for canonical scientific crates.
- New local generic force, measurement, estimation, QC, format-parser, or
  propagation implementations outside the documented SGP4 exception.
- CLI or REST code that assembles scientific pipelines instead of dispatching
  through `spod::service`.

`scripts/check_dep_graph.sh` enforces reproducible registry/lockfile
dependencies. `scripts/check_architecture.sh` enforces the source boundary
using narrowly scoped implementation type patterns rather than a filename
blacklist. Both checks run in the CI project-guards job.
