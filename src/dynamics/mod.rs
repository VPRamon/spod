// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Astrodynamics and POD-dynamics composition layer.
//!
//! This module merges the former `siderust-dynamics` (domain-agnostic
//! astrodynamics primitives) and `spod::dynamics` (POD-specific
//! force-model composition) into a unified surface.
//!
//! ## Contents
//!
//! - [`integrators`] — uniform [`Integrator`] trait over RK4/DOPRI5/DOP853
//! - [`thrust`] — finite-burn thrust-arc physical model
//! - [`low_thrust`] — Tsiolkovsky ΔV bookkeeping
//! - [`validation`] — STM finite-difference validation harness
//! - [`empirical_periodic`] — 1-CPR / 2-CPR periodic empirical accelerations
//! - [`force_config`] — configurable force-model spec
//! - [`forces`] — force-model implementations
//! - [`integrator_adapter`] — typed integrator adapters
//! - [`process_noise`] — Q-matrix construction for EKF
//! - [`registry`] — named force-model registry
//! - [`thrust_arc`] — thrust-arc parameter declaration
//! - [`variational`] — variational/STM propagator

pub mod empirical_periodic;
pub mod error;
pub mod force_config;
pub mod forces;
pub mod integrator_adapter;
pub mod integrators;
pub mod low_thrust;
pub mod pod_error;
pub mod process_noise;
pub mod registry;
pub mod thrust;
pub mod thrust_arc;
pub mod validation;
pub mod variational;

// Re-exports from the low-level layer (former siderust-dynamics public surface)
pub use error::DynamicsError;
pub use integrators::{Dop853Integrator, Dopri5Integrator, Integrator, Rk4Integrator};
pub use low_thrust::{LowThrustLog, LowThrustRecord};
pub use thrust::{mass_flow_rate, thrust_acceleration, ManeuverError, ThrustArc, G0_M_PER_S2};

// Re-exports from the POD dynamics layer (former spod::dynamics public surface)
pub use empirical_periodic::{EmpiricalPeriodicAcceleration, PeriodicHarmonic};
pub use force_config::ForceModelConfig;
pub use forces::{
    AccelPartials, Acceleration3, CartesianState, DragForce, Epoch, ForceModel,
    ForceModelRegistry as EvaluatingForceModelRegistry, J2PerturbationForce,
    SolarRadiationPressureForce, TwoBodyForce,
};
pub use integrator_adapter::{propagate_orbit, propagate_spacecraft};
pub use pod_error::PodDynamicsError;
pub use process_noise::{
    GaussMarkovParams, PiecewiseSegment, ProcessNoise, ProcessNoiseModel, WhiteAccelPsd,
};
pub use registry::{ForceModelFactory, ForceModelParams, ForceModelRegistry, ForceModelSpec};
pub use thrust_arc::ThrustArcConfig;
pub use variational::{
    param_partials_central_diff, ParamColumn, ParamStmReport, PropagatedArc, VariationalPropagator,
};

// Convenience re-exports from the upstream canonical siderust types
#[doc(no_inline)]
pub use siderust::astro::dynamics::{
    forces::{TwoBody, J2},
    DynamicsContext, OrbitState, Position, Propagator, PropagatorConfig, StateTransitionMatrix,
    Velocity,
};
pub use siderust::pod::force::{
    SiderustAccelerationModel as SiderustForceModel, SiderustCompositeModel as CompositeForce,
};
pub use siderust::principia::propagate_stm;
