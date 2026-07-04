//! Sentence prosody as a deterministic schedule transform (phonology.md §9).
//!
//! Composition order matters: stressed-syllable RHYME stretching first (the
//! rhyme = nucleus onward; it re-times everything), then declination against
//! the NEW total, then the stress F0 excursion + amplitude boost inside the
//! WHOLE stressed span (additive above the declination baseline), then the
//! optional xu terminal rise. Stretching the rhyme only (not the onset
//! consonants) is the CP1 fix — whole-span stretch smeared onset clusters.
//! Declination is applied ADDITIVELY (`f0 += baseline(t) − BASE_F0_HZ`) so
//! the Phase-10 attitudinal overlay can compose on top.

use crate::alloc::vec::Vec;
use crate::schedule::{BASE_F0_HZ, Event, UtteranceSchedule};

/// Options for [`apply_prosody`]. Declination and stress realization are
/// always on; the xu terminal rise is per-utterance. The float fields are the
/// tunable knobs (demo tuning console) — they DEFAULT to the pinned constants
/// below, so `ProsodyOptions::default()` reproduces the phonology.md §9.1
/// convention exactly (and every snapshot stays byte-identical).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProsodyOptions {
    pub xu_rise: bool,
    /// Utterance-initial F0 baseline (Hz).
    pub declination_start_hz: f32,
    /// Utterance-final F0 baseline (Hz).
    pub declination_end_hz: f32,
    /// Stressed-rhyme duration multiplier.
    pub stress_duration_factor: f32,
    /// F0 excursion above the baseline inside stressed spans (Hz).
    pub stress_f0_excursion_hz: f32,
    /// Amplitude multiplier inside stressed spans.
    pub stress_amp_factor: f32,
    /// xu terminal-rise magnitude (Hz).
    pub xu_rise_hz: f32,
    /// Global tempo: all timings are scaled by 1/rate (>1 = faster). 1.0 = default.
    pub rate: f32,
}

impl Default for ProsodyOptions {
    fn default() -> Self {
        Self {
            xu_rise: false,
            declination_start_hz: DECLINATION_START_HZ,
            declination_end_hz: DECLINATION_END_HZ,
            stress_duration_factor: STRESS_DURATION_FACTOR,
            stress_f0_excursion_hz: STRESS_F0_EXCURSION_HZ,
            stress_amp_factor: STRESS_AMP_FACTOR,
            xu_rise_hz: XU_RISE_HZ,
            rate: 1.0,
        }
    }
}

/// Utterance-initial F0 baseline (== schedule::BASE_F0_HZ).
pub const DECLINATION_START_HZ: f32 = 120.0;
/// Utterance-final F0 baseline.
pub const DECLINATION_END_HZ: f32 = 95.0;
/// A stressed syllable's rhyme (nucleus onward) stretches to 1.5× duration;
/// its onset consonants keep unit rate (CLL-derived convention, phonology.md
/// §9.1).
pub const STRESS_DURATION_FACTOR: f32 = 1.5;
/// Stress F0 excursion above the declination baseline (middle of the
/// documented +10–30 Hz band).
pub const STRESS_F0_EXCURSION_HZ: f32 = 20.0;
/// Stress amplitude boost (formant amplitudes, linear).
pub const STRESS_AMP_FACTOR: f32 = 1.2;
/// xu terminal rise applied across the final syllable.
pub const XU_RISE_HZ: f32 = 25.0;

/// Apply sentence prosody to a compiled schedule. Deterministic: identical
/// input and options always yield the identical schedule.
pub fn apply_prosody(schedule: UtteranceSchedule, opts: &ProsodyOptions) -> UtteranceSchedule {
    let s = stretch_stressed_spans(schedule, opts.stress_duration_factor);
    let s = apply_declination(s, opts.declination_start_hz, opts.declination_end_hz);
    let s = apply_stress_excursion(s, opts.stress_f0_excursion_hz, opts.stress_amp_factor);
    let s = if opts.xu_rise {
        apply_xu_rise(s, opts.xu_rise_hz)
    } else {
        s
    };
    scale_rate(s, opts.rate)
}

/// Global tempo: scale every timing by `1/rate` (rate 1.0 = exact identity, so
/// default schedules are byte-identical). rate > 1 speeds up.
fn scale_rate(mut s: UtteranceSchedule, rate: f32) -> UtteranceSchedule {
    if rate == 1.0 || rate <= 0.0 {
        return s;
    }
    let k = 1.0 / rate;
    for e in &mut s.events {
        e.at_ms *= k;
        e.transition_ms *= k;
    }
    for sp in &mut s.spans {
        sp.start_ms *= k;
        sp.dur_ms *= k;
        sp.nucleus_off_ms *= k;
    }
    s.total_ms *= k;
    s
}

/// Span-membership epsilon: span ends and following event times are
/// independent f32 accumulations that can differ by ULPs.
const EPS_MS: f32 = 1e-3;

