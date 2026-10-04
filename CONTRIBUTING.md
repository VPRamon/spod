# Contributing

Thanks for contributing to `spod`.

## Development setup

`spod` builds against released crates from the Siderust ecosystem. No sibling repository checkouts are required.

A standard development setup is enough:

```bash
git clone https://github.com/VPRamon/spod.git
cd spod
cargo test
```

The main foundational dependencies are released versions of `siderust`, `principia`, `qtty`, `tempoch`, `affn`, and `cheby`.

If a change requires unreleased functionality from one of those projects, do not add a local path or git dependency as a permanent workaround. Prefer opening or referencing the corresponding upstream change and keep this repository on released dependencies.

## Before opening a pull request

Run the same checks expected by CI:

```bash
cargo fmt -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-fail-fast
cargo test --workspace --no-default-features --no-fail-fast
cargo test --workspace --all-features --no-fail-fast
cargo doc --workspace --no-deps
bash scripts/check_dep_graph.sh
bash scripts/check_no_todos.sh
```

Changes to dependency declarations should also preserve the standalone build: a fresh clone must not require neighbouring Siderust repositories, local `[patch]` overrides, or git dependency workarounds.

## Scientific and numerical changes

Changes to orbital dynamics, estimation, reference-frame transformations, time handling, or file-format semantics should include:

- a clear statement of assumptions and validity range;
- tests against an independent reference, published example, or frozen fixture where practical;
- units and reference frames made explicit at API boundaries;
- a note in the pull request explaining any expected numerical tolerance.

Avoid changing physical constants, time-scale semantics, frame conventions, estimator behaviour, or numerical tolerances as incidental cleanup.

## Dependency boundaries

Foundational astrodynamics and reusable mechanics belong in the upstream Siderust ecosystem when they are broadly applicable. `spod` should remain focused on precise orbit determination concerns such as observations, estimation, orbit products, quality control, and service orchestration.

When functionality overlaps with upstream crates, prefer their released public APIs rather than maintaining duplicate foundational implementations here, provided POD-specific behaviour is preserved.

## API changes

This project is pre-1.0, so APIs can evolve. Still, prefer small, reviewable changes and document user-visible behaviour changes.

## Security

Do not disclose vulnerabilities in public issues or pull requests. Follow [SECURITY.md](SECURITY.md).
