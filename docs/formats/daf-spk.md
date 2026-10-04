# NAIF DAF/SPK

## Summary

NAIF SPICE distributes planetary, satellite, and spacecraft trajectory data
in DAF (Double-precision Array File) containers. SPK (SP-Kernel) stores typed
ephemeris segments inside DAF files; binary kernels conventionally use the
`.bsp` extension.

## Authoritative specifications

- DAF: <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/daf.html>
- SPK: <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/spk.html>

## Ownership

Low-level DAF parsing and raw SPK decoding are canonical Siderust APIs:

```text
siderust::formats::spice::daf
siderust::formats::spice::spk
```

`spod::spice` adds service/application functionality that is not a simple
reexport of those namespaces:

- `SpkKernel` owns kernel bytes, indexes summaries, and resolves body chains;
- `SpkSegment` evaluates the supported segment representations;
- `SpiceEphemerisProvider` adapts kernel queries to the service provider
  boundary;
- `SpiceError` reports kernel/index/evaluation failures.

Callers that only need the low-level DAF/SPK parser should import Siderust
directly.

## Supported SPK segment types

| Type | Description | Status |
| --- | --- | --- |
| 2 | Chebyshev position | supported |
| 3 | Chebyshev position and velocity | supported |

These types cover the JPL DE-series kernels used by the current validation
path.

## Unsupported segment types

Other SPK data types are indexed when possible but are not evaluated by the
current `spod::spice` implementation. A state query that resolves to an
unsupported segment returns:

```rust
spod::spice::SpiceError::UnsupportedDataType { data_type: 9 }
```

Types 9 and 13 remain intentionally deferred; see
[ADR-0007](../adr/0007-spk-type-coverage.md).

## Implementation notes

- `SpkKernel::open` currently reads the kernel bytes into memory.
- DAF metadata is parsed with `siderust::formats::spice::daf::Daf`.
- Body-relative states are resolved by walking the indexed target/center
  segment graph and summing the appropriate segment states.
- Epochs are TDB seconds past J2000, matching NAIF SPK conventions.
