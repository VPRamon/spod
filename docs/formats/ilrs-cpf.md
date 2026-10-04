# ILRS CPF — Consolidated Prediction Format

## Summary

The ILRS Consolidated Prediction Format (CPF) distributes target
predictions (positions, optionally velocities) used by SLR stations to
plan tracking passes. spod ingests CPF for SLR validation runs
and target visibility planning.

## Authoritative spec

<https://ilrs.gsfc.nasa.gov/data_and_products/formats/cpf.html>

## Coverage

| Read | Write |
|------|-------|
|  ✅  |  —    |

## Owning crate

`spod-io`

## Supported record subset

- **H1** — source / version header.
- **H2** — target / reference frame metadata.
- **Record type 10** — position record (position vector at epoch).

## Not supported (read)

- **Record type 20** — velocity record (parsed structurally; not exposed
  as typed velocity in v0.0.x).
- **Record type 30** — corrections / centre-of-mass record.
- **Record type 50** — transponder timing record.
- Trailer / end-of-file diagnostic records beyond what is needed to
  validate file completeness.

## Deviations

- The reference frame declared in `H2` is mapped to an `affn` frame
  marker at the boundary; unknown frame strings are surfaced as
  `Error::UnknownFrame { name }` rather than silently coerced.
- All records within a file are assumed to share a single time system
  (UTC); sub-records carrying their own time system are not yet
  supported.
