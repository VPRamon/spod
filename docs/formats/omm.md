# OMM — Orbit Mean-Elements Message

## Summary

The CCSDS Orbit Mean-Elements Message (OMM) is the modern, structured
replacement for the legacy TLE text format. It expresses the same mean
orbital elements consumable by SGP4/SDP4, in three interchangeable
serialisations: KVN, XML, and JSON. CelesTrak distributes large catalogs
in JSON OMM today.

## Authoritative spec

<https://public.ccsds.org/Pubs/502x0b3e1.pdf> (CCSDS 502.0-B-3) plus the
CelesTrak extensions documented at
<https://celestrak.org/NORAD/documentation/>.

## Coverage

| Read | Write |
|------|-------|
|  ✅  |  —    |

## Owning crate

`siderust::astro::satellite::tle` (and re-exported from `spod`
when an OMM bundle is supplied as an alternative to a TLE file).

## Supported subset

- All three serialisations (**KVN**, **XML**, **JSON**) round-trip into
  the same `Omm` value.
- All TLE-equivalent fields: `EPOCH`, `MEAN_MOTION`, `ECCENTRICITY`,
  `INCLINATION`, `RA_OF_ASC_NODE`, `ARG_OF_PERICENTER`, `MEAN_ANOMALY`,
  `BSTAR`, `MEAN_MOTION_DOT`, `MEAN_MOTION_DDOT`, `EPHEMERIS_TYPE`,
  `CLASSIFICATION_TYPE`, `NORAD_CAT_ID`, `ELEMENT_SET_NO`, `REV_AT_EPOCH`,
  `OBJECT_NAME`, `OBJECT_ID`, `CENTER_NAME`, `REF_FRAME`, `TIME_SYSTEM`,
  `MEAN_ELEMENT_THEORY`.
- Conversion to `Tle` via `Omm::to_tle()` so SGP4 can consume either
  source uniformly.

## Not supported

- Covariance blocks (CCSDS allows attaching a 6×6 covariance to an OMM;
  not yet exposed as a typed covariance value in v0.0.x).
- User-defined parameter blocks (`USER_DEFINED_*`) are carried verbatim
  but not parsed.
- Writing OMM in any serialisation. spod produces orbital
  products as SP3 or OEM, not OMM.

## Deviations

- Parsing is **lenient about case and whitespace** in keyword names for
  KVN. JSON parsing is strict and follows `serde_json` defaults.
- Out-of-range field values (e.g. eccentricity ≥ 1) are accepted at the
  parse boundary but flagged as `TleError::InvalidElement` when the
  caller materialises the record into a `Tle` for SGP4.
