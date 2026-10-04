// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! Inter-satellite range observation model for the LISA mission.
//!
//! This module is compiled only when the `lisa` feature is enabled.
//!
//! ## Scientific scope
//!
//! LISA measures the distance between pairs of spacecraft using laser
//! interferometry.  The measurement equation for one range observable is:
//!
//! ```text
//! ρ_meas = |r_A(t_recv) − r_B(t_emit)| + ε
//! ```
//!
//! where `t_emit = t_recv − ρ/c` (one-way light-time, one Newton iteration).
//! No atmospheric corrections are applied (inter-satellite / deep space).
//!
//! Positions are provided by `LisaEphemerisProvider` via cubic Hermite
//! interpolation over ESA OEM ephemeris files.
//!
//! ## References
//!
//! - Danzmann, K. et al. (2017). *LISA: Laser Interferometer Space Antenna.*
//!   ESA/SRE(2017)1.  <https://arxiv.org/abs/1702.00786>

#[cfg(feature = "lisa")]
mod inner {
    use crate::core::providers::EphemerisProvider;
    use crate::io::lisa::{LisaEphemerisProvider, LisaSpacecraftId};
    use crate::observations::error::PodObservationsError;
    use crate::observations::obs_trait::{CartesianState, ObsType, Observation};
    use crate::observations::provider_bundle::ProviderBundle;
    use siderust::time::JulianDate;
    use std::sync::Arc;

    /// Speed of light in km/s.
    const C_KM_S: f64 = 299_792.458;

    /// J2000.0 Julian date (TT ≈ TDB at the sub-ms level for light-time).
    const JD_J2000: f64 = 2_451_545.0;

    // Convert TT Julian date to approximate TDB seconds since J2000.0.
    fn jd_to_j2000_s(jd: JulianDate) -> f64 {
        (jd.value() - JD_J2000) * 86_400.0
    }

    /// Inter-satellite range observation between two LISA spacecraft.
    ///
    /// Positions are queried from [`LisaEphemerisProvider`] at the appropriate
    /// light-time-corrected epochs.  One Newton iteration is applied:
    /// `t_emit = t_recv − ρ₀/c`.
    ///
    /// `modeled_value` returns `measured_m − modelled_m` (O−C residual, metres).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spod::observations::inter_sat::InterSatRangeObs;
    /// use spod::observations::obs_trait::{CartesianState, Observation};
    /// use spod::observations::provider_bundle::NullProviderBundle;
    /// use spod::io::lisa::{LisaEphemerisProvider, LisaOrbitReader, LisaOrbitSet, LisaSpacecraftId};
    /// use siderust::astro::dynamics::{Position, Velocity};
    /// use siderust::coordinates::frames::GCRS;
    /// use siderust::time::JulianDate;
    /// use std::sync::Arc;
    ///
    /// // In a real scenario, provider is built from OEM files.
    /// # fn doc_only() -> Result<(), Box<dyn std::error::Error>> {
    /// let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src/test-data/lisa");
    /// let load = |name: &str, sc: LisaSpacecraftId| {
    ///     let raw = std::fs::read_to_string(format!("{root}/{name}")).unwrap();
    ///     LisaOrbitReader::from_str(&raw, sc).unwrap()
    /// };
    /// let provider = Arc::new(LisaEphemerisProvider::new(LisaOrbitSet {
    ///     sc1: load("lisa_orbit_sample.oem1", LisaSpacecraftId::SC1),
    ///     sc2: load("lisa_orbit_sample.oem2", LisaSpacecraftId::SC2),
    ///     sc3: load("lisa_orbit_sample.oem3", LisaSpacecraftId::SC3),
    /// }));
    ///
    /// let epoch = JulianDate::new(2_451_545.0 + 1_139_702_400.0 / 86_400.0);
    /// let state: CartesianState = CartesianState::new(
    ///     epoch.to_j2000s(),
    ///     Position::<GCRS>::new(0.0, 0.0, 0.0),
    ///     Velocity::<GCRS>::new(0.0, 0.0, 0.0),
    /// );
    ///
    /// let obs = InterSatRangeObs {
    ///     sc_a: LisaSpacecraftId::SC1,
    ///     sc_b: LisaSpacecraftId::SC2,
    ///     epoch,
    ///     measured_m: 2.5e9,
    ///     sigma: 1e-9,
    ///     provider,
    /// };
    /// let residual = obs.modeled_value(&state, &NullProviderBundle)?;
    /// # Ok(())
    /// # }
    /// ```
    #[derive(Clone)]
    pub struct InterSatRangeObs {
        /// Receiving spacecraft (state at `epoch`).
        pub sc_a: LisaSpacecraftId,
        /// Transmitting spacecraft (state at `epoch − ρ/c`).
        pub sc_b: LisaSpacecraftId,
        /// Observation epoch (TT Julian date, ≈ TDB for light-time purposes).
        pub epoch: JulianDate,
        /// Measured one-way range (metres).
        pub measured_m: f64,
        /// Assumed measurement standard deviation (metres; ≈ pm-level for LISA).
        pub sigma: f64,
        /// Shared LISA ephemeris provider.
        pub provider: Arc<LisaEphemerisProvider>,
    }

