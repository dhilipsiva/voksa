//! Attitudinal (UI-cmavo) voice-quality overlay — Phase 10 (CP2).
//!
//! INVENTED / NON-NORMATIVE. The CLL mandates NO acoustic realization for the
//! attitudinal cmavo: a `.ui` is defined by its *meaning* (joy), not a pitch or
//! voice quality. Everything here — the seven categories, their deviation
//! vectors, the intensity multipliers, the word-scope rule — is voksa's own
//! invention (seeded by docs/research/02-architecture-v2.md §11), expressed as
//! ratios/offsets, not Lojban-validated prosody.
//!
//! The overlay is a deterministic schedule transform that COMPOSES ON TOP of
//! [`crate::prosody::apply_prosody`]: detection runs in [`crate::compiler`]
//! (which has the word analyses + `word_index`) and stores
//! [`AttitudinalScope`]s on the [`UtteranceSchedule`]; [`apply_attitudinal`]
//! then colors each target word's event window. F0 shifts are additive Hz
//! (voksa-core is `no_std` with no transcendental math — same arithmetic-only
//! discipline as the prosody layer).

use crate::alloc::vec::Vec;
use crate::schedule::{SyllableSpan, UtteranceSchedule};

/// The seven attitudinal categories voksa realizes acoustically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttitudinalKind {
    /// `.ui` — joy: raised, wider pitch, brighter, a touch faster.
    Joy,
    /// `.uu` — sadness/pity: lowered, flat (monotone), slower, breathy + dark.
    Sadness,
    /// `.oi` — complaint/pain: slightly low, creaky (diplophonia), tighter.
    Complaint,
    /// `.ii` — fear: high, fast, fluttering (vibrato), breathy.
    Fear,
    /// `.o'o` — patience/calm: low and very flat (monotone).
    Patience,
    /// `.au` — desire: raised, breathy, forward.
    Desire,
    /// `.o'onai` (`.o'o` + `nai`) — anger: raised, wide, fast, tense + harsh.
    Anger,
}

/// One attitudinal coloring: which word it lands on, its category, and the
/// intensity multiplier applied to the deviation vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttitudinalScope {
    /// The `word_index` of the CONTENT word the emotion colors (the scope
    /// resolver's target — usually the attitudinal's preceding word).
    pub word_index: usize,
    pub kind: AttitudinalKind,
    /// Intensity multiplier on the whole deviation vector. `nai` = −1.0 flips
    /// polarity; `cai`/`sai`/`ru'e` scale down; bare = 1.0.
    pub intensity: f32,
}

/// A voice-quality deviation from the modal baseline, pre-intensity. Additive
/// nudges (`f0_mean_hz`, the `d_*` deltas) scale linearly by intensity; the
/// multipliers (`f0_range_mult`, `rate_mult`) interpolate toward 1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deviation {
    /// Additive mean-F0 shift in Hz (+ = higher).
    pub f0_mean_hz: f32,
    /// Multiplier on the local F0 excursion about the word mean (>1 wider,
    /// <1 flatter → monotone).
    pub f0_range_mult: f32,
    /// Global duration/tempo multiplier (>1 = slower/longer).
    pub rate_mult: f32,
    /// Open-quotient delta (+ = breathier, − = creakier/tenser).
    pub d_oq: f32,
    /// Spectral-tilt delta (+ = brighter/tenser, − = darker).
    pub d_tilt: f32,
    /// Diplophonia added, 0..1 (creak / vocal fry).
    pub d_di: f32,
    /// Vibrato depth added, Hz (flutter).
    pub d_vibrato_hz: f32,
    /// Breathiness (aspiration) added to voiced frames, 0..1.
    pub d_aspiration: f32,
}

