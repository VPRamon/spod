# Acceptance Tests — E2E-01 through E2E-12

This document describes the twelve end-to-end acceptance scenarios that
constitute the Phase 10 gate for `spod`. All twelve must pass before
a 0.1.0 release tag may be created.

---

## E2E-01 — Two-body LEO propagation round-trip

**Crates under test**: `spod::dynamics`, `spod::core`
**Fixture**: synthesised circular LEO orbit (h = 500 km, i = 97°)
**Procedure**:
1. Propagate forward 24 h with DOP853.
2. Propagate backward 24 h from the terminal state.
3. Assert that the round-trip residual is < 1 mm in position and < 1 µm/s in
   velocity.
**Pass criterion**: position residual < 1 mm, velocity residual < 1 µm/s.

---

## E2E-02 — Real GNSS LEO arc (GPS-only)

**Crates under test**: `spod::io`, `spod::observations`,
`spod::dynamics`, `spod::estimation`
**Fixture**: 6-h RINEX OBS v3 + RINEX NAV + IGS precise orbits (SP3) for a
LEO spacecraft (e.g., GRACE-FO or TerraSAR-X epoch)
**Fixture gate**: `SPOD_E2E02_DIR` (see `fixtures.md`)
**Procedure**:
1. Read RINEX OBS + NAV + SP3.
2. Form ionosphere-free pseudorange combinations.
3. Run kinematic POD (no dynamics).
4. Compare estimated positions vs SP3 truth.
**Pass criterion**: 3D RMS < 10 cm over the arc.

---

## E2E-03 — Reduced-dynamic LEO arc

**Crates under test**: full pipeline
**Fixture**: same as E2E-02 + EOP + ANTEX
**Fixture gate**: `SPOD_E2E02_DIR` (shared)
**Procedure**: reduced-dynamic batch-LS with EMP accelerations.
**Pass criterion**: 3D RMS < 5 cm; empirical-accel estimates < 50 nm/s².

---

## E2E-04 — SLR validation (LEO)

**Crates under test**: `spod::io`, `spod::observations`,
`spod::qc`
**Fixture**: ILRS CRD session file (≥ 20 normal points)
**Fixture gate**: `SPOD_CRD_FILE`
**Procedure**: compute SLR residuals against estimated orbit from E2E-03.
**Pass criterion**: mean residual < 1 cm; σ < 2 cm.

---

## E2E-05 — SGP4 TLE propagation + SP3 comparison

**Crates under test**: `siderust-sgp4`, `siderust-tle`, `spod::io`
**Fixture**: ISS TLE epoch + IGS SP3 for the same epoch
**Procedure**: propagate ISS TLE to 6 epochs; compare vs SP3 positions.
**Pass criterion**: RMS < 1 km (TLE accuracy budget is ~1 km).

---

## E2E-06 — Lambert solver Earth-to-Mars transfer

**Crates under test**: `siderust-lambert`
**Fixture**: hard-coded Earth (r₁) and Mars (r₂) positions at a known transfer epoch
**Procedure**: solve Lambert problem for the optimal 0-rev transfer; verify ΔV
budget is within 5% of the Hohmann-approximate value for a circular-to-circular
approximation.
**Pass criterion**: ΔV within 5% of Hohmann estimate; solution converges in < 50 iterations.

---

## E2E-07 — SPICE-based planetary ephemeris accuracy

**Crates under test**: `siderust-spice`
**Fixture gate**: local `de440.bsp` or `de441.bsp` on `SIDERUST_SPICE_DE_PATH`
**Procedure**: evaluate Earth, Moon, Mars positions at 100 epochs; compare vs
JPL HORIZONS reference values committed in `test-data/de440_ref.json`.
**Pass criterion**: position error < 1 mm for planets, < 1 m for Moon.

---

## E2E-08 — EKF orbit determination

**Crates under test**: `spod::estimation` (EKF), `spod::dynamics`
**Fixture**: synthesised pseudorange measurements with known truth state
**Procedure**: run EKF for 2 h; run RTS smoother; compare vs truth.
**Pass criterion**: post-fit 3σ covariance contains truth state; smoother
improves RMS by ≥ 10% over filter-only.

---

## E2E-09 — Orbit overlap

**Crates under test**: full pipeline
**Fixture**: two 3-h arcs with 1-h overlap, same spacecraft
**Procedure**: estimate both arcs independently; compute 3D RMS of the estimated
positions in the overlap interval.
**Pass criterion**: overlap RMS < 5 cm.

---

## E2E-10 — QC HTML report generation

**Crates under test**: `spod::qc`
**Fixture**: residuals CSV from E2E-03
**Procedure**: generate QC HTML report; assert all required sections are present
(residual time series, sky plot, orbit-compare table, SLR validation summary).
**Pass criterion**: report renders without error; all required sections present.

---

## E2E-11 — REST API end-to-end

**Crates under test**: `spod-rest`, `spod::service`
**Fixture**: config YAML + small synthesised RINEX OBS fixture
**Procedure**:
1. Start `spod-rest` in a background thread.
2. POST a job via HTTP.
3. Poll until complete.
4. GET the SP3 product.
**Pass criterion**: job completes without error; SP3 product contains valid orbit data.

---

## E2E-12 — LISA POC (heliocentric three-spacecraft)

**Crates under test**: `spod::observations` (`InterSatRange`),
`spod::dynamics`, `spod::estimation`, `spod::io` (LISA orbit reader)
**Fixture**: LISA orbit files from `https://github.com/esa/lisa-orbit-files`
(text/ASCII format); downloaded to `SPOD_LISA_ORBIT_DIR`
**Scope**: Heliocentric three-spacecraft orbit estimation from simulated MOSA
inter-spacecraft ranges. No TDI or laser metrology (see
[`../adr/0005-lisa-poc-scope.md`](../adr/0005-lisa-poc-scope.md)).
**Procedure**:
1. Load LISA heliocentric truth orbits for all three spacecraft.
2. Simulate MOSA inter-spacecraft ranges with realistic noise (σ = 1 m).
3. Run batch-LS or EKF POD estimating the three spacecraft states.
4. Compare estimated orbits vs truth over a 30-day arc.
**Pass criterion**:
- Position RMS < 100 m for all three spacecraft.
- Estimated inter-spacecraft distances agree with truth to < 10 m.
- No divergence or NaN in the estimation.

---

## Running the full E2E suite

```bash
# All non-ignord E2E tests (synthesised fixtures only):
cargo test -p spod::service --test e2e -- --nocapture

# Unlock fixture-gated tests (all required env vars must be set):
SPOD_E2E02_DIR=/path/to/e2e02 \
SPOD_CRD_FILE=/path/to/crd \
SIDERUST_SPICE_DE_PATH=/path/to/de440.bsp \
SPOD_LISA_ORBIT_DIR=/path/to/lisa-orbit-files \
cargo test -p spod::service --test e2e -- --include-ignored --nocapture
```
