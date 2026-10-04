# Static vs dynamic dispatch

- **Static / generic dispatch** is used in inner loops:
  - integrator step (`rk4_step` is generic over `F: ForceModel`),
  - residual evaluation,
  - per-step force evaluation through `CompositeForce`.

- **Dynamic dispatch** (`Box<dyn ForceModel>`, `Box<dyn MeasurementModel>`)
  is used at *configuration assembly* time inside `spod-service`.
  The boxed objects are converted into a concrete composite type before
  entering the integration / estimation kernels, so the inner loops see a
  monomorphised call.

- **Rationale:** generic monomorphisation gives competitive performance
  on the hot path; the dynamic seam at the top of the pipeline keeps
  configuration ergonomic and avoids a combinatorial explosion of
  generic instantiations exposed in the public API.