    impl std::fmt::Debug for InterSatRangeObs {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("InterSatRangeObs")
                .field("sc_a", &self.sc_a)
                .field("sc_b", &self.sc_b)
                .field("epoch_jd", &self.epoch.value())
                .field("measured_m", &self.measured_m)
                .field("sigma", &self.sigma)
                .finish()
        }
    }

    impl Observation for InterSatRangeObs {
        type Residual = f64;

        fn modeled_value(
            &self,
            _state: &CartesianState,
            _providers: &dyn ProviderBundle,
        ) -> Result<f64, PodObservationsError> {
            let t_recv = jd_to_j2000_s(self.epoch);
            let naif_a = self.sc_a.naif_id();
            let naif_b = self.sc_b.naif_id();

            // State of receiver spacecraft (SC_A) at reception epoch
            let pt_a = self
                .provider
                .state(naif_a, t_recv)
                .map_err(|e| PodObservationsError::LisaProvider(e.to_string()))?;

            // Initial one-way range estimate: use SC_B at the same epoch
            let pt_b0 = self
                .provider
                .state(naif_b, t_recv)
                .map_err(|e| PodObservationsError::LisaProvider(e.to_string()))?;

            let pos_a = pt_a.position;
            let pos_b0 = pt_b0.position;

            let dx0 = pos_a.x().value() - pos_b0.x().value();
            let dy0 = pos_a.y().value() - pos_b0.y().value();
            let dz0 = pos_a.z().value() - pos_b0.z().value();
            let rho0_km = (dx0 * dx0 + dy0 * dy0 + dz0 * dz0).sqrt();

            // One Newton light-time iteration
            let t_emit = t_recv - rho0_km / C_KM_S;
            let pt_b1 = self
                .provider
                .state(naif_b, t_emit)
                .map_err(|e| PodObservationsError::LisaProvider(e.to_string()))?;

            let pos_b1 = pt_b1.position;
            let dx1 = pos_a.x().value() - pos_b1.x().value();
            let dy1 = pos_a.y().value() - pos_b1.y().value();
            let dz1 = pos_a.z().value() - pos_b1.z().value();
            let rho_km = (dx1 * dx1 + dy1 * dy1 + dz1 * dz1).sqrt();
            let rho_m = rho_km * 1_000.0;

            Ok(self.measured_m - rho_m)
        }

        fn obs_type(&self) -> ObsType {
            ObsType::InterSatRange
        }

        fn epoch(&self) -> JulianDate {
            self.epoch
        }

        fn sigma(&self) -> f64 {
            self.sigma
        }
    }
}

#[cfg(feature = "lisa")]
pub use inner::InterSatRangeObs;
