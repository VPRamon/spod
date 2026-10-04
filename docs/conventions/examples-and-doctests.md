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

The Lambert module exposes its API directly through `spod::lambert`:

```rust
use siderust::affn::cartesian::Position;
use siderust::affn::frames::ICRS;
use siderust::qtty::unit::Kilometer;
use siderust::qtty::{GravitationalParameter, Second};
use spod::lambert::{lambert, LambertBranch};

let r1 = Position::<(), ICRS, Kilometer>::new(15_945.34, 0.0, 0.0);
let r2 = Position::<(), ICRS, Kilometer>::new(12_214.83899, 10_249.46731, 0.0);
let tof = Second::new(4_560.0);
let mu = GravitationalParameter::new(398_600.4418);

let solution = lambert(r1, r2, tof, mu, LambertBranch::Prograde).unwrap();
assert!(solution.v1.x().value().is_finite());
```

## Examples that need large files

When an example requires a large external kernel or observation file, use a
`no_run` fence and document the prerequisite:

```rust,no_run
use spod::spice::SpkKernel;

let kernel = SpkKernel::open("/data/spk/de441.bsp")?;
let state = kernel.state(399, 0, 0.0)?;
assert!(state[0].is_finite());
# Ok::<(), spod::spice::SpiceError>(())
```

`no_run` examples are still type-checked.

## Runnable examples

The current runnable examples are:

```bash
cargo run --example 03_lambert_earth_to_mars
cargo run --example 04_sgp4_from_tle
```

The larger service demonstration is the synthetic POD configuration:

```bash
cargo run --bin spod -- validate-config examples/configs/leo_gnss_mvp1.yaml
cargo run --bin spod -- run examples/configs/leo_gnss_mvp1.yaml
```

See [`examples/README.md`](../../examples/README.md) for the current example
inventory and conventions.
