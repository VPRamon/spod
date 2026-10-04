# Feature parity policy

## Purpose

Architectural consolidation should reduce code ownership without accidentally shrinking the product. This policy applies whenever a local implementation is removed, replaced, or moved upstream.

Tracking: [#19](https://github.com/VPRamon/spod/issues/19).

> **Remove duplicated implementation, not product capability.**

## Three layers

### Implementation

A local file such as `src/io/sp3.rs` can disappear when Siderust provides the canonical parser.

### Rust API compatibility

An old namespace such as `spod::io::sp3::*` does not have to remain. Broad compatibility reexports are discouraged because they recreate ambiguous ownership.

### Product capability

A supported workflow being able to consume SP3 may still be a required product capability even after both the implementation and old namespace disappear.

## Required classifications

### KEEP IN SPOD

Operational/application behavior such as job lifecycle, input resolution, mission profiles, processing baselines, QC thresholds, REST/CLI, and reprocessing.

### MIGRATE TO SIDERUST

The feature remains available to workflows, but the reusable implementation is canonical upstream. Typical examples are standard format parsing, observation models, estimators, and product serialization.

### UPSTREAM GAP FIRST

The product needs the capability but upstream does not yet provide equivalent semantics. The local implementation must not be deleted or silently degraded until the gap is resolved.

Full Vallado-style SGP4/SDP4 is the current example; see [#29](https://github.com/VPRamon/spod/issues/29).

### INTENTIONALLY RETIRE

The capability does not belong in the target product and is deliberately removed. This requires rationale, impact assessment, changelog/documentation, and confirmation that no supported workflow depends on it.

Lambert may become an example of this category, but that decision must be explicit.

## Migration checklist

Every deletion/replacement PR should answer:

1. What user-visible capability existed?
2. Is it still part of the product?
3. Where will the implementation live?
4. Is the replacement semantically equivalent for the supported workflow?
5. Which product-level test proves the workflow still works?
6. Which algorithm/parser tests can move upstream?
7. Are dependencies/features removed only after the last product consumer migrates?
8. Does documentation describe ownership correctly?

## Testing policy

Tests that may move upstream include parser conformance, numerical verification, estimator kernels, SPICE segment evaluation, generic format round-trips, and generic SGP4 reference cases once upstream owns that backend.

Tests that stay in `spod` include real workflow ingestion, configuration mapping, provenance, artifact generation, QC policy, unsupported-combination failures, CLI/REST parity, and end-to-end reference datasets.

## No silent downgrade

Compiling against Siderust is not enough. Differences in model, format subset, interpolation, physical corrections, deep-space support, or numerical precision must be evaluated against product requirements.

## Review gate

For #9-#13 and future consolidation work, unclassified capability loss is a blocker.

The desired result is:

```text
less duplicated code
+ same or better product capability
+ clearer ownership
+ stronger operational tests
```