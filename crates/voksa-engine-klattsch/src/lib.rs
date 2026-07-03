//! Thin adapter over the klattsch-core parallel-formant engine.
//!
//! klattsch-core is std-only, so it must never appear in no_std voksa-core's
//! dependency tree; this crate owns the dependency and will later lower
//! voksa-core's schedule IR into klattsch schedules. Kept deliberately thin:
//! the end-of-Phase-2 decision gate may swap the engine for a hand-rolled one.

pub mod lower;

use klattsch_core::schedule::Schedule;
use klattsch_core::synth::FormantSynth;

pub use lower::{
    lower_events, lower_sequence, render_phonemes, render_steady_phoneme, render_utterance,
    render_utterance_prosodic,
};

/// Project-wide sample rate (Hz). Matches the klattsch convention and the
/// typical browser AudioContext rate.
pub const SAMPLE_RATE: u32 = 48_000;

/// Fixed noise seed so every render is bit-reproducible within a platform.
const NOISE_SEED: u32 = 0x766f_6b73; // "voks"

/// Steady /a/ vowel schedule (Phase-1 engine spike, now sourced from the
/// Phase-2 phoneme table so the spike acceptance test guards the table too).
pub fn steady_a_schedule(sample_rate: u32) -> Schedule {
    use voksa_core::phonemes::{Phoneme, Vowel};
    lower_sequence(&[Phoneme::Vowel(Vowel::A)], sample_rate).0
}

/// Render a schedule offline to mono f32 PCM in [-1, 1].
///
/// Processes in small chunks to exercise the same streaming path the realtime
/// adapters (Phases 8/9) will use — klattsch output is block-size invariant.
pub fn render_schedule(schedule: Schedule, sample_rate: u32, duration_ms: u32) -> Vec<f32> {
    let mut synth = FormantSynth::with_seed(sample_rate, NOISE_SEED);
    synth.queue_schedule(schedule);
    let total = (u64::from(sample_rate) * u64::from(duration_ms) / 1000) as usize;
    let mut out = vec![0.0f32; total];
    for chunk in out.chunks_mut(128) {
        synth.process(chunk);
    }
    out
}
