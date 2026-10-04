//! # spod observation models
//!
//! ## Scientific scope
//!
//! This crate houses the measurement equations that connect estimated
//! spacecraft states to tracked observables.  The current scientific focus
//! covers:
//!
//! - **GNSS** pseudorange and carrier-phase (code + phase with Saastamoinen
//!   troposphere and Klobuchar ionosphere).
//! - **SLR** normal-point two-way range (Marini-Murray troposphere, Shapiro
//!   relativistic delay).
//! - **LISA** inter-satellite range (feature-gated; light-time-iterated range
//!   from ESA OEM ephemerides).
//! - **Generic** typed `Observation` trait with object-safe batching, a
//!   `ProviderBundle` abstraction for clock/atmosphere providers, and a
//!   `CorrectionRegistry` pipeline (PCO, Shapiro, EarthTide).
//!
//! ## Technical scope
//!
//! The crate re-exports the generic `MeasurementModel` abstraction together
//! with GNSS and SLR observation/model structs and the frame-transform
//! provider trait.  These APIs return scalar predictions plus partial
//! derivatives, ready for assembly by the estimation layer.
//!
//! Parsing of raw data files and orchestration of estimation loops are
//! outside this crate.
//!
//! ## Crate features
//!
//! | Feature | Adds |
//! |---------|------|
//! | `lisa`  | `inter_sat::InterSatRangeObs`, requires `spod::io` |
//!
//! ## References
//!
//! - Misra, P., & Enge, P. (2012). *Global Positioning System: Signals,
//!   Measurements, and Performance* (2nd ed.). Ganga-Jamuna Press.
//! - Tapley, B. D., Schutz, B. E., & Born, G. H. (2004). *Statistical Orbit
//!   Determination*. Elsevier Academic Press.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ── New typed observation layer ───────────────────────────────────────────────

pub mod batch;
pub mod corrections;
pub mod error;
pub mod gnss_obs;
pub mod inter_sat;
pub mod obs_trait;
pub mod provider_bundle;
pub mod slr_obs;

pub use batch::ObservationBatch;
pub use corrections::{
    Correction, CorrectionRegistry, EarthTideDisplacement, PhaseCenterOffset, ShapiroDelay,
};
pub use error::PodObservationsError;
pub use gnss_obs::{
    GnssCarrierPhaseObs, GnssPseudorangeObs, IonoModel, KlobucharParams, TropModel,
};
pub use obs_trait::{
    AnyObservation, CartesianState, ObsResidual, ObsType, Observation, PhaseResidual,
};
pub use provider_bundle::{NullProviderBundle, ProviderBundle};
pub use slr_obs::SlrNormalPointObs;

#[cfg(feature = "lisa")]
pub use inter_sat::InterSatRangeObs;

// ── Legacy MVP observation layer (keep unchanged) ─────────────────────────────

pub mod frame_transform;
pub mod gnss;
pub mod model;
pub mod slr;

pub use frame_transform::{FrameTransformError, FrameTransformProvider};
pub use gnss::{CarrierPhaseObs, GnssCarrierModel, GnssCodeModel, PseudorangeObs};
pub use model::{MeasurementModel, Partials, Prediction};
pub use slr::{SlrRangeModel, SlrRangeObs};
