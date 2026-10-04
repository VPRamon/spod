# Examples and Doctests

Public APIs should have useful rustdoc examples when an example materially
clarifies the intended call site. Examples must use the current `spod` and
Siderust namespaces and should compile under the repository doctest suite.

## The rule

For public APIs where a usage example is useful:

1. Prefer a `# Examples` section in rustdoc.
2. Use typed Siderust/qtty APIs at scientific boundaries.
3. Keep runnable examples deterministic and independent of network access.
4. Use `no_run` only when a real external dataset is required.
5. Keep examples aligned with the current public API rather than preserving
   old compatibility namespaces.

CI runs:

```bash
cargo test --doc --workspace --all-features
```

and builds documentation with broken intra-doc links denied.

## Typed example

The service demonstration is the synthetic POD configuration:

```bash
cargo run --bin spod -- validate-config examples/configs/leo_gnss_mvp1.yaml
cargo run --bin spod -- run examples/configs/leo_gnss_mvp1.yaml
```

See [`examples/README.md`](../../examples/README.md) for the current example
inventory and conventions.
