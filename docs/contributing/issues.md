# Issue and use-case guidelines

GitHub issues in `spod` are part of the product-development system, not only a task queue.

The repository provides four issue forms:

| Template | Use it for |
|---|---|
| Bug | Incorrect behavior or regression in an existing supported capability |
| Feature | A new or expanded product capability |
| Documentation | Missing, stale, misleading, or unverifiable documentation |
| Product use case | An end-to-end user outcome that acts as product documentation and a verification contract |

## Product boundary

When opening or refining an issue, preserve the architectural distinction:

### Siderust

Owns reusable scientific and numerical functionality, including standard-format parsing, astrodynamics, observation models, estimation primitives, and reusable scientific QC.

### spod

Owns product workflows, orchestration, mission configuration, operational policy, artifacts, provenance, CLI/REST interfaces, deployment, observability, and product-level verification.

A feature request should not introduce a second implementation in `spod` when canonical reusable science belongs in Siderust.

If `spod` requires a capability that Siderust does not yet provide with the required semantics, classify it as an **upstream gap** and preserve the product requirement explicitly.

## Use cases as living specifications

A product use case should survive implementation.

The issue describes:

```text
user/problem
    |
    v
inputs + trigger
    |
    v
documented workflow
    |
    +---- Siderust scientific responsibilities
    |
    +---- spod product responsibilities
    |
    v
outputs + artifacts
    |
    v
quality gates
    |
    v
verification evidence
```

This makes the issue useful in three phases:

1. **Discovery/documentation** — explains why the capability exists and its boundaries.
2. **Implementation** — provides the functional contract and acceptance criteria.
3. **Verification/maintenance** — records how the behavior is proven and what future changes must preserve.

## Acceptance criteria

Good acceptance criteria describe externally observable outcomes.

Prefer:

```text
- [ ] Every consumed input is represented in run provenance.
- [ ] An unsupported force model returns an explicit configuration error.
- [ ] CLI and REST execute the same service workflow.
```

Avoid criteria that only prescribe internal implementation:

```text
- [ ] Create struct Foo.
- [ ] Add function bar().
```

Internal design may change while the product contract remains stable.

## Verification evidence

Issues should identify how the behavior will be proven. Depending on the capability, that may include:

- unit tests;
- integration tests;
- end-to-end workflows;
- reference datasets;
- numerical tolerances;
- generated artifacts and hashes;
- CLI commands;
- REST exchanges;
- QC thresholds;
- Docker/service smoke tests;
- documentation examples exercised by CI.

For scientific workflows, a green process exit is not sufficient evidence of product correctness.

## Closing an issue

Before closing a feature or use case, reviewers should be able to answer:

- Does the implementation satisfy the documented user outcome?
- Are inputs, outputs, failure modes, and quality gates represented correctly?
- Is reusable scientific functionality in the correct upstream layer?
- Is product-level verification present?
- Does current user-facing documentation match the implemented behavior?
- Would a future contributor understand what must not regress?

A closed use-case issue should therefore remain useful as historical and functional product documentation.
