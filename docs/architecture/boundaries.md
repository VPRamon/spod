# Module boundaries

| Module | Owns | Does NOT own |
|---|---|---|
| `spod::io` | Parsers/writers for SP3, RINEX, ANTEX, EOP, OEM, ... | Numerics, estimation. |
| `siderust::pod` | Force models, propagation, observations, estimation, QC, products, and run metadata. | Service configuration and transport. |
| `spod::service` | Configuration loading, job orchestration, service-owned adapters, artifact layout, and manifest finalisation using `siderust::pod`. | Reusable numerical algorithms. |
| `spod` binaries | `clap`-based commands and the REST server; delegates to `spod::service`. | Any numerical logic. |

The full set of allowed and forbidden dependency edges is in
`dependency-rules.md` and is enforced by `scripts/check_dep_graph.sh`
in CI.
