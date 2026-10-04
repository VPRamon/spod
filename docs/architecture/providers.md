# Provider traits

> **Note:** The `spod::core` crate has been eliminated. See
> [ADR-0004](../adrs/ADR-0004-provider-traits.md) for the rationale.
> `FrameTransformProvider` (the only trait that was ever a real trait)
> now lives in `spod::dynamics`. POD crates import other provider
> types (ephemerides, EOP, gravity, atmosphere) directly from `siderust`.

`FrameTransformProvider` in `spod::dynamics::frame_transform`
is the seam for ITRF ↔ GCRF frame rotations inside POD. All other
`siderust` provider types are used directly by each POD crate as needed.
