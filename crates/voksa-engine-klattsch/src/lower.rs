//! Lowering: voksa-core phoneme IR → klattsch schedules.
//!
//! Conventions follow klattsch-text's proven shapes: steady segments get one
//! event with transition min(35 ms, dur·0.4); stops get a closure event then a
//! burst event; a diphthong is 25% onset / 50% glide / 25% offset where the
//! glide is a single event whose transition time IS the glide (the synth ramps
//! parameters linearly per sample).

use klattsch_core::params::ParamUpdate;
use klattsch_core::schedule::{MsEvent, Schedule};
use voksa_core::phonemes::{Phoneme, SegmentKind, SegmentSpec, Targets, specs};

/// Flat robotic baseline F0 until the prosody layer (Phase 7) exists.
pub const BASE_F0_HZ: f32 = 120.0;

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

fn update_from(t: &Targets) -> ParamUpdate {
    update_with_f0(t, BASE_F0_HZ)
}

fn update_with_f0(t: &Targets, f0: f32) -> ParamUpdate {
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
        // alternation lives here in the lowering.
        a2: Some(-t.formants[1].amp),
        f3: Some(t.formants[2].freq_hz),
        bw3: Some(t.formants[2].bw_hz),
        a3: Some(t.formants[2].amp),
        aspiration: Some(t.aspiration),
        ..ParamUpdate::default()
    }
}

/// [h] has no shape of its own: unvoiced noise through the following vowel's
/// formants at reduced amplitude (docs/formants.md).
fn aspirate_targets(next: Option<Targets>) -> Targets {
    let mut t = next.unwrap_or(FALLBACK_SCHWA);
    t.voicing = 0.0;
    t.aspiration = 1.0;
    for f in &mut t.formants {
        f.amp *= 0.7;
    }
    t
}

/// Utterance-final [h] fallback shape (schwa-like); phonotactically the
/// apostrophe is intervocalic, so this only guards degenerate input.
const FALLBACK_SCHWA: Targets = Targets {
    formants: [
        voksa_core::phonemes::Formant {
            freq_hz: 500.0,
            bw_hz: 90.0,
            amp: 0.5,
        },
        voksa_core::phonemes::Formant {
            freq_hz: 1500.0,
            bw_hz: 110.0,
            amp: 0.3,
        },
        voksa_core::phonemes::Formant {
            freq_hz: 2500.0,
            bw_hz: 150.0,
            amp: 0.15,
        },
    ],
    voicing: 0.0,
    aspiration: 1.0,
};

/// Lower one segment starting at `at_ms`; push its events, return its end time.
pub fn lower_segment(
    seg: &SegmentSpec,
    next: Option<Targets>,
    at_ms: f32,
    out: &mut Vec<MsEvent>,
) -> f32 {
    match seg.kind {
        SegmentKind::Steady(t) => {
            out.push(MsEvent::new(
                at_ms,
                update_from(&t),
                (seg.dur_ms * 0.4).min(35.0),
            ));
            at_ms + seg.dur_ms
        }
        SegmentKind::Glide { from, to } => {
            let onset_ms = seg.dur_ms * 0.25;
            let glide_ms = seg.dur_ms * 0.5;
            out.push(MsEvent::new(
                at_ms,
                update_from(&from),
                (onset_ms * 0.4).min(35.0),
            ));
            out.push(MsEvent::new(at_ms + onset_ms, update_from(&to), glide_ms));
            at_ms + seg.dur_ms
        }
        SegmentKind::Stop {
            closure,
            burst,
            closure_ms,
            burst_ms,
        } => {
            out.push(MsEvent::new(
                at_ms,
                update_from(&closure),
                (closure_ms * 0.4).min(20.0),
            ));
            out.push(MsEvent::new(
                at_ms + closure_ms,
                update_from(&burst),
                (burst_ms * 0.2).min(5.0),
            ));
            at_ms + closure_ms + burst_ms
        }
        SegmentKind::Aspirate => {
            let t = aspirate_targets(next);
            out.push(MsEvent::new(
                at_ms,
                update_from(&t),
                (seg.dur_ms * 0.4).min(35.0),
            ));
            at_ms + seg.dur_ms
        }
    }
}

/// Lower a phoneme sequence to a schedule. Returns the schedule and the total
/// duration in ms. Deterministic: same input, same events, always.
pub fn lower_sequence(phonemes: &[Phoneme], sample_rate: u32) -> (Schedule, f32) {
    let segs = specs(phonemes);
    let mut events = Vec::new();
    let mut t_ms = 0.0;
    for (i, seg) in segs.iter().enumerate() {
        let next = segs.get(i + 1).and_then(SegmentSpec::leading_targets);
        t_ms = lower_segment(seg, next, t_ms, &mut events);
    }
    (Schedule::from_ms_events(sample_rate, events), t_ms)
}

/// Render a phoneme sequence offline (short tail appended so final transients
/// decay inside the buffer).
pub fn render_phonemes(phonemes: &[Phoneme], sample_rate: u32) -> Vec<f32> {
    let (schedule, total_ms) = lower_sequence(phonemes, sample_rate);
    crate::render_schedule(schedule, sample_rate, (total_ms + 20.0) as u32)
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
            update_with_f0(&t, MEASUREMENT_F0_HZ),
            5.0,
        )],
    );
    crate::render_schedule(schedule, sample_rate, hold_ms)
}
