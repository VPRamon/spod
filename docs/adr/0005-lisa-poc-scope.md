# ADR-0005 — LISA POC scope: heliocentric three-spacecraft + ranging only

## Status

Accepted (2026-05-12, per user decision)

## Context

The LISA mission (Laser Interferometer Space Antenna) is a space-based
gravitational-wave detector consisting of three spacecraft in a heliocentric
orbit, exchanging laser beams. Full LISA data processing involves:

1. **Heliocentric orbit propagation** of three spacecraft.
2. **Inter-spacecraft ranging** (MOSA metrology — distance measurements).
3. **Time-delay interferometry (TDI)** — combining multiple laser links to
   cancel laser frequency noise.
4. **Laser metrology stack** — optical bench, gravitational reference sensor,
   test-mass dynamics.

A POD proof-of-concept using real LISA orbit data (from
https://github.com/esa/lisa-orbit-files) was proposed as E2E-12.

The scope question: should the POC cover the full LISA instrument stack, or
only the orbital mechanics layer?

## Decision

The LISA POC (`E2E-12`) covers **only**:

1. Heliocentric orbit propagation of three spacecraft using `siderust::pod::propagation`.
2. Simulated MOSA inter-spacecraft range measurements (`InterSatRange` observation
   model in `siderust::pod::observation`).
3. A batch-LS or EKF POD run that estimates the three spacecraft states from the
   range measurements, using orbits from `esa/lisa-orbit-files` as truth
   reference.

**Out of scope** for the POC (and for the entire 0.x series of spod):

- Time-delay interferometry (TDI) combination.
- Laser metrology stack.
- Optical bench / gravitational reference sensor dynamics.
- Laser frequency noise characterisation.

## Rationale

TDI and the laser metrology stack are specialised signal-processing algorithms,
not orbit-determination algorithms. They belong in a dedicated
`siderust-lisa-metrology` crate (not yet planned) that composes with
`spod` outputs, not inside the POD pipeline itself.

The orbit-estimation layer (propagation + range-measurement model + estimator)
exercises the full POD pipeline with a real science-mission dataset and is
sufficient to validate `spod` at the FocusPOD M6 milestone.

## Consequences

- `siderust::pod::observation` implements `InterSatRange` as a first-class
  observation model alongside GNSS pseudorange and SLR normal points.
- The LISA POC is implemented as a **configuration** of existing primitives
  (a `LisaEphemerisProvider` reading `esa/lisa-orbit-files` and an
  `InterSatRange` model), not a bespoke pipeline.
- The POC does not require pulling an HDF5 crate if the lisa-orbit-files
  are in text / ASCII format; check the actual format before adding deps.
