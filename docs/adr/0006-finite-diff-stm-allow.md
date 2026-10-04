# ADR-0006 — `#[allow(deprecated)]` on `finite_diff_stm_series` in pod-service

## Status

Accepted (Phase 1)

## Context

The upstream `siderust` crate marks `finite_diff_stm_series` as deprecated in
favour of `variational::propagate_stm`. The deprecation note states that
`propagate_stm` is the preferred path for single-arc STM computation.

`siderust-pod-service/src/pipeline.rs` calls `finite_diff_stm_series` inside
the batch-LS assembly loop.

The reason `propagate_stm` cannot be used here is:

> `propagate_stm` returns the state-transition matrix Φ at the **terminal
> epoch only**. Batch-LS assembly requires Φ at **every measurement epoch**
> along the arc, which is exactly the use case the upstream deprecation note
> explicitly preserves the series form for.

## Decision

`siderust-pod-service/src/pipeline.rs` retains its call to
`finite_diff_stm_series` with a scoped `#[allow(deprecated)]` attribute and a
multi-line rationale comment explaining why `propagate_stm` is insufficient for
this call-site.

The allow is **not** applied crate-wide or module-wide; it is scoped to the
minimum necessary block.

## Consequences

- If upstream `siderust` eventually adds a per-epoch variant of `propagate_stm`
  (or exposes an iterator-based STM propagator), this call-site should be
  migrated and the allow removed.
- The allow is documented here so future maintainers understand the intent and
  know to revisit it when the upstream API evolves.
- `cargo clippy` does not produce a warning at this call-site because the
  `allow` is present; the CI `todo-sweep` script does not touch `deprecated`
  attrs.
