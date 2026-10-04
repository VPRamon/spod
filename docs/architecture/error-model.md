# Error Model

`spod` follows a single, uniform convention for fallible APIs:
**one `thiserror`-backed `Error` enum per crate**, re-exported from
`lib.rs`.

## Rules

1. Every library crate defines its error type in a private `error` module:

   ```rust
   // src/error.rs
   #[derive(Debug, thiserror::Error)]
   pub enum Error {
       #[error("invalid SP3 header at line {line}: {reason}")]
       InvalidHeader { line: usize, reason: String },

       #[error("io error: {0}")]
       Io(#[from] std::io::Error),
       // ...
   }
   ```

2. The crate root re-exports it:

   ```rust
   // src/lib.rs
   mod error;
   pub use error::Error;
   ```

3. **No `anyhow` in library crates.** `anyhow::Error` erases the variant,
   making programmatic error handling impossible for downstream callers.
   `anyhow` is permitted *only* in the binary crates `spod` and
   `spod-rest`, where the consumer is a human reading a terminal or
   an HTTP error body.

4. **Variants must be structured.** A single `String` payload is forbidden
   when the error context contains discrete fields (line number, satellite
   PRN, frame name, parameter index, …). Structured variants enable
   localisation, machine-readable logs, and pattern matching.

5. **Wrap, don’t flatten.** When a lower-layer crate’s error needs to be
   surfaced, wrap it as a `#[from]` variant. Never re-stringify it.

6. The crate-level error must be `Send + Sync + 'static`. This is satisfied
   by `thiserror`'s default codegen.

7. See ADR-0002 in this directory series ([`docs/adr/0002-no-anyhow.md`](../adr/0002-no-anyhow.md))
   for the full rationale.

## Per-crate error type registry

| Crate                          | Error type (re-exported)                  | `anyhow` allowed? |
|--------------------------------|-------------------------------------------|-------------------|
| `siderust-dynamics`            | `siderust_dynamics::Error`                | no                |
| `siderust-lambert`             | `siderust_lambert::LambertError`          | no                |
| `siderust-sgp4`                | `siderust_sgp4::Error`                    | no                |
| `siderust-spice`               | `siderust_spice::SpiceError`              | no                |
| `siderust-tle`                 | `siderust_tle::TleError`                  | no                |
| `spod::core`            | `spod_core::Error`                | no                |
| `spod::dynamics`        | `spod_dynamics::Error`            | no                |
| `spod::io`              | `spod::io::Error`                  | no                |
| `spod::observations`    | `spod_observations::Error`        | no                |
| `spod::estimation`      | `spod_estimation::EstimationError`| no                |
| `spod::qc`              | `spod_qc::Error`                  | no                |
| `spod::products`        | `spod_products::Error`            | no                |
| `spod::service`         | `spod_service::PipelineError`     | no                |
| `spod`             | n/a (binary)                              | **yes**           |
| `spod-rest`            | n/a (binary)                              | **yes**           |

## Example

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown SPK data type: {ty}")]
    UnknownDataType { ty: i32 },

    #[error("requested epoch {epoch} outside coverage [{start}, {end}]")]
    EpochOutOfRange {
        epoch: tempoch::JulianDate<tempoch::TDB>,
        start: tempoch::JulianDate<tempoch::TDB>,
        end: tempoch::JulianDate<tempoch::TDB>,
    },

    #[error("io error reading kernel: {0}")]
    Io(#[from] std::io::Error),
}
```

A downstream caller can then write:

```rust,ignore
match provider.position_at(epoch) {
    Err(siderust_spice::SpiceError::EpochOutOfRange { start, end, .. }) => {
        // domain-specific recovery
    }
    Err(other) => return Err(other.into()),
    Ok(p)  => use_position(p),
}
```

— which is impossible with `anyhow`.