impl AttitudinalKind {
    /// The invented deviation vector for this category (v2 §11-derived).
    pub const fn deviation(self) -> Deviation {
        match self {
            AttitudinalKind::Joy => Deviation {
                f0_mean_hz: 14.0,
                f0_range_mult: 1.4,
                rate_mult: 0.95,
                d_oq: 0.05,
                d_tilt: 0.15,
                d_di: 0.0,
                d_vibrato_hz: 0.0,
                d_aspiration: 0.0,
            },
            AttitudinalKind::Sadness => Deviation {
                f0_mean_hz: -12.0,
                f0_range_mult: 0.6,
                rate_mult: 1.15,
                d_oq: 0.20,
                d_tilt: -0.20,
                d_di: 0.0,
                d_vibrato_hz: 0.0,
                d_aspiration: 0.15,
            },
            AttitudinalKind::Complaint => Deviation {
                f0_mean_hz: -4.0,
                f0_range_mult: 0.9,
                rate_mult: 1.05,
                d_oq: -0.10,
                d_tilt: -0.05,
                d_di: 0.10,
                d_vibrato_hz: 0.0,
                d_aspiration: 0.0,
            },
            AttitudinalKind::Fear => Deviation {
                f0_mean_hz: 18.0,
                f0_range_mult: 1.2,
                rate_mult: 0.90,
                d_oq: 0.05,
                d_tilt: 0.05,
                d_di: 0.0,
                d_vibrato_hz: 6.0,
                d_aspiration: 0.10,
            },
            AttitudinalKind::Patience => Deviation {
                f0_mean_hz: -6.0,
                f0_range_mult: 0.3,
                rate_mult: 1.0,
                d_oq: 0.0,
                d_tilt: -0.05,
                d_di: 0.0,
                d_vibrato_hz: 0.0,
                d_aspiration: 0.0,
            },
            AttitudinalKind::Desire => Deviation {
                f0_mean_hz: 8.0,
                f0_range_mult: 1.1,
                rate_mult: 1.0,
                d_oq: 0.10,
                d_tilt: 0.0,
                d_di: 0.0,
                d_vibrato_hz: 0.0,
                d_aspiration: 0.08,
            },
            AttitudinalKind::Anger => Deviation {
                f0_mean_hz: 10.0,
                f0_range_mult: 1.3,
                rate_mult: 0.90,
                d_oq: -0.20,
                d_tilt: 0.25,
                d_di: 0.15,
                d_vibrato_hz: 0.0,
                d_aspiration: 0.0,
            },
        }
    }
}

/// Map a lowercased word (the tokenizer strips the leading `.`, so `.ui` →
/// `ui`) to its attitudinal category, or `None` if it is not one voksa
/// realizes. `.o'onai` is recognized as a fused single token (anger); the
/// two-word `.o'o nai` form resolves as Patience × −1 via the intensity path.
pub fn attitudinal_kind(lowered: &str) -> Option<AttitudinalKind> {
    Some(match lowered {
        "ui" => AttitudinalKind::Joy,
        "uu" => AttitudinalKind::Sadness,
        "oi" => AttitudinalKind::Complaint,
        "ii" => AttitudinalKind::Fear,
        "o'o" => AttitudinalKind::Patience,
        "au" => AttitudinalKind::Desire,
        "o'onai" => AttitudinalKind::Anger,
        _ => return None,
    })
}

/// Map an intensity cmavo following an attitudinal to its multiplier. `nai`
/// flips polarity (−1.0); the rest scale the deviation down.
pub fn intensity_mult(lowered: &str) -> Option<f32> {
    Some(match lowered {
        "cai" => 1.0,
        "sai" => 0.7,
        "ru'e" => 0.4,
        "nai" => -1.0,
        _ => return None,
    })
}

/// Span-membership epsilon (mirrors [`crate::prosody`]): span ends and event
/// times are independent f32 accumulations that can differ by ULPs.
const EPS_MS: f32 = 1e-3;

fn in_window(at_ms: f32, start_ms: f32, end_ms: f32) -> bool {
    at_ms >= start_ms - EPS_MS && at_ms < end_ms - EPS_MS
}

