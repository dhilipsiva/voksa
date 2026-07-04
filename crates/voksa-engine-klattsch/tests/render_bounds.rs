//! Review finding (D2b): a hostile-but-finite tuning config (huge segment
//! durations, tiny prosody rate) must degrade to a CAPPED render, never an
//! allocation abort — `voksa --config` runs community-authored JSON, and the
//! browser demo runs the same values inside an AudioWorklet.

use voksa_core::attitudinal::AttitudinalTable;
use voksa_core::compiler::CompileOptions;
use voksa_core::phonemes::VoiceTable;
use voksa_core::prosody::ProsodyOptions;
use voksa_engine_klattsch::{SAMPLE_RATE, render_utterance_expressive};

#[test]
fn render_length_is_bounded_under_hostile_configs() {
    // rate 1e-3 stretches the schedule ~1000× (≈ 26 minutes) — the render must
    // cap (~10 minutes), not allocate whatever the schedule asks for.
    let prosody = ProsodyOptions {
        rate: 1e-3,
        ..Default::default()
    };
    let samples = render_utterance_expressive(
        "mi klama",
        &CompileOptions::default(),
        &prosody,
        &AttitudinalTable::default(),
        &VoiceTable::default(),
        SAMPLE_RATE,
    )
    .expect("hostile-but-finite configs still render");
    assert!(
        samples.len() <= SAMPLE_RATE as usize * 601,
        "render must cap at ~10 minutes of audio, got {} samples (~{} s)",
        samples.len(),
        samples.len() / SAMPLE_RATE as usize
    );
}
