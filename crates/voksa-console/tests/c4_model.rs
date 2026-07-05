//! C4 wave 1: the A/B render-time naturalness override + the voice-table
//! reset range. `ab_effective` is a render-time override — stored values are
//! never touched; only the nine naturalness slots move to identity.

use voksa_console::model::{Descriptors, ab_effective};

fn desc() -> Descriptors {
    Descriptors::from_defaults(&voksa_web::default_params()).unwrap()
}

#[test]
fn ab_off_overrides_only_the_nine_naturalness_slots() {
    let d = desc();
    let mut params = voksa_web::default_params();
    // Dirty a spread of knobs across all four sections.
    params[0] = 150.0; // prosody: pitch start
    params[7] = 30.0; // attitudinal: ui f0_mean
    params[63] = 1400.0; // voice: a.f1 (widened territory)
    params[440] = 40.0; // naturalness: flutter
    params[446] = 1.8; // naturalness: final_lengthen

    let off = ab_effective(&d, &params, true);
    // The nine naturalness slots become their identity values.
    assert_eq!(
        &off[440..449],
        &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    );
    // Everything else is untouched.
    assert_eq!(off[0], 150.0);
    assert_eq!(off[7], 30.0);
    assert_eq!(off[63], 1400.0);
    assert_eq!(&off[..440], &params[..440]);
    // Stored params are NOT mutated (render-time override only).
    assert_eq!(params[440], 40.0);
    assert_eq!(params[446], 1.8);
}

#[test]
fn ab_on_is_the_identity() {
    let d = desc();
    let params = {
        let mut p = voksa_web::default_params();
        p[440] = 40.0;
        p
    };
    assert_eq!(ab_effective(&d, &params, false), params);
}

#[test]
fn voice_range_is_the_full_voice_section() {
    let d = desc();
    assert_eq!(d.voice_range(), 63..440);
}
