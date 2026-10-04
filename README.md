# spod

[![CI](https://github.com/VPRamon/spod/actions/workflows/ci.yml/badge.svg)](https://github.com/VPRamon/spod/actions/workflows/ci.yml)
[![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)

**A service application for Siderust POD in Rust.**

`spod` is an engineering-oriented service for configuration, orchestration, artifact handling, and interfaces around the reusable POD APIs in [Siderust](https://github.com/Siderust). It does not maintain a second scientific POD implementation.

The current MSRV is Rust 1.89, matching the dependency graph and the
Docker builder image.

> **Status: engineering preview (pre-1.0).**
> The synthetic end-to-end POD path is usable for development and validation. Real-data workflows and the REST surface are still evolving. This project is not yet intended for flight-critical or safety-critical operational use.

## What is implemented

| Area | Current scope |
| --- | --- |
| POD science | Provided by `siderust::pod` (forces, propagation, observations, estimation, QC and products) |
| Orbit mechanics | Lambert solver, TLE/3LE/OMM handling, SGP4/SDP4 propagation, SPICE ephemerides |
| Formats | SP3, RINEX, ANTEX, EOP, CRD, CPF and CCSDS OEM support at the currently implemented subsets |
| Products & QC | Residuals, orbit products, manifests, comparison/QC utilities |
| Interfaces | Rust library, command-line interface, experimental Axum REST API |

The project separates service orchestration, external interfaces, persistence and format adapters from the reusable POD implementation in Siderust.

## Repository structure

```text
src/
  io/            Space/geodesy format readers and writers
  service/       Configuration, orchestration, REST/CLI and artifact handling
  lambert/       Lambert solver
  sgp4/          SGP4 integration
  spice/         SPICE helpers/providers
  tle/           TLE/3LE/OMM handling
  bin/           CLI and experimental REST entry points
```

## Quick start

```bash
git clone https://github.com/VPRamon/spod.git
cd spod
cargo test
```

## Relationship with the Siderust ecosystem

Foundational astrodynamics, typed quantities, time scales, frames, and reusable numerical mechanics live in the released Siderust ecosystem crates. `spod` is built on top of `siderust::pod`; it does not expose a scientific compatibility facade. Callers needing reusable force, observation, estimation, QC, or product APIs should import Siderust directly.

Validate and run the synthetic POD configuration:

```bash
cargo run --bin spod -- validate-config examples/configs/leo_gnss_mvp1.yaml
cargo run --bin spod -- run examples/configs/leo_gnss_mvp1.yaml
```

Run the experimental REST service in a container:

```bash
docker build -t spod:dev .
docker run --rm -p 8080:8080 spod:dev
```

The service listens on `SPOD_REST_BIND` and writes job output below
`SPOD_REST_OUT`.

The remaining Rust examples cover Lambert transfer and SGP4 integration:

```bash
cargo run --example 03_lambert_earth_to_mars
cargo run --example 04_sgp4_from_tle
```

See [examples/README.md](examples/README.md) for the purpose of each example and the typed-API conventions they follow.

## Design principles

- **Type safety for physical quantities.** Units, epochs, frames and states should be difficult to mix accidentally.
- **Traceable numerical behaviour.** Scientific algorithms should have explicit assumptions and validation tests.
- **Separation of concerns.** Dynamics, observations, estimation, formats and orchestration remain independently testable.
- **No unsafe Rust.** The library forbids `unsafe_code`.
- **Standards-oriented interoperability.** Common astrodynamics and geodesy formats are treated as first-class interfaces.

## Validation

The CI pipeline checks formatting, Clippy, tests across feature combinations, documentation, dependency direction, and source hygiene.

Run the main checks locally with:

```bash
cargo fmt -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-fail-fast
cargo test --workspace --no-default-features --no-fail-fast
cargo test --workspace --all-features --no-fail-fast
cargo test --doc --workspace --all-features
cargo doc --workspace --no-deps
bash scripts/check_dep_graph.sh
bash scripts/check_no_todos.sh
bash scripts/check_legacy_name_refs.sh
```

Build and exercise the container locally:

```bash
docker build -t spod:dev .
docker run --rm --entrypoint spod spod:dev --help
docker run --rm -p 8080:8080 spod:dev
```

The REST container defaults to `0.0.0.0:8080`, so the published port is
reachable from outside the container. Set `SPOD_REST_BIND` and
`SPOD_REST_OUT` to override the defaults.

## Contributing

Contributions, validation cases and format-compatibility reports are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

For security issues, see [SECURITY.md](SECURITY.md). For support and commercial-licensing enquiries, see [SUPPORT.md](SUPPORT.md).

## License

`spod` is available under **AGPL-3.0-or-later**. See [LICENSE](LICENSE).

Commercial licensing can be discussed separately; see [SUPPORT.md](SUPPORT.md).
