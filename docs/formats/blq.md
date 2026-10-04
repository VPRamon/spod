# BLQ — Ocean Loading Coefficients

## Summary

The BLQ format (a.k.a. SPOTL ocean-loading file) distributes per-station
ocean-loading amplitude and phase coefficients for the major tidal
constituents (M2, S2, N2, K2, K1, O1, P1, Q1, Mf, Mm, Ssa). Ocean
loading is required for high-accuracy SLR station-displacement modelling
and for sub-cm GNSS station coordinate work.

## Authoritative spec

<http://holt.oso.chalmers.se/loading/blq.html>

## Coverage

| Read | Write |
|------|-------|
|  —   |  —    |

Planned for **Phase 5** alongside the SINEX coordinate-frame extension.

## Owning crate

`siderust::formats` (when supported upstream).

## Notes

- BLQ files are the de-facto standard distributed by the Onsala
  Bohnsdorff loading service and IGS station coordinate processors.
- Until BLQ ingestion lands, callers must compose their own per-station
  displacement series at the service boundary.
- Ocean loading is **required** for the SLR validation accuracy gates
  (`PB-008` reference; see also the Mendes-Pavlis tropospheric model).
