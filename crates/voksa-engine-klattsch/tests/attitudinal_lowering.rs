//! Phase-10 lowering: a Frame's voice-quality lanes (oq/tilt/di/vibrato_hz)
//! map to klattsch ParamUpdate fields, Option-gated against the PREVIOUS frame
//! so (a) fully-modal utterances stay byte-identical (all-None → engine
//! schedule snapshots unchanged) and (b) a colored word resets to modal on
//! exit instead of bleeding into the next word.

use voksa_core::attitudinal::apply_attitudinal;
use voksa_core::compiler::{CompileOptions, compile};
use voksa_core::phonemes::{Phoneme, Targets, Vowel, spec};
use voksa_core::prosody::{ProsodyOptions, apply_prosody};
use voksa_core::schedule::{Event, Frame};
use voksa_engine_klattsch::lower_events;

fn vowel_targets() -> Targets {
    spec(Phoneme::Vowel(Vowel::A))
        .leading_targets()
        .expect("a vowel has steady targets")
}

fn ev(at_ms: f32, frame: Frame) -> Event {
    Event {
        at_ms,
        transition_ms: 10.0,
        frame,
        micro: voksa_core::schedule::MicroClass::Vowel(voksa_core::schedule::VowelHeight::Low),
    }
}

#[test]
fn modal_events_lower_without_vq_overrides() {
    // Byte-identity guard: modal frames must leave every Phase-10 lane at None
    // so the lowered ParamUpdate is exactly the pre-Phase-10 one.
    let t = vowel_targets();
    let events = vec![
        ev(0.0, Frame::modal(120.0, t)),
        ev(100.0, Frame::modal(118.0, t)),
    ];
    for m in lower_events(&events) {
        assert_eq!(m.target.open_quotient, None);
        assert_eq!(m.target.tilt, None);
        assert_eq!(m.target.diplophonia, None);
        assert_eq!(m.target.vibrato_depth, None);
    }
}

#[test]
fn vq_frames_lower_to_param_updates() {
    let t = vowel_targets();
    let mut f = Frame::modal(120.0, t);
    f.oq = 0.7;
    f.tilt = 0.2;
    f.di = 0.5;
    f.vibrato_hz = 6.0;
    let lowered = lower_events(&[ev(0.0, f)]);
    let u = lowered[0].target;
    assert_eq!(u.open_quotient, Some(0.7));
    assert_eq!(u.tilt, Some(0.2));
    assert_eq!(u.diplophonia, Some(0.5));
    assert_eq!(u.vibrato_depth, Some(6.0));
    assert!(
        u.vibrato_rate.is_some(),
        "a vibrato depth must carry a rate so the engine actually oscillates"
    );
}

#[test]
fn vq_resets_to_neutral_on_word_exit() {
    // Enter a diplophonic word then leave it. The prev-frame diff must emit
    // Some(0.5) on entry, None while steady, and Some(0.0) on exit (the reset)
    // — otherwise the creak bleeds into the following word.
    let t = vowel_targets();
    let modal = Frame::modal(120.0, t);
    let mut colored = modal;
    colored.di = 0.5;
    let events = vec![
        ev(0.0, modal),
        ev(50.0, colored),
        ev(100.0, colored),
        ev(150.0, modal),
    ];
    let l = lower_events(&events);
    assert_eq!(l[0].target.diplophonia, None, "modal onset");
    assert_eq!(l[1].target.diplophonia, Some(0.5), "creak begins");
    assert_eq!(l[2].target.diplophonia, None, "steady creak: no re-send");
    assert_eq!(l[3].target.diplophonia, Some(0.0), "reset on exit");
}

#[test]
fn prosodic_pipeline_threads_diplophonia_to_engine() {
    // The full compile → prosody → attitudinal → lower path must carry the .oi
    // (complaint) diplophonia into at least one ParamUpdate.
    let s = apply_attitudinal(apply_prosody(
        compile("coi munje .oi", &CompileOptions::default()).unwrap(),
        &ProsodyOptions::default(),
    ));
    let lowered = lower_events(&s.events);
    assert!(
        lowered
            .iter()
            .any(|m| matches!(m.target.diplophonia, Some(d) if d > 0.0)),
        "the .oi pipeline must thread diplophonia into the engine schedule"
    );
}
