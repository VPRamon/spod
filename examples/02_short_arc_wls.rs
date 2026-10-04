//! Short-arc weighted least-squares correction.
//!
//! A small POD-like linearized model is used here:
//!
//! residual(t) ≈ Δr + t·Δv
//!
//! where Δr is a radial position correction and Δv is a radial velocity
//! correction. NormalEquations intentionally operates in normalized solver
//! space (f64); the result is wrapped back into qtty quantities immediately
//! at the domain boundary.
//!
//! Run with:
//! cargo run --example 02_short_arc_wls

#![allow(clippy::print_stdout)]

use siderust::qtty::unit::Kilometer;
use siderust::qtty::KmPerSecond;
use siderust::qtty::Quantity;
use siderust_pod::estimation::{NormalEquations, ParameterKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let solve_for = [ParameterKind::Position(0), ParameterKind::Velocity(0)];

    // Synthetic range-like residuals in km at successive seconds.
    // The trend is close to Δr = 0.120 km and Δv = -0.000020 km/s.
    let observations = [
        (0.0, 0.1200, 0.0010),
        (60.0, 0.1187, 0.0010),
        (120.0, 0.1177, 0.0010),
        (180.0, 0.1163, 0.0010),
        (240.0, 0.1153, 0.0010),
    ];

    let mut normal = NormalEquations::new(solve_for.len());
    for (t_s, residual_km, sigma_km) in observations {
        normal.add_row(&[(0, 1.0), (1, t_s)], residual_km, sigma_km)?;
    }

    let result = normal.solve()?;
    let delta_r = Quantity::<Kilometer>::new(result.update[0]);
    let delta_v = Quantity::<KmPerSecond>::new(result.update[1]);

    println!("=== Short-arc WLS correction ===");
    println!("solve-for       : {:?}, {:?}", solve_for[0], solve_for[1]);
    println!("observations    : {}", result.n_obs);
    println!("Δr              : {:.6}", delta_r);
    println!("Δv              : {:.9}", delta_v);
    println!("reduced χ²      : {:.6}", result.reduced_chi2());

    Ok(())
}
