//! Minimal typed LEO propagation with the siderust dynamics stack.
//!
//! The example keeps epoch, frame, position, velocity, duration, and gravity
//! parameter typed from construction through propagation. Raw scalars only
//! appear when defining the initial numerical values.
//!
//! Run with:
//! cargo run --example 01_typed_two_body_propagation

#![allow(clippy::print_stdout)]

use siderust::astro::dynamics::GM_EARTH;
use siderust::coordinates::frames::GCRS;
use siderust::qtty::Second;
use siderust::time::JulianDate;
use spod::dynamics::{
    DynamicsContext, Integrator, OrbitState, Position, Rk4Integrator, TwoBody, Velocity,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let initial = OrbitState::new(
        JulianDate::new(2_451_545.0).to_j2000s(),
        Position::<GCRS>::new(7_000.0, 0.0, 0.0),
        Velocity::<GCRS>::new(0.0, 7.545, 0.0),
    );

    let duration = Second::new(45.0 * 60.0);
    let integrator = Rk4Integrator {
        step: Second::new(10.0),
    };
    let gravity = TwoBody::new(GM_EARTH);
    let context = DynamicsContext::empty();

    let initial_radius = initial.position.distance();
    let initial_speed = initial.velocity.magnitude();
    let propagated = integrator.propagate(&gravity, initial, duration, &context)?;

    println!("=== Typed two-body LEO propagation ===");
    println!("duration       : {duration}");
    println!("initial radius : {:.3}", initial_radius);
    println!("initial speed  : {:.6}", initial_speed);
    println!("final radius   : {:.3}", propagated.position.distance());
    println!("final speed    : {:.6}", propagated.velocity.magnitude());
    println!("final position :");
    println!("  x = {:.3}", propagated.position.x());
    println!("  y = {:.3}", propagated.position.y());
    println!("  z = {:.3}", propagated.position.z());

    Ok(())
}
