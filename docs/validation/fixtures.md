# Test Fixtures — Provenance and Redistribution

This file is the authoritative registry of every test fixture committed to the
`spod` workspace. For each fixture the following are recorded:

- **Path** — relative to workspace root
- **Format** — file format / protocol
- **Source** — upstream URL or synthetic origin
- **License** — redistribution status
- **Size** — approximate uncompressed size
- **Last fetched** — date the fixture was last updated from its upstream source
- **Notes** — any deviation from the upstream spec or truncation rationale

---

## siderust (tle / satellite)

| Path | Format | Source | License | Size | Last fetched | Notes |
|---|---|---|---|---|---|---|
| `test-data/tle/iss_zarya.3le` | 3LE | Synthesised (plausible ISS parameters) | N/A | < 1 KB | 2026-05-12 | Not a real-time TLE; epoch is notional |
| `test-data/omm/iss_zarya.kvn` | OMM KVN | Synthesised | N/A | < 1 KB | 2026-05-12 | Matches `iss_zarya.3le` parameters |
| `test-data/omm/iss_zarya.xml` | OMM XML | Synthesised | N/A | < 1 KB | 2026-05-12 | Matches `iss_zarya.3le` parameters |
| `test-data/omm/iss_zarya.json` | OMM JSON | Synthesised | N/A | < 1 KB | 2026-05-12 | Matches `iss_zarya.3le` parameters |
| `test-data/omm/celestrak_sample.json` | OMM JSON | Synthesised (CelesTrak schema) | N/A | < 2 KB | 2026-05-12 | Multi-object sample; not real CelesTrak data |

---

## spod (temporary SGP4 upstream-gap validation)

| Path | Format | Source | License | Size | Last fetched | Notes |
|---|---|---|---|---|---|---|
| `tests/sgp4_vallado.rs` | Rust regression vectors | Derived from Vallado `SGP4-VER.TLE` + `tcppver.out` (https://celestrak.org/software/vallado-sw.php) | Public domain (Vallado) | < 10 KB | 2026-10-04 | Near-Earth and deep-space position/velocity epochs plus a documented propagation error |

---

## siderust (data / spk)

| Path | Format | Source | License | Size | Last fetched | Notes |
|---|---|---|---|---|---|---|
| `test-data/de440_ref.json` | JSON (reference values) | Derived from JPL HORIZONS web service outputs | JPL open-data | < 5 KB | 2026-05-12 | Position reference for Earth, Moon, Mars at 10 epochs; used by `--features de440` validation gate |

---

## Siderust canonical formats

| Path | Format | Source | License | Size | Last fetched | Notes |
|---|---|---|---|---|---|---|
| `test-data/sp3/example.sp3` | SP3-d | Synthesised (IGS-compliant structure) | N/A | < 5 KB | 2026-05-12 | 3 GPS SVs, 2 epochs; round-trip fixture |
| `test-data/rinex_obs/example_v3.rnx` | RINEX OBS v3 | Synthesised (IGS RINEX 3.05 schema) | N/A | < 5 KB | 2026-05-12 | GPS + Galileo; round-trip fixture |
| `test-data/rinex_nav/example_gps_v3.rnx` | RINEX NAV v3 | Synthesised | N/A | < 3 KB | 2026-05-12 | GPS broadcast ephemeris; round-trip fixture |
| `test-data/eop/C04_14_2022.eop` | IERS C04 | Synthesised (IERS EOP C04 schema) | N/A | < 2 KB | 2026-05-12 | 10 daily records; round-trip fixture |

### Large fixtures (not committed; env-var unlocked)

The following fixtures are too large to commit (> 500 KB) or have unclear
redistribution terms. Tests that depend on them are marked `#[ignore]` and
unlocked by setting the indicated environment variable.

| Test | Required fixture | Env var | Source |
|---|---|---|---|
| `rinex_obs_large` | 24-h RINEX OBS for a 30-station network | `SPOD_RINEX_OBS_DIR` | IGS/CDDIS |
| `sp3_igs_rapid` | IGS rapid orbit SP3 | `SPOD_SP3_FILE` | IGS/CDDIS |
| `antex_igs20` | `igs20.atx` | `SPOD_ANTEX_FILE` | IGS |
| `eop_c04_full` | Full IERS C04 series | `SPOD_EOP_C04_FILE` | IERS |
| `slr_crd` | ILRS CRD session file | `SPOD_CRD_FILE` | ILRS/EUROLAS |
| `slr_cpf` | ILRS CPF prediction file | `SPOD_CPF_FILE` | ILRS |

---

## LISA POC (E2E-12)

The LISA orbit files from `https://github.com/esa/lisa-orbit-files` are not
committed to this repository. They are downloaded at CI time (or by the user
locally) and unlocked via `SPOD_LISA_ORBIT_DIR`. See
[`acceptance-tests.md`](./acceptance-tests.md) for the E2E-12 test description.

---

## Adding a new fixture

1. Confirm the license permits redistribution in a public repository.
2. Prefer synthesised or public-domain fixtures; avoid IGS/CDDIS data with
   unclear redistribution terms unless the file is small and the IGS data
   policy covers it.
3. Add a row to this file with all fields populated.
4. Keep fixture files under 500 KB. For larger real-world files, use the
   env-var unlock pattern (see above).
