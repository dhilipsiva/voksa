//! Phase-11 lever 2: Klatt-style F0 flutter must be acoustically measurable —
//! a slow (3–15 Hz) quasi-random wobble on the F0 contour, fully deterministic
//! (sum of incommensurate sinusoids, no PRNG).

use klattsch_core::params::ParamUpdate;
use klattsch_core::schedule::{MsEvent, Schedule};
use voksa_engine_klattsch::{SAMPLE_RATE, render_schedule};
use voksa_testkit::{f0_band_rms, measure_f0_track_raw};

fn render_vowel(flutter: f32, hold_ms: u32) -> Vec<f32> {
    let mut u = ParamUpdate {
        f0: Some(120.0),
        gain: Some(1.0),
        voicing: Some(1.0),
        f1: Some(700.0),
        bw1: Some(90.0),
        a1: Some(1.0),
        f2: Some(1200.0),
        bw2: Some(110.0),
        a2: Some(0.8),
        f3: Some(2600.0),
        bw3: Some(150.0),
        a3: Some(0.4),
        ..Default::default()
    };
    if flutter != 0.0 {
        u.flutter = Some(flutter);
    }
    let sched = Schedule::from_ms_events(SAMPLE_RATE, [MsEvent::new(0.0, u, 5.0)]);
    render_schedule(sched, SAMPLE_RATE, hold_ms)
}

#[test]
fn flutter_produces_measurable_f0_modulation() {
    let steady = render_vowel(0.0, 2000);
    let fluttered = render_vowel(50.0, 2000);
    let steady_rms = f0_band_rms(&measure_f0_track_raw(&steady, SAMPLE_RATE), 3.0, 15.0);
    let flutter_rms = f0_band_rms(&measure_f0_track_raw(&fluttered, SAMPLE_RATE), 3.0, 15.0);
    assert!(
        flutter_rms > steady_rms * 3.0 && flutter_rms > 0.3,
        "FL=50 must raise the 3–15 Hz F0 band RMS: steady={steady_rms:.3}, flutter={flutter_rms:.3}"
    );
}
