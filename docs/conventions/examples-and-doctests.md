# Examples and Doctests

Every public item in `spod` carries an executable example. This is
non-negotiable: the `missing_docs = deny` lint already requires *some*
documentation; this convention says that documentation must include
working code.

## The rule

> Every `pub` type, function, trait, method, and constant has a
> `# Examples` section in its rustdoc, and that example compiles under
> `cargo test --doc`.

This is enforced by:

1. `#![deny(missing_docs)]` (workspace lint).
2. `#![deny(rustdoc::broken_intra_doc_links)]` (workspace lint).
3. CI step `cargo test --workspace --doc`.
4. PR review: a reviewer rejects a PR that adds undocumented public items
   even if it compiles.

## The shape of a good doctest

```rust
/// Compute the Lambert transfer between two heliocentric positions.
///
/// # Examples
///
/// ```
/// use qtty::si::Length;
/// use siderust_lambert::{lambert, TransferGeometry};
///
/// let r1 = Length::from_metres(1.495_978_707e11);  // 1 AU
/// let r2 = Length::from_metres(2.279_392_4e11);    // ~Mars distance
/// let tof = std::time::Duration::from_secs(259 * 86_400);
///
/// let solution = lambert(r1, r2, tof, TransferGeometry::Prograde)
///     .expect("feasible transfer");
/// assert!(solution.v1.norm().value() > 0.0);
/// ```
pub fn lambert(/* ... */) -> Result<Solution, LambertError> { /* ... */ }
```

Notes:

- The example **uses the typed APIs**. It does not strip down to bare
  `f64` for the sake of brevity.
- The example **runs**. Don't fake outputs with comments — assert.
- The example demonstrates the *intended* call site, not a debugging path.

## When the example needs network access or large files

Some examples genuinely require multi-megabyte SP3 files, the DE441 SPK
kernel, or a network round-trip. In those cases:

1. Mark the fence with `no_run`:

   ```rust
   /// # Examples
   ///
   /// ```no_run
   /// # use siderust_spice::SpkKernel;
   /// let kernel = SpkKernel::open("/data/spk/de441.bsp")?;
   /// // ... use the kernel ...
   /// # Ok::<(), siderust_spice::SpiceError>(())
   /// ```
   ```

2. Add a `# Notes` section explaining what the example *would* do and
   how to obtain the inputs:

   ```text
   /// # Notes
   ///
   /// This example requires the JPL DE441 ephemeris kernel
   /// (`de441.bsp`, ~3 GB). Download from
   /// <https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/>.
   ```

3. Prefer `compile_fail` for examples whose entire point is type-safety
   ("`Position + Position` must not compile").

`no_run` is still type-checked, so the typed signatures are still verified.

## The `examples/` directory

Workspace-level runnable examples live in `examples/` at the workspace
root and are invoked with:

```bash
cargo run --example 01_minimal_leo_run
```

Each example file maps to a milestone or scenario. The mapping is
maintained in [`examples/README.md`](../../examples/README.md), which
also lists fixture requirements and expected runtime per example.
