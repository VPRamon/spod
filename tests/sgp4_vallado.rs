//! Regression coverage for the locally retained full Vallado SGP4/SDP4 path.

use siderust::formats::tle::parse_tle;
use spod::sgp4::{Sgp4Error, Sgp4Propagator};

fn assert_state(
    propagator: &Sgp4Propagator,
    minutes: f64,
    expected_position: [f64; 3],
    expected_velocity: [f64; 3],
) {
    let state = propagator.propagate_minutes(minutes).unwrap();
    for (actual, expected) in state.position().as_array().iter().zip(expected_position) {
        assert!((actual.value() - expected).abs() < 1e-6);
    }
    for (actual, expected) in state.velocity().as_array().iter().zip(expected_velocity) {
        assert!((actual.value() - expected).abs() < 1e-9);
    }
}

#[test]
fn matches_vallado_near_earth_reference_epochs() {
    let tle = parse_tle(
        "1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753",
        "2 00005  34.2682 348.7242 1859667 331.7664  19.3264 10.82419157413667",
    )
    .unwrap();
    let propagator = Sgp4Propagator::from_tle(&tle).unwrap();

    assert_state(
        &propagator,
        0.0,
        [7022.46529266, -1400.08296755, 0.03995155],
        [1.893841015, 6.405893759, 4.534807250],
    );
    assert_state(
        &propagator,
        1440.0,
        [-938.55923943, -6268.18748831, -4294.02924751],
        [7.536105209, -0.427127707, 0.989878080],
    );
}

#[test]
fn matches_vallado_deep_space_reference_epoch() {
    let tle = parse_tle(
        "1 04632U 70093B   04031.91070959 -.00000084  00000-0  10000-3 0  9955",
        "2 04632  11.4628 273.1101 1450506 207.6000 143.9350  1.20231981 44145",
    )
    .unwrap();
    let propagator = Sgp4Propagator::from_tle(&tle).unwrap();

    assert_state(
        &propagator,
        -5184.0,
        [-29020.02587128, 13819.84419063, -5713.33679183],
        [-1.768068390, -3.235371192, -0.395206135],
    );
}

#[test]
fn reports_vallado_propagation_error() {
    let tle = parse_tle(
        "1 33333U 05037B   05333.02012661  .25992681  00000-0  24476-3 0  1532",
        "2 33333  96.4736 157.9986 9950000 244.0492 110.6523  4.00004038 10700",
    )
    .unwrap();
    let propagator = Sgp4Propagator::from_tle(&tle).unwrap();

    let error = propagator.propagate_minutes(25.0).unwrap_err();
    assert!(matches!(error, Sgp4Error::Propagation { .. }));
    assert!(error.to_string().contains("NegativeSemiLatusRectum"));
}
