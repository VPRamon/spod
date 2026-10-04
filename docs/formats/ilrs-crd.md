# ILRS CRD — Consolidated Laser Ranging Data

## Summary

The ILRS Consolidated Laser Ranging Data (CRD) format is the standard
exchange format for SLR (Satellite Laser Ranging) observations,
distributing both full-rate and normal-point measurements together with
station, target, and meteorological metadata.

## Authoritative spec

<https://ilrs.gsfc.nasa.gov/data_and_products/formats/crd.html>

## Coverage

| Read | Write |
|------|-------|
|  ✅  |  —    |

## Owning crate

`spod-io`

## Supported record subset

- **H1** — header (station / file metadata).
- **H2** — station information.
- **H3** — target information (PRN / NORAD ID / SLR identifier).
- **H4** — session information (start / stop, environment, configuration).
- **C0** — system configuration (laser / detector / timing).
- **Record type 10** — full-rate range record.
- **Record type 11** — normal-point range record.

## Not supported (read)

- **Record type 20** — meteorological record (parsed structurally as raw
  values, not yet bridged into the SLR observation model).
- Calibration records (types 40, 41, 42).
- Statistics records (types 50, 60).
- Configuration records other than `C0`.

## Deviations

- The CRD line endings (CR/LF or LF) are normalised on read.
- Time tags within a session are interpreted in the time system declared
  in `H4`; UTC is the only currently supported value.
- Range values are exposed as typed `qtty::si::Length` (metres);
  intrinsic CRD time-of-flight encoding is unwrapped at the boundary.
