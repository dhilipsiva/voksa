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
use crate::attitudinal::AttitudinalScope;
use crate::phonemes::{
    Consonant, Formant, Phoneme, SegmentKind, SegmentSpec, Targets, Vowel, specs,
};

/// Flat robotic baseline F0 until the prosody layer (Phase 7) transforms it.
pub const BASE_F0_HZ: f32 = 120.0;

/// Mandatory-pause silence length (CLL specifies none; phonology.md §9's
/// 50–150 ms band, middle chosen).
pub const PAUSE_MS: f32 = 100.0;

/// Modal (neutral) open quotient: the value the engine reproduces upstream
/// with (1.0 = no override). The Phase-10 attitudinal overlay nudges it.
pub const NEUTRAL_OQ: f32 = 1.0;
/// Modal spectral tilt (0.0 = flat, engine default).
pub const NEUTRAL_TILT: f32 = 0.0;
/// Modal diplophonia (0.0 = off).
pub const NEUTRAL_DI: f32 = 0.0;
/// Modal vibrato depth in Hz (0.0 = off).
pub const NEUTRAL_VIBRATO_HZ: f32 = 0.0;
/// Modal Klatt flutter (0.0 = off; FL percent, Phase 11).
pub const NEUTRAL_FLUTTER: f32 = 0.0;

/// Intrinsic-F0 vowel height class (Phase-11 microprosody: high vowels carry
/// slightly higher F0 than low vowels — Whalen & Levitt).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelHeight {
    /// i, u — intrinsic F0 raised.
    High,
    /// e, o, y, and the epenthetic buffer — no shift.
    Mid,
    /// a — intrinsic F0 lowered.
    Low,
}

/// The segment class an event was emitted from — the metadata the Phase-11
/// microprosody/coarticulation transforms need after compile (events otherwise
/// lose their phoneme identity). Travels WITH the event through every insert/
/// re-time (a parallel table would desync when transforms insert events).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroClass {
    /// Monophthong nucleus (incl. the buffer vowel).
    Vowel(VowelHeight),
    /// Diphthong nucleus: dynamic — no intrinsic F0, no undershoot (MVP).
    Diphthong,
    /// b d g v z j (both events of a voiced stop carry it).
    VoicedObstruent,
    /// p t k f s c x.
    VoicelessObstruent,
    /// l m n r, including syllabic nuclei.
    Sonorant,
    /// ' ([h]) — carries no perturbation of its own (MVP).
    Aspirate,
    /// Pause events — blocks obstruent perturbation across pauses.
    Silence,
}

/// The [`MicroClass`] a phoneme's events carry.
pub fn micro_class(p: Phoneme) -> MicroClass {
    match p {
        Phoneme::Vowel(v) => MicroClass::Vowel(match v {
            Vowel::I | Vowel::U => VowelHeight::High,
            Vowel::A => VowelHeight::Low,
            Vowel::E | Vowel::O | Vowel::Y => VowelHeight::Mid,
        }),
        Phoneme::Diphthong(..) => MicroClass::Diphthong,
        Phoneme::Consonant(c) => match c {
            Consonant::B
            | Consonant::D
            | Consonant::G
            | Consonant::V
            | Consonant::Z
            | Consonant::J => MicroClass::VoicedObstruent,
            Consonant::P
            | Consonant::T
            | Consonant::K
            | Consonant::F
            | Consonant::S
            | Consonant::C
            | Consonant::X => MicroClass::VoicelessObstruent,
            Consonant::L | Consonant::M | Consonant::N | Consonant::R => MicroClass::Sonorant,
        },
        Phoneme::H => MicroClass::Aspirate,
    }
}

