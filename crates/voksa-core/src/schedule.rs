//! voksa's engine-neutral schedule IR: sparse timed parameter events plus
//! syllable spans for the Phase-7 prosody transform.
//!
//! The frame vocabulary deliberately matches what the current engine consumes
//! (formant targets + voicing + aspiration + F0); OQ/TL/FL/DI arrive with the
//! Phase-10 attitudinal fork. Amplitudes are POSITIVE here — any engine
//! topology quirk (e.g. alternating parallel-branch polarity) is the
//! adapter's business.
//!
//! Timing conventions follow the reference engine frontend's proven shapes
//! (ported verbatim from the Phase-2 adapter so lowered output stays
//! byte-identical): steady segments get one event with transition
//! min(35 ms, dur·0.4); stops get a closure event then a burst event; a glide
//! is 25% onset / 50% ramp / 25% offset where the ramp is a single event
//! whose transition time IS the glide.

use crate::alloc::vec::Vec;
use crate::phonemes::{Formant, Phoneme, SegmentKind, SegmentSpec, Targets, specs};

/// Flat robotic baseline F0 until the prosody layer (Phase 7) transforms it.
pub const BASE_F0_HZ: f32 = 120.0;

/// Mandatory-pause silence length (CLL specifies none; phonology.md §9's
/// 50–150 ms band, middle chosen).
pub const PAUSE_MS: f32 = 100.0;

/// One parameter frame: what the voice should be doing from an event onward.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub f0_hz: f32,
    pub targets: Targets,
}

/// One timed event: reach `frame` by ramping over `transition_ms` starting at
/// `at_ms`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    pub at_ms: f32,
    pub transition_ms: f32,
    pub frame: Frame,
}

/// One syllable's time span, with the metadata prosody needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyllableSpan {
    pub start_ms: f32,
    pub dur_ms: f32,
    /// Offset from `start_ms` to the vowel-nucleus onset (0 for onsetless
    /// syllables; includes [h] aspiration and any onset-side epenthetic
    /// buffer). Stress stretching applies only to the rhyme —
    /// `[start_ms + nucleus_off_ms, start_ms + dur_ms)` — so onset consonant
    /// clusters keep unit rate (CP1: they otherwise smear).
    pub nucleus_off_ms: f32,
    pub word_index: usize,
    pub stressed: bool,
    /// False for y-nucleus / iy-uy / syllabic-consonant / buffer syllables.
    pub countable: bool,
}

/// A compiled utterance: the deterministic parameter schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct UtteranceSchedule {
    pub events: Vec<Event>,
    pub spans: Vec<SyllableSpan>,
    pub total_ms: f32,
}

/// Silence frame targets (pauses, voiceless closures).
pub fn silence_targets() -> Targets {
    Targets {
        formants: [
            Formant {
                freq_hz: 500.0,
                bw_hz: 90.0,
                amp: 0.0,
            },
            Formant {
                freq_hz: 1500.0,
                bw_hz: 110.0,
                amp: 0.0,
            },
            Formant {
                freq_hz: 2500.0,
                bw_hz: 150.0,
                amp: 0.0,
            },
        ],
        voicing: 0.0,
        aspiration: 0.0,
    }
}

/// Utterance-final [h] fallback shape (schwa-like); phonotactically the
/// apostrophe is intervocalic, so this only guards degenerate input.
const FALLBACK_SCHWA: Targets = Targets {
    formants: [
        Formant {
            freq_hz: 500.0,
            bw_hz: 90.0,
            amp: 0.5,
        },
        Formant {
            freq_hz: 1500.0,
            bw_hz: 110.0,
            amp: 0.3,
        },
        Formant {
            freq_hz: 2500.0,
            bw_hz: 150.0,
            amp: 0.15,
        },
    ],
    voicing: 0.0,
    aspiration: 1.0,
};

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

/// Schedule one segment starting at `at_ms`; push its events, return its end
/// time. `next` is the following segment's leading targets ([h] lookahead).
pub fn schedule_segment(
    seg: &SegmentSpec,
    next: Option<Targets>,
    f0_hz: f32,
    at_ms: f32,
    out: &mut Vec<Event>,
) -> f32 {
    let ev = |at_ms: f32, targets: Targets, transition_ms: f32| Event {
        at_ms,
        transition_ms,
        frame: Frame { f0_hz, targets },
    };
    match seg.kind {
        SegmentKind::Steady(t) => {
            out.push(ev(at_ms, t, (seg.dur_ms * 0.4).min(35.0)));
            at_ms + seg.dur_ms
        }
        SegmentKind::Glide { from, to } => {
            let onset_ms = seg.dur_ms * 0.25;
            let glide_ms = seg.dur_ms * 0.5;
            out.push(ev(at_ms, from, (onset_ms * 0.4).min(35.0)));
            out.push(ev(at_ms + onset_ms, to, glide_ms));
            at_ms + seg.dur_ms
        }
        SegmentKind::Stop {
            closure,
            burst,
            closure_ms,
            burst_ms,
        } => {
            out.push(ev(at_ms, closure, (closure_ms * 0.4).min(20.0)));
            out.push(ev(at_ms + closure_ms, burst, (burst_ms * 0.2).min(5.0)));
            at_ms + closure_ms + burst_ms
        }
        SegmentKind::Aspirate => {
            let t = aspirate_targets(next);
            out.push(ev(at_ms, t, (seg.dur_ms * 0.4).min(35.0)));
            at_ms + seg.dur_ms
        }
    }
}

/// Schedule a bare phoneme sequence (the Phase-2 path, kept byte-compatible
/// with the original adapter lowering). Returns events and total duration.
pub fn schedule_phonemes(phonemes: &[Phoneme], f0_hz: f32) -> (Vec<Event>, f32) {
    let segs = specs(phonemes);
    let mut events = Vec::new();
    let mut t_ms = 0.0;
    for (i, seg) in segs.iter().enumerate() {
        let next = segs.get(i + 1).and_then(SegmentSpec::leading_targets);
        t_ms = schedule_segment(seg, next, f0_hz, t_ms, &mut events);
    }
    (events, t_ms)
}
