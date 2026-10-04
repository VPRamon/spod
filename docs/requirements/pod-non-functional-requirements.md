# POD Non-Functional Requirements

This document captures the performance benchmarks (PB), accuracy gates,
and reproducibility requirements for `spod`.

## Performance benchmarks

All benchmarks run under `cargo bench` on a single-node Linux x86_64
reference machine (16 cores @ 3.5 GHz, 64 GB RAM, NVMe storage). Targets
are per-thread unless noted.

| ID     | Subject                                              | Target                              |
|--------|------------------------------------------------------|-------------------------------------|
| PB-001 | SP3 parse throughput                                 | ≥ 50 MB/s                           |
| PB-002 | RINEX OBS parse throughput                           | ≥ 200 k observations/s              |
| PB-003 | 24 h LEO propagation (full force model)              | ≤ 250 ms wall                       |
| PB-004 | GNSS residual evaluation                             | ≥ 500 k residuals/s                 |
| PB-005 | Batch LSQ on 24 h synthetic LEO config               | ≤ 3 s wall, ≤ 200 MB peak RSS       |
| PB-006 | QC report generation                                 | ≥ 1 M residual records/s            |
| PB-007 | SGP4 propagation, single thread                      | ≥ 1 M propagations/s                |
| PB-008 | SPK Type 2/3 evaluation, per body                    | ≥ 100 k evals/s                     |
| PB-009 | LAMBDA integer fix on a 30-satellite configuration   | ≤ 50 ms wall                        |

A regression of >10 % from the published target on a release build fails
the nightly performance run; smaller fluctuations are tracked over time.

## Accuracy gates

Per-orbit-class accuracy targets, validated by the corresponding E2E
acceptance tests (see [`../validation/acceptance-tests.md`](../validation/acceptance-tests.md)).

| Orbit / domain                               | Observable                                   | Target                                         |
|----------------------------------------------|----------------------------------------------|------------------------------------------------|
| LEO GNSS, batch WLS                          | 3D position RMS vs reference SP3             | ≤ 5 cm                                         |
| LEO GNSS, EKF replay                         | 3D position vs batch state                   | ≤ 10 cm and within 3·σ                         |
| LEO SLR validation                           | Normal-point residuals                       | Within Mendes-Pavlis tropospheric tolerance    |
| Lambert solver (1 AU transfer)               | Round-trip closure                           | < 1 m                                          |
| SGP4 vs IGS SP3                              | Position difference                          | Within published SGP4 error budget (~1 km)     |
| SPICE planets vs JPL HORIZONS (DE441)        | Position                                     | ≤ 1 mm                                         |
| SPICE Moon vs JPL HORIZONS (DE441)           | Position                                     | ≤ 1 m                                          |
| LISA POC                                     | Inter-spacecraft state                       | Within ranging-only observability              |

## Reproducibility

- **Bit-identical outputs.** Two consecutive runs of the same `RunConfig`
  on the same input fixtures must produce byte-identical SP3 products,
  byte-identical OEM products, byte-identical residual files, and an
  identical canonical-JSON manifest hash.
- **Deterministic ordering.** The 15-stage pipeline runs in a fixed
  order; any internal parallelism is paired with a deterministic merge
  step.
- **Environment hashing.** The manifest records: input file hashes,
  config canonical hash, crate version + git commit, rustc version,
  EOP source date, and any environment overrides.
- **No wall-clock drift.** Outputs do not embed `now()` timestamps; all
  embedded time references are derived from the input data.
- **No locale dependence.** Numeric formatting uses `C`-locale floating
  point with fixed precision. No `LC_NUMERIC` surprises.

These guarantees are exercised by E2E-06 (see acceptance tests).

## Resource ceilings (advisory)

Soft ceilings used for sizing CI runners and example configs:

| Resource                 | Ceiling                                  |
|--------------------------|------------------------------------------|
| Single-arc memory peak   | ≤ 2 GB                                   |
| Single-arc wall time     | ≤ 5 minutes on CI runner                 |
| Workspace `cargo build`  | ≤ 5 minutes (cold) / ≤ 30 s (incremental)|
| Workspace `cargo test`   | ≤ 10 minutes (cold) on CI runner         |
