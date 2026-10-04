# IERS EOP — Earth Orientation Parameters (C04)

## Summary

The IERS Earth Orientation Parameters (EOP) C04 series provides daily
values of polar motion (`x_p`, `y_p`), UT1−UTC, length-of-day (LOD), and
celestial-pole offsets (`dPsi`, `dEps`) consistent with the IAU 2006/2000
precession-nutation model. EOP values are required for the GCRF↔ITRF
frame transform.

## Authoritative spec

<https://www.iers.org/IERS/EN/DataProducts/EarthOrientationData/eop.html>

## Coverage

| Read | Write |
|------|-------|
|  ✅  |  —    |

## Owning crate

`siderust::formats::iers::eop`

## Supported subset

- Daily C04-style row format with whitespace-separated columns.
- Fields parsed: MJD, x_p, y_p, UT1−UTC, LOD, dPsi, dEps (and the
  associated 1-σ values when present).
- File comment / header lines beginning with `#` or non-numeric leading
  characters are skipped.

## Helpers

- `EopTable::interpolate_at(mjd)` performs **linear interpolation**
  between bracketing daily rows. The interpolation also handles UT1
  step discontinuities at leap seconds (UT1−UTC is unwrapped before
  interpolation and re-wrapped on the way out).
- `EopTable::range()` reports the (start, end) MJD of the loaded series.

## Not supported (read)

- Sub-daily (high-frequency) EOP time series.
- The Bulletin A predicted columns (long-term predictions) — only the
  observed columns are interpreted.

## Deviations

- The reader does **not** automatically download updates; the caller
  supplies the file path. A future runtime/bundled-data integration will
  bridge to `tempoch`'s data-loading conventions.
- Values are stored in their natural units (`x_p`, `y_p` in arc-seconds;
  UT1−UTC in seconds; dPsi/dEps in milli-arc-seconds) and exposed as
  typed `qtty` quantities.
