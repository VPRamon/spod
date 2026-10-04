# ADR-0002 — Unified per-crate `thiserror` Error enums; no `anyhow`

## Status

Accepted (Phase 1, enforced as workspace lint)

## Context

Early scaffolding in several crates used `anyhow::Error` as a catch-all return
type. `anyhow` is an ergonomic choice for application-layer code where callers
only need to print or propagate opaque errors. It is a poor choice for library
crates because:

1. **Caller cannot match on variants.** Library users need to inspect errors to
   decide whether to retry, fall back, or surface diagnostics to their own
   callers. `anyhow::Error` erases that information.
2. **Documentation.** `thiserror`-derived enums appear in `rustdoc` with their
   variant list, making the error contract part of the public API and
   searchable.
3. **`cargo-public-api` snapshots.** A `thiserror` enum is a stable, snapshotted
   API surface. `anyhow::Error` is not.
4. **Composability.** Downstream crates (e.g., `spod-service`) compose
   errors from multiple crates; typed enums allow clean `From` impls or
   `?`-compatible conversions.

## Decision

Every library crate in `spod` defines exactly **one** error type per
crate, named `<CratePrefix>Error` (e.g., `PodIoError`, `LambertError`). It is:

- Derived with `thiserror::Error`.
- Re-exported from the crate's `lib.rs` at the crate root.
- The **only** error type that appears on public API `Result<_, E>` signatures.
- `#[non_exhaustive]` where future variants are expected.

`anyhow` must not appear in any library crate dependency. It may be used in
test binaries and the CLI binary where ergonomic error printing is the only
requirement.

## Consequences

- Library callers get exhaustive match coverage and can inspect error details.
- Error types are versioned through `api.snapshot` CI gate.
- Minor code overhead vs `anyhow` in library internals: internal helpers use
  `Result<_, <CratePrefix>Error>` throughout.
- The workspace `[workspace.lints]` does not currently auto-ban `anyhow` in
  libraries; authors rely on code-review and the `api.snapshot` diff to catch
  regressions.
