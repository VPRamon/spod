//! Parse a TLE and propagate typed TEME states with SGP4.
//!
//! The SGP4 wrapper already exposes the natural minutes-since-epoch API, so
//! this example uses it directly. Position and velocity stay typed in TEME,
//! kilometres, and kilometres per second all the way to presentation.
//!
//! Run with:
//! cargo run --example 04_sgp4_from_tle

#![allow(clippy::print_stdout)]

use spod::sgp4::{GravityModel, Sgp4Propagator};
use spod::tle::parse_tle;

const L1: &str = "1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753";
const L2: &str = "2 00005  34.2682 348.7242 1859667 331.7664  19.3264 10.82419157413667";

fn main() {
    let tle = parse_tle(L1, L2).expect("Vallado verification TLE should parse");
    let propagator = Sgp4Propagator::from_tle_with_model(&tle, GravityModel::Wgs72)
        .expect("SGP4 initialization should succeed");

    println!("=== Typed SGP4 propagation ===");
    println!("satellite     : NORAD {}", tle.norad_id.0);
    println!("gravity model : {:?}", propagator.gravity_model());

    for minutes in [0.0, 360.0, 720.0, 1_080.0] {
        let state = propagator
            .propagate_minutes(minutes)
            .expect("propagation should succeed");

        println!();
        println!("t = {minutes:>7.1} min");
        println!("radius : {:.3}", state.position().distance());
        println!("speed  : {:.6}", state.velocity().magnitude());
        println!("position:");
        println!("  x = {:.3}", state.position().x());
        println!("  y = {:.3}", state.position().y());
        println!("  z = {:.3}", state.position().z());
        println!("velocity:");
        println!("  vx = {:.6}", state.velocity().x());
        println!("  vy = {:.6}", state.velocity().y());
        println!("  vz = {:.6}", state.velocity().z());
    }
}
