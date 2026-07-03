//! Lowering: voksa-core's engine-neutral schedule IR → klattsch schedules.
//!
//! Since Phase 5 the timing conventions live in voksa-core
//! (`schedule::schedule_segment` / `schedule_phonemes` / `compiler::compile`);
//! this module is a 1:1 event translation that owns ONLY the klattsch
//! specifics: the linear-range gain, the Klatt-1980 alternating A2 polarity,
//! and the parameter fields the engine has that the IR doesn't.

use klattsch_core::params::ParamUpdate;
use klattsch_core::schedule::{MsEvent, Schedule};
use voksa_core::compiler::{CompileError, CompileOptions, compile};
use voksa_core::phonemes::{Phoneme, Targets};
use voksa_core::schedule::{Event, schedule_phonemes};

/// Flat robotic baseline F0 (defined in core; re-exported for compatibility).
pub use voksa_core::schedule::BASE_F0_HZ;

/// Steady-measurement F0: 105 Hz is the unique grid whose harmonics land
/// within tolerance of EVERY docs/formants.md vowel target (e.g. 8×105 = 840
/// exactly for /o/ F2; 735 vs 730; 315 vs 300 within the ±50 Hz F1 floor).
/// FFT peak-picking can only observe the resonance envelope at source
/// harmonics; the resonators themselves are identical at any F0 — only the
/// observation grid moves. LPC estimates the envelope directly, F0-agnostic.
const MEASUREMENT_F0_HZ: f32 = 105.0;

/// klattsch's default gain (3.5) drives soft_clip nonlinear (research: linear
/// only below |y| ≈ 0.85); clipping distorts LPC pole estimates and spawns
/// intermodulation peaks. Keep the adapter's renders linear.
const LINEAR_GAIN: f32 = 1.0;

fn targets_update(t: &Targets, f0: f32) -> ParamUpdate {
    ParamUpdate {
        f0: Some(f0),
        gain: Some(LINEAR_GAIN),
        voicing: Some(t.voicing),
        f1: Some(t.formants[0].freq_hz),
        bw1: Some(t.formants[0].bw_hz),
        a1: Some(t.formants[0].amp),
        f2: Some(t.formants[1].freq_hz),
        bw2: Some(t.formants[1].bw_hz),
        // Klatt 1980 alternating polarity: adjacent parallel bandpass branches
        // are ~180° out of phase between resonances, so summing them all
        // positive carves deep spectral zeros mid-spectrum (which biases any
        // all-pole/LPC analysis and thins the timbre). Flipping A2 fills the
        // F1-F2 and F2-F3 notches. Magnitude at the resonance peaks is
        // unchanged. klattsch itself sums branches positively, so the
        // alternation lives here in the lowering; the core IR stays positive.
        a2: Some(-t.formants[1].amp),
        f3: Some(t.formants[2].freq_hz),
        bw3: Some(t.formants[2].bw_hz),
        a3: Some(t.formants[2].amp),
        aspiration: Some(t.aspiration),
        ..ParamUpdate::default()
    }
}

/// 1:1 translation of core IR events into klattsch millisecond events.
pub fn lower_events(events: &[Event]) -> Vec<MsEvent> {
    events
        .iter()
        .map(|e| {
            MsEvent::new(
                e.at_ms,
                targets_update(&e.frame.targets, e.frame.f0_hz),
                e.transition_ms,
            )
        })
        .collect()
}

/// Lower a phoneme sequence to a klattsch schedule (Phase-2 compatible path).
/// Returns the schedule and the total duration in ms.
pub fn lower_sequence(phonemes: &[Phoneme], sample_rate: u32) -> (Schedule, f32) {
    let (events, total_ms) = schedule_phonemes(phonemes, BASE_F0_HZ);
    (
        Schedule::from_ms_events(sample_rate, lower_events(&events)),
        total_ms,
    )
}

/// Render a phoneme sequence offline (short tail appended so final transients
/// decay inside the buffer).
pub fn render_phonemes(phonemes: &[Phoneme], sample_rate: u32) -> Vec<f32> {
    let (schedule, total_ms) = lower_sequence(phonemes, sample_rate);
    crate::render_schedule(schedule, sample_rate, (total_ms + 20.0) as u32)
}

/// Compile Lojban text (voksa-core pipeline) and render it offline.
pub fn render_utterance(
    text: &str,
    opts: &CompileOptions,
    sample_rate: u32,
) -> Result<Vec<f32>, CompileError> {
    let utterance = compile(text, opts)?;
    let schedule = Schedule::from_ms_events(sample_rate, lower_events(&utterance.events));
    Ok(crate::render_schedule(
        schedule,
        sample_rate,
        (utterance.total_ms + 20.0) as u32,
    ))
}

/// Render one steady-capable phoneme held for `hold_ms` (measurement helper —
/// per-vowel/fricative acceptance tests need long stationary segments).
pub fn render_steady_phoneme(p: Phoneme, sample_rate: u32, hold_ms: u32) -> Vec<f32> {
    let seg = voksa_core::phonemes::spec(p);
    let t = seg
        .leading_targets()
        .expect("render_steady_phoneme requires a phoneme with steady targets");
    let schedule = Schedule::from_ms_events(
        sample_rate,
        [MsEvent::new(
            0.0,
            targets_update(&t, MEASUREMENT_F0_HZ),
            5.0,
        )],
    );
    crate::render_schedule(schedule, sample_rate, hold_ms)
}
