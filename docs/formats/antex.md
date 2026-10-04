# ANTEX — Antenna Exchange Format

## Summary

ANTEX is the IGS exchange format for GNSS antenna phase-centre offsets
(PCO) and phase-centre variations (PCV). It is used to correct the
geometric difference between an antenna's reference point (ARP) and the
electrical phase centre on a per-frequency basis.

## Authoritative spec

<https://files.igs.org/pub/data/format/antex14.txt>

## Coverage

| Read | Write |
|------|-------|
|  ✅  |  —    |

## Owning crate

`spod::io`

## Supported subset

- Header section: version, satellite system, ref antenna.
- Antenna blocks: `START OF ANTENNA` … `END OF ANTENNA`.
- `TYPE / SERIAL NO`, `DAZI`, `ZEN1 / ZEN2 / DZEN`, `# OF FREQUENCIES`,
  `VALID FROM / VALID UNTIL`.
- Per-frequency `START OF FREQUENCY` … `END OF FREQUENCY` blocks.
- **NORTH / EAST / UP** PCO values per frequency.

## Not supported (read)

- **PCV grids.** The phase-centre variation grids (azimuth × zenith) are
  parsed structurally but exposed as opaque arrays; the MVP-1 observation
  model uses PCO-only corrections.
- Receiver antenna entries are read but only the PCO values are exposed.

## Deviations

- ANTEX 1.3 files are accepted; missing 1.4-only fields default to
  documented values.
- Comments inside antenna blocks are preserved as `String` for round-trip
  diagnostics.
