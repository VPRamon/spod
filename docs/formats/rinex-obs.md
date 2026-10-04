# RINEX 3 — Observation Files

## Summary

RINEX (Receiver Independent Exchange Format) version 3 observation files
contain GNSS observables (pseudoranges, carrier phases, Doppler, signal-to-
noise) at a configurable sampling rate, together with a header describing
the receiver, antenna, and observation types.

## Authoritative spec

<https://files.igs.org/pub/data/format/rinex305.pdf>

## Coverage

| Read | Write |
|------|-------|
|  ✅  |  —    |

## Owning crate

`siderust::formats::rinex::obs`

## Supported subset

- Header records: marker name, approx. position XYZ, antenna delta H/E/N,
  observation types per system, interval, time of first/last observation,
  leap seconds.
- Epoch blocks with epoch flag, number of satellites, receiver clock offset.
- Per-satellite observable lines (PRN + observable values + LLI + signal
  strength).
- **GPS L1/L2** and **Galileo E1/E5a** signal subset for the MVP-1 path.
- Comment records are preserved as text.

## Not supported (read)

- Other constellations (GLONASS, BeiDou, QZSS, IRNSS, SBAS) are parsed
  structurally but their observables are not interpreted by the MVP-1
  pipeline.
- RINEX 2 files (use a converter; RINEX 2 support is out of scope).
- Event flags other than `0` (OK) and `4` (header information follows).

## Deviations

- The parser is **permissive** about whitespace and trailing columns:
  rows shorter than the spec maximum are zero-padded; rows longer are
  truncated with a `tracing::warn!`.
- Receiver clock offsets are read into a typed `qtty::time::Duration`
  even when the source value is encoded with the legacy fixed-format
  precision.
