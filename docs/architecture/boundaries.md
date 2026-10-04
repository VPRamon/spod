# Module boundaries

| Module | Owns | Does NOT own |
|---|---|---|
| `siderust::formats` | Standard-format parsing and writing | Service orchestration and transport. |
| `siderust::pod` | Force models, propagation, observations, estimation, QC, products, and run metadata. | Service configuration and transport. |
| `spod::service` | Configuration and validation, workflow dispatch, run orchestration, input boundaries, artifact layout, and provenance finalisation using `siderust::pod`. | Reusable numerical algorithms and scientific compatibility facades. |
| `spod` binaries | `clap`-based commands and the REST server; delegates to `spod::service`. | Any numerical logic. |

The full set of allowed and forbidden dependency edges is in
`dependency-rules.md` and is enforced by `scripts/check_dep_graph.sh`
in CI.
