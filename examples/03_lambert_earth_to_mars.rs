//! Typed heliocentric Earth → Mars Lambert transfer.
//!
//! This is deliberately a compact API example rather than a mission-design
//! model. It defines two notional heliocentric ICRS positions, a typed time of
//! flight and a typed solar gravitational parameter, then keeps the returned
//! velocities as affn/qtty values instead of unpacking them into f64.
//!
//! Run with:
//! cargo run --example 03_lambert_earth_to_mars

#![allow(clippy::print_stdout)]

use siderust::affn::cartesian::Position;
use siderust::affn::frames::ICRS;
use siderust::qtty::unit::Kilometer;
use siderust::qtty::GravitationalParameter;
use siderust::qtty::Second;
use spod::lambert::{lambert, LambertBranch};

const AU_KM: f64 = 1.495_978_707e8;
const MU_SUN_KM3_S2: f64 = 1.327_124_400_18e11;
const SECONDS_PER_DAY: f64 = 86_400.0;

fn main() {
    let earth_radius = AU_KM;
    let mars_radius = 1.524 * AU_KM;
    let mars_lead = 60.0_f64.to_radians();

    let earth = Position::<(), ICRS, Kilometer>::new(earth_radius, 0.0, 0.0);
    let mars = Position::<(), ICRS, Kilometer>::new(
        mars_radius * mars_lead.cos(),
        mars_radius * mars_lead.sin(),
        0.0,
    );

    let time_of_flight = Second::new(258.0 * SECONDS_PER_DAY);
    let mu_sun = GravitationalParameter::new(MU_SUN_KM3_S2);

    let solution = lambert(earth, mars, time_of_flight, mu_sun, LambertBranch::Prograde)
        .expect("Lambert solver failed for the demonstration geometry");

    println!("=== Typed Earth → Mars Lambert transfer ===");
    println!("time of flight : {time_of_flight}");
    println!("departure speed: {:.6}", solution.v1.magnitude());
    println!("arrival speed  : {:.6}", solution.v2.magnitude());
    println!("departure velocity:");
    println!("  vx = {:.6}", solution.v1.x());
    println!("  vy = {:.6}", solution.v1.y());
    println!("  vz = {:.6}", solution.v1.z());
    println!("arrival velocity:");
    println!("  vx = {:.6}", solution.v2.x());
    println!("  vy = {:.6}", solution.v2.y());
    println!("  vz = {:.6}", solution.v2.z());
    println!("iterations     : {}", solution.diagnostics.iterations);
    println!("revolutions    : {}", solution.diagnostics.revolutions);
}
