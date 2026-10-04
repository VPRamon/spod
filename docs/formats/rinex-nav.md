# RINEX 3 — Navigation Files

## Summary

RINEX 3 navigation files distribute broadcast ephemeris records for one or
more GNSS constellations. Each record contains the Keplerian-style orbital
elements, clock parameters, and metadata required to evaluate satellite
positions and clocks at user-defined epochs.

## Authoritative spec

<https://files.igs.org/pub/data/format/rinex305.pdf>

## Coverage

| Read | Write |
|------|-------|
|  ✅  |  —    |

## Owning crate

`siderust::formats::rinex::nav`

## Supported subset

- Header records: program/agency, leap seconds, time-system corrections,
  ionosphere model parameters (Klobuchar / NeQuick).
- **GPS** broadcast ephemeris records (8 broadcast lines, all 30 fields).
- Per-record validity intervals and SV health flags.

## Not supported (read)

- GLONASS, Galileo, BeiDou, QZSS, IRNSS, SBAS broadcast records — parsed
  structurally and exposed as raw lines via the `RinexNavRecord::Other`
  variant; not interpreted into orbital elements.
- Mixed-system files are supported, but only GPS records are converted to
  typed `BroadcastEphemeris` values.

## Deviations

- **Permissive parsing.** Records with non-zero leap-second offsets in
  the header are accepted; conversion to GPS time uses the value found.
- Records with malformed numeric fields are skipped with a
  `tracing::warn!`; the rest of the file is still consumed.
- Time-system corrections are exposed but not automatically applied; the
  caller selects whether to use them via the observation-model
  configuration.