/// The [start, end) time window of a word: min span start .. max span end over
/// the spans with `word_index`.
fn word_window(spans: &[SyllableSpan], word_index: usize) -> Option<(f32, f32)> {
    let mut start = f32::INFINITY;
    let mut end = f32::NEG_INFINITY;
    for sp in spans.iter().filter(|s| s.word_index == word_index) {
        start = start.min(sp.start_ms);
        end = end.max(sp.start_ms + sp.dur_ms);
    }
    if start.is_finite() && end.is_finite() {
        Some((start, end))
    } else {
        None
    }
}

/// Scale every timing by `mult` (>1 slows/lengthens). `mult == 1.0` is exact
/// identity so a rate-neutral overlay leaves timings byte-identical.
fn scale_time(mut s: UtteranceSchedule, mult: f32) -> UtteranceSchedule {
    if mult == 1.0 || mult <= 0.0 {
        return s;
    }
    for e in &mut s.events {
        e.at_ms *= mult;
        e.transition_ms *= mult;
    }
    for sp in &mut s.spans {
        sp.start_ms *= mult;
        sp.dur_ms *= mult;
        sp.nucleus_off_ms *= mult;
    }
    s.total_ms *= mult;
    s
}

/// Apply the attitudinal overlay stored on `s` (by the compiler) to its event
/// schedule. Deterministic: identical input always yields the identical
/// schedule. No-op when there are no attitudinals.
///
/// Per scope, over the target word's event window: re-center the F0 excursion
/// about the word mean by `f0_range_mult`, add the mean Hz shift, and set the
/// voice-quality lanes (`oq`/`tilt`/`di`/`vibrato_hz`) + breathiness — all
/// scaled by intensity. A single global tempo scale (the first, dominant
/// scope's `rate_mult`) is applied last; per-word rate is a documented MVP
/// limitation.
pub fn apply_attitudinal(mut s: UtteranceSchedule) -> UtteranceSchedule {
    if s.attitudinals.is_empty() {
        return s;
    }
    let scopes: Vec<AttitudinalScope> = s.attitudinals.clone();

    for scope in &scopes {
        let dev = scope.kind.deviation();
        let it = scope.intensity;
        let Some((w_start, w_end)) = word_window(&s.spans, scope.word_index) else {
            continue;
        };

        // Word-mean F0 (for the range re-centering).
        let mut sum = 0.0f32;
        let mut n = 0u32;
        for e in &s.events {
            if in_window(e.at_ms, w_start, w_end) {
                sum += e.frame.f0_hz;
                n += 1;
            }
        }
        let mean = if n > 0 { sum / n as f32 } else { 0.0 };

        // Multipliers interpolate toward 1.0 with intensity; additive nudges
        // scale linearly. nai (it = −1) inverts both about their neutral point.
        let range_mult = 1.0 + (dev.f0_range_mult - 1.0) * it;
        let mean_shift = dev.f0_mean_hz * it;

        for e in &mut s.events {
            if !in_window(e.at_ms, w_start, w_end) {
                continue;
            }
            let f = e.frame.f0_hz;
            e.frame.f0_hz = mean + (f - mean) * range_mult + mean_shift;
            e.frame.oq = (e.frame.oq + dev.d_oq * it).clamp(0.2, 2.0);
            e.frame.tilt = (e.frame.tilt + dev.d_tilt * it).clamp(-0.95, 0.95);
            e.frame.di = (e.frame.di + dev.d_di * it).clamp(0.0, 1.0);
            e.frame.vibrato_hz = (e.frame.vibrato_hz + dev.d_vibrato_hz * it).max(0.0);
            // Breathiness only makes sense on a voiced frame.
            if e.frame.targets.voicing > 0.0 {
                e.frame.targets.aspiration =
                    (e.frame.targets.aspiration + dev.d_aspiration * it).clamp(0.0, 1.0);
            }
        }
    }

    // Global tempo from the dominant (first) attitudinal.
    let dom = &scopes[0];
    let rate_mult = 1.0 + (dom.kind.deviation().rate_mult - 1.0) * dom.intensity;
    scale_time(s, rate_mult)
}
