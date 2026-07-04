//! Phase-10 acoustic acceptance: the voice-quality lanes produce their
//! intended effects when rendered. Engine-direct tests drive the forked
//! glottal source with explicit params (steady vowel, clean measurement); the
//! end-to-end test proves the render path runs the attitudinal overlay.

use klattsch_core::params::ParamUpdate;
use klattsch_core::schedule::{MsEvent, Schedule};
use voksa_core::compiler::CompileOptions;
use voksa_core::prosody::ProsodyOptions;
use voksa_engine_klattsch::{SAMPLE_RATE, render_schedule, render_utterance_prosodic};
use voksa_testkit::{measure_diplophonia, measure_f0_variance, measure_spectral_tilt};

/// Render a steady voiced vowel at 120 Hz with the given voice-quality params.
fn render_vq(oq: f32, di: f32, vib: f32, hold_ms: u32) -> Vec<f32> {
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
    if oq != 1.0 {
        u.open_quotient = Some(oq);
    }
    if di != 0.0 {
        u.diplophonia = Some(di);
    }
    if vib != 0.0 {
        u.vibrato_depth = Some(vib);
        u.vibrato_rate = Some(5.5);
    }
    let sched = Schedule::from_ms_events(SAMPLE_RATE, [MsEvent::new(0.0, u, 5.0)]);
    render_schedule(sched, SAMPLE_RATE, hold_ms)
}

#[test]
fn diplophonia_adds_an_f0_half_subharmonic() {
    let clean = render_vq(1.0, 0.0, 0.0, 600);
    let creaky = render_vq(1.0, 0.6, 0.0, 600);
    let clean_d = measure_diplophonia(&clean, SAMPLE_RATE, 120.0);
    let creaky_d = measure_diplophonia(&creaky, SAMPLE_RATE, 120.0);
    assert!(
        creaky_d > clean_d + 0.1,
        "diplophonia must raise the F0/2 subharmonic: clean={clean_d:.3}, creaky={creaky_d:.3}"
    );
}

#[test]
fn open_quotient_shifts_spectral_tilt() {
    // A shorter open phase reshapes the glottal source spectrum, so the low/high
    // band balance shifts measurably (direction is timbre, judged at CP2; here we
    // only require the lane to have a real acoustic effect).
    let modal = render_vq(1.0, 0.0, 0.0, 600);
    let tense = render_vq(0.4, 0.0, 0.0, 600);
    let modal_t = measure_spectral_tilt(&modal, SAMPLE_RATE);
    let tense_t = measure_spectral_tilt(&tense, SAMPLE_RATE);
    assert!(
        (modal_t - tense_t).abs() > 1.0,
        "open quotient must shift the spectral tilt: modal={modal_t:.2}, tense={tense_t:.2}"
    );
}

#[test]
fn vibrato_makes_the_f0_track_oscillate() {
    let steady = render_vq(1.0, 0.0, 0.0, 900);
    let vib = render_vq(1.0, 0.0, 8.0, 900);
    let steady_v = measure_f0_variance(&steady, SAMPLE_RATE);
    let vib_v = measure_f0_variance(&vib, SAMPLE_RATE);
    assert!(
        vib_v > steady_v + 1.0,
        "vibrato must raise F0 variance: steady={steady_v:.2}, vibrato={vib_v:.2}"
    );
}

#[test]
fn vq_render_is_deterministic() {
    assert_eq!(
        render_vq(0.5, 0.4, 6.0, 300),
        render_vq(0.5, 0.4, 6.0, 300),
        "identical voice-quality params must render bit-identically"
    );
}

#[test]
fn fear_vibrato_reaches_the_utterance_render() {
    // render_utterance_prosodic must apply the attitudinal overlay: .ii (fear)
    // adds vibrato to munje, so the whole render's F0 variance rises vs modal.
    let render = |text: &str| {
        render_utterance_prosodic(
            text,
            &CompileOptions::default(),
            &ProsodyOptions::default(),
            SAMPLE_RATE,
        )
        .unwrap_or_else(|e| panic!("{text}: {e:?}"))
    };
    let modal_v = measure_f0_variance(&render("coi munje"), SAMPLE_RATE);
    let fear_v = measure_f0_variance(&render("coi munje .ii"), SAMPLE_RATE);
    assert!(
        fear_v > modal_v + 1.0,
        "fear vibrato must raise the render's F0 variance: modal={modal_v:.2}, fear={fear_v:.2}"
    );
}
