# Module boundaries

| Module | Owns | Does NOT own |
|---|---|---|
| `siderust::formats` | Standard-format parsing and writing | Service orchestration and transport. |
| `siderust::pod` | Force models, propagation, observations, estimation, QC, products, and run metadata. | Service configuration and transport. |
| `spod::service` | Configuration and validation, canonical job lifecycle/status/error contracts, workflow dispatch, run orchestration, input boundaries, artifact layout, and provenance finalisation using `siderust::pod`. | Reusable numerical algorithms, HTTP semantics, and durable scheduling/persistence. |
| `spod` binaries | `clap`-based commands and the REST server; delegates to `spod::service`. | Any numerical logic. |

The current dependency and source-ownership rules are in
`dependency-rules.md`. Registry/lockfile provenance is enforced by
`scripts/check_dep_graph.sh`; local scientific implementation drift is
enforced by `scripts/check_architecture.sh` in CI.
