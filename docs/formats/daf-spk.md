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

`spod` does not add a SPICE compatibility namespace or maintain a second
kernel implementation. Service code that needs SPICE should import the
canonical Siderust APIs directly.
