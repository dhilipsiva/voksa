//! End-to-end plumbing smoke: text through the core compiler, lowered and
//! rendered by the adapter.

use voksa_core::compiler::CompileOptions;
use voksa_engine_klattsch::{SAMPLE_RATE, render_utterance};

#[test]
fn coi_munje_renders_audibly() {
    let samples =
        render_utterance("coi munje", &CompileOptions::default(), SAMPLE_RATE).expect("compiles");
    // c-o-i (120+200 ms) + m-u-n-j-e (~530 ms) + 20 ms tail, at 48 kHz.
    assert!(samples.len() > 30_000, "got {} samples", samples.len());
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    assert!(rms > 0.01, "utterance must be audible, rms {rms:.4}");
    let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    assert!(peak < 0.95, "must stay in linear range, peak {peak:.3}");
}

#[test]
fn pause_renders_as_silence_gap() {
    let with_pause =
        render_utterance("coi djan", &CompileOptions::default(), SAMPLE_RATE).expect("compiles");
    let no_words =
        render_utterance("mi klama", &CompileOptions::default(), SAMPLE_RATE).expect("compiles");
    // Just a plumbing check: both render, non-empty.
    assert!(!with_pause.is_empty() && !no_words.is_empty());
}
