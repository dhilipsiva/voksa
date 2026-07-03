//! Thin adapter over the klattsch-core parallel-formant engine.
//!
//! klattsch-core is std-only, so it must never appear in no_std voksa-core's
//! dependency tree; this crate owns the dependency and will later lower
//! voksa-core's schedule IR into klattsch schedules. Kept deliberately thin:
//! the end-of-Phase-2 decision gate may swap the engine for a hand-rolled one.

use klattsch_core::params::ParamUpdate;
use klattsch_core::schedule::{MsEvent, Schedule};
use klattsch_core::synth::FormantSynth;

/// Project-wide sample rate (Hz). Matches the klattsch convention and the
/// typical browser AudioContext rate.
pub const SAMPLE_RATE: u32 = 48_000;

/// Fixed noise seed so every render is bit-reproducible within a platform.
const NOISE_SEED: u32 = 0x766f_6b73; // "voks"

/// Hardcoded steady /a/ vowel schedule (Phase-1 engine spike).
/// Targets and bandwidth seeds from docs/formants.md: F1 730/90, F2 1090/110,
/// F3 2440/150 Hz; amplitudes fall with formant number so band peak-picking
/// resolves each formant. klattsch defaults are silent, so voicing and a1–a3
/// must be set explicitly.
pub fn steady_a_schedule(sample_rate: u32) -> Schedule {
    let a_target = ParamUpdate {
        f0: Some(120.0),
        voicing: Some(1.0),
        f1: Some(730.0),
        bw1: Some(90.0),
        a1: Some(1.0),
        f2: Some(1090.0),
        bw2: Some(110.0),
        a2: Some(0.6),
        f3: Some(2440.0),
        bw3: Some(150.0),
        a3: Some(0.3),
        ..ParamUpdate::default()
    };
    Schedule::from_ms_events(sample_rate, [MsEvent::new(0.0, a_target, 5.0)])
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
