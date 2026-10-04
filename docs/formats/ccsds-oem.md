# CCSDS OEM — Orbit Ephemeris Message

## Summary

The CCSDS Orbit Ephemeris Message (OEM) is the standard ASCII / XML
exchange format for tabulated spacecraft trajectories. spod uses
OEM as its primary deliverable for non-GNSS POD products and as an
interoperable input format for trajectory ingest.

## Authoritative spec

<https://public.ccsds.org/Pubs/502x0b3e1.pdf> (CCSDS 502.0-B-3)

## Coverage

| Read | Write |
|------|-------|
|  ✅ (KVN) | ✅ (KVN) |

## Owning crate

`spod-io`

## Supported on read

- KVN (Keyword-Value Notation) flavour.
- Multi-segment files; each segment carries its own metadata block
  (`OBJECT_NAME`, `OBJECT_ID`, `CENTER_NAME`, `REF_FRAME`, `TIME_SYSTEM`,
  `START_TIME`, `STOP_TIME`).
- ISO-8601 epoch encoding (`YYYY-MM-DDTHH:MM:SS.ffffff`).
- Position-only and position+velocity ephemeris lines.
- Comments (`COMMENT` keyword) preserved as `Vec<String>` per segment.

## Supported on write

- ASCII KVN.
- **Single segment** (POD products are written as one continuous arc).
- Position **and** velocity per epoch.
- **No covariance block** in v0.0.x (the planned covariance writer is
  tracked under `FR-QC-08` extensions).

## Not supported

- XML OEM serialisation.
- KVM (Keyword-Value Message) covariance blocks on either read or write
  in v0.0.x.
- User-defined parameter blocks beyond the standard CCSDS keywords.

## Deviations

- The writer emits header lines in a fixed canonical order to support
  byte-identical reproducibility (PB-006 / E2E-06).
- Numeric formatting uses fixed precision (15 significant digits for
  positions in metres, 12 for velocities in m/s); this exceeds the CCSDS
  minimum precision and is documented in the run manifest.
