# Personas and buyers

## LEO / Earth-observation operator

**Needs:** routine precise orbit products, predictable latency, reproducibility, low operator intervention, and explicit QC.

**Pain:** maintaining a full POD stack is specialized; input versions and ad-hoc scripts make reproducibility and support difficult.

**Buying trigger:** first operational mission, scaling beyond one spacecraft, or replacing fragile internal processing.

## Ground-segment integrator

**Needs:** a deployable POD component with stable APIs, versioned configuration, deterministic artifacts, and clear errors.

**Pain:** every mission otherwise becomes a custom integration around scientific libraries.

**Buying trigger:** standardizing multiple ground segments or needing an on-prem/containerized supported product.

## Flight-dynamics / POD engineer

**Needs:** diagnostics, mission profiles, baseline comparisons, residual inspection, and controlled overrides.

**Pain:** too much glue code and poor traceability between data, configuration, and outputs.

**Role:** technical champion and primary scientific user.

## Mission commissioning team

**Needs:** early assessment of GNSS availability, time tags, clock behavior, residuals, convergence, and comparison against external/onboard solutions.

**Pain:** commissioning evidence is often assembled manually and problems are hard to localize.

## Constellation operations team

**Needs:** controlled processing across dozens/hundreds of spacecraft, independent retries, fleet QC, and safe baseline rollouts.

**Pain:** per-spacecraft scripts and configuration drift do not scale.

## Science / reprocessing team

**Needs:** reproducible long-period reprocessing, processing baselines, restartable campaigns, and old/new baseline comparison.

**Pain:** algorithms and reference products evolve, but historical processing is difficult to reproduce.

## Platform / SRE team

**Needs:** containers, structured logs, metrics, health endpoints, failure categories, and predictable resource behavior.

**Pain:** scientific applications often expose only process exit codes and unstructured logs.

## Buying-center summary

| Persona | Primary concern | Likely role |
|---|---|---|
| Mission/operator lead | reliable orbit products | economic buyer |
| Ground-segment integrator | integration/deployment | buyer / partner |
| Flight-dynamics engineer | scientific quality | technical champion |
| Commissioning engineer | early mission validation | user/champion |
| Constellation ops | scaling/reliability | buyer/user |
| Science processing | reproducibility | user/buyer |
| SRE/platform | operability | approver/user |

A commercially credible `spod` must satisfy both the scientific champion and the operational/platform buyer.