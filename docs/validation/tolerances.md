# Numerical Tolerances

This document records the per-orbit-class numerical tolerances that
`spod` holds itself to. These form the quantitative acceptance
criteria referenced in the E2E test suite and in Phase 13 benchmarks.

All tolerances are **3D RMS** unless stated otherwise.

---

## Orbit determination accuracy

| Orbit class | Orbit type | Observable | Tolerance | Reference |
|---|---|---|---|---|
| LEO (GNSS-only, kinematic) | 400–800 km circular | Ionosphere-free pseudorange | < 10 cm 3D RMS | IGS kinematic POD benchmark |
| LEO (reduced-dynamic, GNSS) | 400–800 km circular | IF pseudorange + EMP | < 5 cm 3D RMS | FocusPOD M4 spec |
| LEO (SLR validation) | 400–800 km circular | SLR normal points | mean < 1 cm, σ < 2 cm | ILRS validation standard |
| LEO overlap | 400–800 km circular | 1-h overlap | < 5 cm 3D RMS | FocusPOD M5 spec |
| MEO (GNSS only) | GPS/Galileo orbit | IF pseudorange | < 5 cm 3D RMS | IGS MEO comparison |
| GEO (two-body reference) | 35 786 km circular | Analytical Kepler | < 1 mm 24-h propagation | numerical precision |
| LISA (heliocentric) | 1 AU heliocentric | Simulated ranges σ = 1 m | < 100 m 3D RMS | design target |

---

## Propagator accuracy

| Model | Test case | Tolerance | Notes |
|---|---|---|---|
| Two-body (RK4/DOP853) | 24-h GEO circular round-trip | < 1 mm | Analytical Kepler reference |
| Two-body (DOP853 adaptive) | 24-h LEO with J2 | < 1 cm | Numerical integration error |
| STM (finite-difference) | 6×6 two-body | < 1e-7 relative | Finite-diff step = 1 m / 1 mm/s |
| STM (finite-difference) | 6×6 J2 | < 1e-7 relative | Same step |
| STM (parameter rows, Cd/Crp) | Drag scale | < 1e-6 relative | Wider tolerance for empirical params |
| SGP4 | Vallado SGP4-VER reference | < 1e-6 km / < 1e-9 km/s | Per Vallado `tcppver.out` |
| Lambert (Izzo 0-rev) | Vallado Ex 7-5, Curtis Ex 5.2 | < 1e-6 km/s residual | Convergence tolerance |

---

## SPICE ephemeris accuracy

| Body | Epoch range | Tolerance | Reference |
|---|---|---|---|
| Earth (geocentre) | 2000–2030 | < 1 mm | JPL HORIZONS |
| Mars | 2000–2030 | < 1 mm | JPL HORIZONS |
| Moon | 2000–2030 | < 1 m | JPL HORIZONS (lunar model gap) |
| Jupiter | 2000–2030 | < 1 km | JPL HORIZONS (outer-planet accuracy) |

---

## File format round-trip accuracy

| Format | Field | Tolerance |
|---|---|---|
| SP3-d | Position | 1 mm (float encoding) |
| SP3-d | Clock | 1 ps |
| RINEX OBS | Pseudorange | 0.001 m (RINEX encoding) |
| RINEX OBS | Phase | 0.001 cycles |
| CCSDS OEM | Position | 1 mm |
| CCSDS OEM | Velocity | 1 µm/s |
| EOP C04 | UT1-UTC | 1 µs |
| EOP C04 | Pole | 0.1 µas |

Round-trip means: read → write → re-read → compare. All round-trip errors
must be within the encoding-precision tolerance above; they must not
accumulate with repeated round-trips.

---

## Performance gates (Phase 13 — PB-001 through PB-009)

See `docs/requirements/pod-non-functional-requirements.md` for the full
benchmark gate definitions. The benchmarks are implemented in Phase 13 and
are not yet run in CI; they will become gating at the 0.1.0 release milestone.

| Gate | Target | Notes |
|---|---|---|
| PB-001 SP3 read throughput | ≥ 500 MB/s | Streaming iterator, warm disk |
| PB-002 RINEX OBS read | ≥ 200 MB/s | Streaming iterator |
| PB-003 SGP4 propagation | ≥ 500 k calls/s | Single satellite, single epoch |
| PB-004 Lambert solver | ≥ 100 k calls/s | 0-rev, single-precision seed |
| PB-005 DOP853 propagation | ≥ 10 k steps/s | 10-body force, LEO orbit |
| PB-006 Batch-LS iteration | ≥ 1 k iterations/s | 100 observations, 6-state |
| PB-007 EKF step | ≥ 10 k steps/s | 6-state, 3 obs per epoch |
| PB-008 SPICE SPK Type 2 | ≥ 100 k evals/s | Per body, warm cache |
| PB-009 Full POD pipeline | ≤ 2× wall-clock arc length | 6-h GNSS LEO arc |

A ≥ 15% regression on any benchmark triggers CI failure (Phase 13 will
implement the regression gate via `cargo-criterion`).
