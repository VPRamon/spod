# Crate boundaries

| Crate | Owns | Does NOT own |
|---|---|---|
| `spod-dynamics` | Force models, integrators, propagation, STM, RTN/LVLH/VNC frame helpers, `FrameTransformProvider` trait. | File I/O, observation models, estimator. |
| `spod-io` | Parsers/writers for SP3, RINEX, ANTEX, EOP, OEM, ... | Numerics, estimation. |
| `spod-observations` | `MeasurementModel` trait + GNSS / SLR / DORIS / VLBI implementations and corrections. | File parsing, estimator solver. |
| `spod-estimation` | Parameter blocks (`ParameterKind`, `Parameter`), design-matrix assembly, WLS / EKF, robust weighting, covariance extraction. | File parsing, observation modelling. |
| `spod-qc` | Residual statistics, orbit comparison, QC JSON, HTML reports. | Estimation algorithms, product writing. |
| `spod-products` | SP3 / OEM writers, residual product packaging, manifest packaging, naming, validation. | Numerics, service runtime. |
| `spod-service` | Job model (`RunManifest`, `DatasetRef`), config loading/validation, pipeline runner, artifact layout, manifest finalisation. | Numerical algorithms. |
| `spod` | `clap`-based commands; passthrough to `spod-service`. | Any numerical logic. |

The full set of allowed and forbidden dependency edges is in
`dependency-rules.md` and is enforced by `scripts/check_dep_graph.sh`
in CI.
