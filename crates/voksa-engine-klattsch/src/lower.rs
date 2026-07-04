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
use voksa_core::schedule::{
    Event, NEUTRAL_DI, NEUTRAL_OQ, NEUTRAL_TILT, NEUTRAL_VIBRATO_HZ, schedule_phonemes,
};

/// Vibrato rate (Hz) paired with any nonzero depth — a natural ~5.5 Hz flutter
/// (the engine ignores rate when depth is 0, so modal frames never send it).
const DEFAULT_VIBRATO_RATE_HZ: f32 = 5.5;

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

/// Offline-render ceiling (~10 minutes). A hostile-but-finite tuning config
/// (huge durations, tiny rate) otherwise saturates the length and aborts on
/// the sample allocation; capping degrades it to a truncated render instead.
const RENDER_MS_CEILING: f32 = 600_000.0;

/// The offline render length for a schedule: total + a short decay tail,
/// bounded to [`RENDER_MS_CEILING`].
fn render_ms(total_ms: f32) -> u32 {
    (total_ms + 20.0).clamp(0.0, RENDER_MS_CEILING) as u32
}

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
///
/// The Phase-10 voice-quality lanes (oq/tilt/di/vibrato) are Option-gated
/// against the LAST-EMITTED value (seeded with the engine's modal defaults):
/// a lane is only overridden when it CHANGES. So a fully-modal utterance emits
/// them all-None (byte-identical to the pre-Phase-10 lowering, engine snapshots
/// unchanged), and a colored word resets to modal on exit — the reset event
/// carries the change back to neutral instead of the creak/vibrato bleeding on.
pub fn lower_events(events: &[Event]) -> Vec<MsEvent> {
    let mut prev_oq = NEUTRAL_OQ;
    let mut prev_tilt = NEUTRAL_TILT;
    let mut prev_di = NEUTRAL_DI;
    let mut prev_vib = NEUTRAL_VIBRATO_HZ;
    events
        .iter()
        .map(|e| {
            let mut u = targets_update(&e.frame.targets, e.frame.f0_hz);
            if e.frame.oq != prev_oq {
                u.open_quotient = Some(e.frame.oq);
                prev_oq = e.frame.oq;
            }
            if e.frame.tilt != prev_tilt {
                u.tilt = Some(e.frame.tilt);
                prev_tilt = e.frame.tilt;
            }
            if e.frame.di != prev_di {
                u.diplophonia = Some(e.frame.di);
                prev_di = e.frame.di;
            }
            if e.frame.vibrato_hz != prev_vib {
                u.vibrato_depth = Some(e.frame.vibrato_hz);
                u.vibrato_rate = Some(DEFAULT_VIBRATO_RATE_HZ);
                prev_vib = e.frame.vibrato_hz;
            }
            MsEvent::new(e.at_ms, u, e.transition_ms)
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
    crate::render_schedule(schedule, sample_rate, render_ms(total_ms))
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
        render_ms(utterance.total_ms),
    ))
}

/// Compile Lojban text, apply sentence prosody (declination + stress + xu) and
/// the Phase-10 attitudinal voice-quality overlay, then render offline. Any UI
/// cmavo in the text (`.ui`/`.oi`/`.ii`/…) colors its target word automatically
/// — on native, CLI, and the browser (no C-ABI change). The flat path
/// ([`render_utterance`]) skips both transforms.
pub fn render_utterance_prosodic(
    text: &str,
    opts: &CompileOptions,
    prosody: &voksa_core::prosody::ProsodyOptions,
    sample_rate: u32,
) -> Result<Vec<f32>, CompileError> {
    render_utterance_expressive(
        text,
        opts,
        prosody,
        &voksa_core::attitudinal::AttitudinalTable::default(),
        &voksa_core::phonemes::VoiceTable::default(),
        sample_rate,
    )
}

/// Like [`render_utterance_prosodic`] but with RUNTIME attitudinal deviation
/// (demo tuning console D2a) and per-phoneme voice (D2b) tables. The default
/// tables render byte-identically to [`render_utterance_prosodic`].
pub fn render_utterance_expressive(
    text: &str,
    opts: &CompileOptions,
    prosody: &voksa_core::prosody::ProsodyOptions,
    attitudinals: &voksa_core::attitudinal::AttitudinalTable,
    voice: &voksa_core::phonemes::VoiceTable,
    sample_rate: u32,
) -> Result<Vec<f32>, CompileError> {
    let utterance = voksa_core::attitudinal::apply_attitudinal_with(
        voksa_core::prosody::apply_prosody(
            voksa_core::compiler::compile_with(text, opts, voice)?,
            prosody,
        ),
        attitudinals,
    );
    let schedule = Schedule::from_ms_events(sample_rate, lower_events(&utterance.events));
    Ok(crate::render_schedule(
        schedule,
        sample_rate,
        render_ms(utterance.total_ms),
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