/// One parameter frame: what the voice should be doing from an event onward.
///
/// `f0_hz` + `targets` are the modal (Phase 1–7) vocabulary; `oq`/`tilt`/`di`/
/// `vibrato_hz` are the Phase-10 voice-quality lanes. They default to their
/// NEUTRAL_* constants (modal voice) so every non-attitudinal frame lowers to
/// the exact pre-Phase-10 ParamUpdate (the adapter Option-gates them out).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub f0_hz: f32,
    pub targets: Targets,
    /// Glottal open-quotient multiplier (NEUTRAL_OQ = modal).
    pub oq: f32,
    /// Spectral tilt (NEUTRAL_TILT = flat).
    pub tilt: f32,
    /// Diplophonia amount 0..1 (NEUTRAL_DI = off).
    pub di: f32,
    /// Vibrato depth in Hz (NEUTRAL_VIBRATO_HZ = off).
    pub vibrato_hz: f32,
    /// Klatt flutter, FL percent (NEUTRAL_FLUTTER = off; Phase 11).
    pub flutter: f32,
}

impl Frame {
    /// A modal frame: neutral voice quality across all Phase-10/11 lanes.
    pub const fn modal(f0_hz: f32, targets: Targets) -> Self {
        Self {
            f0_hz,
            targets,
            oq: NEUTRAL_OQ,
            tilt: NEUTRAL_TILT,
            di: NEUTRAL_DI,
            vibrato_hz: NEUTRAL_VIBRATO_HZ,
            flutter: NEUTRAL_FLUTTER,
        }
    }
}

/// One timed event: reach `frame` by ramping over `transition_ms` starting at
/// `at_ms`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    pub at_ms: f32,
    pub transition_ms: f32,
    pub frame: Frame,
    /// The segment class this event was emitted from (Phase-11 microprosody /
    /// coarticulation metadata; the 1:1 lowering ignores it).
    pub micro: MicroClass,
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
    /// Cluster consonants before the nucleus, POST-buffering (a buffered
    /// cluster reports 1 — the buffer broke it). Phase-11 duration rules.
    pub onset_count: u8,
    /// Cluster consonants after the nucleus, post-buffering.
    pub coda_count: u8,
}

/// A compiled utterance: the deterministic parameter schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct UtteranceSchedule {
    pub events: Vec<Event>,
    pub spans: Vec<SyllableSpan>,
    pub total_ms: f32,
    /// Attitudinal (UI-cmavo) colorings detected at compile time; consumed by
    /// [`crate::attitudinal::apply_attitudinal`]. Empty for modal utterances.
    pub attitudinals: Vec<AttitudinalScope>,
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
/// time. `next` is the following segment's leading targets ([h] lookahead);
/// `micro` is the segment class its events carry (Phase-11 metadata).
pub fn schedule_segment(
    seg: &SegmentSpec,
    next: Option<Targets>,
    micro: MicroClass,
    f0_hz: f32,
    at_ms: f32,
    out: &mut Vec<Event>,
) -> f32 {
    let ev = |at_ms: f32, targets: Targets, transition_ms: f32| Event {
        at_ms,
        transition_ms,
        frame: Frame::modal(f0_hz, targets),
        micro,
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
        t_ms = schedule_segment(
            seg,
            next,
            micro_class(phonemes[i]),
            f0_hz,
            t_ms,
            &mut events,
        );
    }
    (events, t_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_frame_equals_pinned_constants() {
        // A frame built the modal way carries neutral voice quality across every
        // Phase-10 lane; the adapter Option-gates these out so modal frames lower
        // to the exact pre-Phase-10 ParamUpdate.
        let f = Frame::modal(BASE_F0_HZ, silence_targets());
        assert_eq!(f.oq, NEUTRAL_OQ);
        assert_eq!(f.tilt, NEUTRAL_TILT);
        assert_eq!(f.di, NEUTRAL_DI);
        assert_eq!(f.vibrato_hz, NEUTRAL_VIBRATO_HZ);
        assert_eq!(f.flutter, NEUTRAL_FLUTTER);
        assert_eq!(
            (
                NEUTRAL_OQ,
                NEUTRAL_TILT,
                NEUTRAL_DI,
                NEUTRAL_VIBRATO_HZ,
                NEUTRAL_FLUTTER
            ),
            (1.0, 0.0, 0.0, 0.0, 0.0)
        );
    }
}
