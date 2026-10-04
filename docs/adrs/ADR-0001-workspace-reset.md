# ADR-0001 — Workspace reset

## Status
Accepted.

> Historical record: this decision predates the rename to `spod`. At the time,
> the repository and planned packages were named `siderust-pod`.

## Context
`siderust-pod/` originally contained a verbatim clone of the `siderust`
crate, with a single `[package] name = "siderust"` declaration. This caused:
- name collision with the upstream crate,
- silent divergence risk if the cloned source were ever edited,
- no place for POD domain code to live.

## Decision
Wipe `siderust-pod/`'s old `src/`, `build.rs`, vendored `siderust-ffi/`, and
upstream-shaped tests/examples/benches. Replace the root `Cargo.toml` with a
virtual workspace whose members are the new POD crates under `crates/*`.

## Consequences
- Upstream `siderust` is now consumed as a normal `path` dependency.
- POD code has a clean home; `qtty`, `tempoch`, `affn`, `cheby`, `siderust`
  remain unmodified.
