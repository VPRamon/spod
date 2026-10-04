# Product use cases

These use cases define the workflows `spod` should productize. They are deliberately written from the operator/customer perspective rather than from the module/API perspective.

Each use case separates:

- customer problem;
- inputs;
- `spod` responsibilities;
- Siderust responsibilities;
- outputs;
- operational quality criteria;
- MVP scope;
- related roadmap issues.

## Use-case index

| Use case | Core value | Tracking |
|---|---|---|
| [Operational onboard-GNSS POD](operational-gnss-pod.md) | precise LEO orbit products from tracking data | [#20](https://github.com/VPRamon/spod/issues/20) |
| [Fleet / constellation POD](fleet-constellation-pod.md) | repeatable multi-spacecraft operations | [#31](https://github.com/VPRamon/spod/issues/31) |
| [Independent orbit validation](independent-orbit-validation.md) | objective quality assessment of an orbit solution | [#24](https://github.com/VPRamon/spod/issues/24) |
| [Mission commissioning](mission-commissioning.md) | prove GNSS/POD chain health after launch/change | [#30](https://github.com/VPRamon/spod/issues/30) |
| [Historical reprocessing](historical-reprocessing.md) | reproducible processing baselines over long periods | [#25](https://github.com/VPRamon/spod/issues/25) |
| [Rapid/final latency classes](latency-classes.md) | explicit timeliness/quality operating modes | [#26](https://github.com/VPRamon/spod/issues/26) |

## Shared foundation

All use cases should rely on the same platform capabilities:

- [#21](https://github.com/VPRamon/spod/issues/21) dataset resolution;
- [#22](https://github.com/VPRamon/spod/issues/22) mission profiles;
- [#23](https://github.com/VPRamon/spod/issues/23) operational QC;
- [#27](https://github.com/VPRamon/spod/issues/27) observability;
- [#28](https://github.com/VPRamon/spod/issues/28) product/artifact API.

Scientific primitives should be consumed from Siderust under the [feature parity policy](../feature-parity-policy.md).