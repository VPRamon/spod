# SP3 — Standard Product 3

## Summary

The Standard Product 3 (SP3) format is the IGS-defined exchange format for
precise satellite orbit and clock products. Each file contains a header
describing the satellite system, the epoch grid, and per-epoch position
records (`P`), with optional velocity (`V`), event (`EP`), and event-velocity
(`EV`) records.

## Authoritative spec

<https://files.igs.org/pub/data/format/sp3d.pdf> (SP3-d)

## Coverage

| Read | Write |
|------|-------|
|  ✅  |  ✅   |

## Owning crate

`spod-io`

## Supported subset

- Header lines: `#`, `##`, `+`, `++`, `%c`, `%f`, `%i`, `*`.
- Epoch lines (`*  YYYY MM DD ...`).
- `P` records: per-satellite position + clock with accuracy code.
- Clock records associated with `P` records.
- Multiple satellite systems per file (G, R, E, C, J, I, S).

## Not supported (read)

- `V` records (velocity) — silently skipped.
- `EP` / `EV` event records — silently skipped.
- Correlation matrix records (SP3-c extension) — silently skipped.

## Deviations

- **Verbatim header preservation.** When round-tripping a file (read →
  write), the original header lines are preserved byte-for-byte where
  the public `Sp3Product` API has not modified the corresponding field.
  This avoids gratuitous diffs against archived IGS products.
- Trailing whitespace in records is normalised on write.
- The writer always emits `EOF` as the last line.