/// Whole-syllable windows (start .. end): the F0 excursion and amplitude
/// boost cover the entire stressed syllable, onset consonants included.
fn stressed_windows(s: &UtteranceSchedule) -> Vec<(f32, f32)> {
    let mut w: Vec<(f32, f32)> = s
        .spans
        .iter()
        .filter(|sp| sp.stressed)
        .map(|sp| (sp.start_ms, sp.start_ms + sp.dur_ms))
        .collect();
    w.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite span times"));
    w
}

/// Rhyme windows (nucleus onset .. end): only the rhyme stretches, so onset
/// consonant clusters keep unit rate (CP1 — they otherwise smear).
fn stressed_stretch_windows(s: &UtteranceSchedule) -> Vec<(f32, f32)> {
    let mut w: Vec<(f32, f32)> = s
        .spans
        .iter()
        .filter(|sp| sp.stressed)
        .map(|sp| (sp.start_ms + sp.nucleus_off_ms, sp.start_ms + sp.dur_ms))
        .collect();
    w.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite span times"));
    w
}

fn inside(at_ms: f32, (ws, we): (f32, f32)) -> bool {
    at_ms >= ws - EPS_MS && at_ms < we - EPS_MS
}

/// Stretch every stressed syllable's RHYME (nucleus onward) to
/// [`STRESS_DURATION_FACTOR`]×, shifting all later material (events, spans,
/// pauses, total) by the added time. Onset consonants keep unit rate — the
/// stretch window opens at the nucleus, not the span start (CP1 fix).
fn stretch_stressed_spans(mut s: UtteranceSchedule, factor: f32) -> UtteranceSchedule {
    let windows = stressed_stretch_windows(&s);
    let map_time = |t: f32| -> f32 {
        let mut delta = 0.0f32;
        for (ws, we) in &windows {
            if t >= *we - EPS_MS {
                delta += (we - ws) * (factor - 1.0);
            } else if t > *ws - EPS_MS {
                return ws + delta + (t - ws) * factor;
            } else {
                break;
            }
        }
        t + delta
    };
    for e in &mut s.events {
        let in_stressed = windows.iter().any(|w| inside(e.at_ms, *w));
        e.at_ms = map_time(e.at_ms);
        if in_stressed {
            e.transition_ms *= factor;
        }
    }
    for sp in &mut s.spans {
        let end = map_time(sp.start_ms + sp.dur_ms);
        sp.start_ms = map_time(sp.start_ms);
        sp.dur_ms = end - sp.start_ms;
    }
    s.total_ms = map_time(s.total_ms);
    s
}

/// Additive linear declination: baseline falls [`DECLINATION_START_HZ`] →
/// [`DECLINATION_END_HZ`] across the (post-stretch) utterance.
fn apply_declination(mut s: UtteranceSchedule, start_hz: f32, end_hz: f32) -> UtteranceSchedule {
    let total = s.total_ms.max(1.0);
    for e in &mut s.events {
        let baseline = start_hz + (end_hz - start_hz) * (e.at_ms / total);
        e.frame.f0_hz += baseline - BASE_F0_HZ;
    }
    s
}

/// +F0 excursion and amplitude boost inside stressed spans only.
fn apply_stress_excursion(
    mut s: UtteranceSchedule,
    excursion_hz: f32,
    amp_factor: f32,
) -> UtteranceSchedule {
    let windows = stressed_windows(&s);
    for e in &mut s.events {
        if windows.iter().any(|w| inside(e.at_ms, *w)) {
            e.frame.f0_hz += excursion_hz;
            for f in &mut e.frame.targets.formants {
                f.amp *= amp_factor;
            }
        }
    }
    s
}

/// Insert one rise event inside the final syllable: the prevailing frame's
/// targets, f0 raised by [`XU_RISE_HZ`], ramped across the span remainder.
fn apply_xu_rise(mut s: UtteranceSchedule, rise_hz: f32) -> UtteranceSchedule {
    let Some(last_span) = s
        .spans
        .iter()
        .max_by(|a, b| a.start_ms.partial_cmp(&b.start_ms).expect("finite"))
        .copied()
    else {
        return s;
    };
    let span_end = last_span.start_ms + last_span.dur_ms;
    let rise_at = last_span.start_ms + last_span.dur_ms * 0.25;
    let Some(prevailing) = s
        .events
        .iter()
        .rfind(|e| e.at_ms <= rise_at + EPS_MS)
        .copied()
    else {
        return s;
    };
    // Later events inside the final span (e.g. the vowel after a sonorant
    // onset) would otherwise re-set F0 back down: carry the rise on them.
    for e in &mut s.events {
        if e.at_ms > rise_at && e.at_ms < span_end - EPS_MS {
            e.frame.f0_hz += rise_hz;
        }
    }
    let mut frame = prevailing.frame;
    frame.f0_hz += rise_hz;
    let idx = s.events.partition_point(|e| e.at_ms <= rise_at);
    s.events.insert(
        idx,
        Event {
            at_ms: rise_at,
            transition_ms: (span_end - rise_at).max(1.0),
            frame,
        },
    );
    s
}
