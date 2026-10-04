# ADR-0002 — Typed library errors; `anyhow` only at application boundaries

## Status

Accepted.

## Context

`anyhow::Error` is useful in binaries where the consumer primarily needs a
human-readable diagnostic. It is less suitable for reusable library APIs,
where callers need to match on structured failure modes.

The current `spod` repository is a single package with library modules plus
CLI/REST entry points, while reusable POD science comes from Siderust.

## Decision

- Reusable `spod` library modules expose structured error enums, normally
  derived with `thiserror::Error`.
- Errors should preserve useful fields and wrap lower-level typed errors rather
  than flattening them to strings when a typed boundary is available.
- `anyhow` is acceptable in the `spod` and `spod-rest` binary entry points,
  where errors terminate at a human/operator-facing application boundary.
- Service orchestration may translate external errors into its own structured
  service error where that adds meaningful context.

## Consequences

- Downstream Rust callers can inspect library failures programmatically.
- Rustdoc exposes the error contract of reusable APIs.
- CLI/REST startup and top-level command handling retain ergonomic contextual
  error reporting without turning `anyhow::Error` into a scientific/library
  API surface.
