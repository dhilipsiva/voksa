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

    // ---- Phase-11 naturalness (identity values = byte-exact no-op; the
    // pinned defaults switch on at N-D; see [`Self::naturalness_off`]) ----
    /// Klatt flutter FL percent (0 = off): slow deterministic F0 wobble.
    pub flutter: f32,
    /// Baseline breathiness: aspiration added to voiced frames (0 = off).
    pub breath_aspiration: f32,
    /// Baseline open-quotient delta on every frame (0 = off).
    pub baseline_oq_delta: f32,
    /// Baseline spectral-tilt delta on every frame (0 = off).
    pub baseline_tilt_delta: f32,
    /// Intrinsic vowel F0 (Hz): high vowels +Δ, low vowels −Δ (0 = off).
    pub micro_f0_hz: f32,
    /// Obstruent F0 perturbation (Hz): post-voiceless vowel onset +Δ (voiced
    /// dip = −[`OBSTRUENT_DIP_RATIO`]·Δ), settling over [`MICRO_DECAY_MS`].
    pub obstruent_f0_hz: f32,
    /// Phrase-final rhyme lengthening multiplier (1 = off).
    pub final_lengthen: f32,
    /// Onset-cluster compression rate: window ×max(1−r·(k−1),
    /// [`CLUSTER_SHORTEN_FLOOR`]) for k cluster consonants (0 = off).
    pub cluster_shorten: f32,
    /// Duration-dependent formant undershoot toward the schwa center, max
    /// fraction at zero duration (reference [`UNDERSHOOT_REF_MS`]; 0 = off).
    pub undershoot: f32,
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
            flutter: FLUTTER_FL,
            breath_aspiration: BREATH_ASPIRATION,
            baseline_oq_delta: BASELINE_OQ_DELTA,
            baseline_tilt_delta: BASELINE_TILT_DELTA,
            micro_f0_hz: MICRO_F0_HZ,
            obstruent_f0_hz: OBSTRUENT_F0_HZ,
            final_lengthen: FINAL_LENGTHEN,
            cluster_shorten: CLUSTER_SHORTEN,
            undershoot: UNDERSHOOT,
        }
    }
}

impl ProsodyOptions {
    /// Every Phase-11 naturalness knob at its IDENTITY value (byte-exact
    /// no-op stages) with everything else default — the CP3 A/B "off" arm and
    /// the anchor of the frozen `snapshot_naturalness_off_*` contract.
    pub fn naturalness_off() -> Self {
        Self {
            flutter: 0.0,
            breath_aspiration: 0.0,
            baseline_oq_delta: 0.0,
            baseline_tilt_delta: 0.0,
            micro_f0_hz: 0.0,
            obstruent_f0_hz: 0.0,
            final_lengthen: 1.0,
            cluster_shorten: 0.0,
            undershoot: 0.0,
            ..Default::default()
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

// ---- Phase-11 naturalness pinned constants. IDENTITY until N-D flips the
// default voice ON (the flip is the one deliberate sound change of Phase 11;
// `naturalness_off()` stays at these identity values forever). ----
/// Klatt flutter FL percent.
pub const FLUTTER_FL: f32 = 0.0;
/// Baseline breathiness (aspiration on voiced frames).
pub const BREATH_ASPIRATION: f32 = 0.0;
/// Baseline open-quotient delta.
pub const BASELINE_OQ_DELTA: f32 = 0.0;
/// Baseline spectral-tilt delta.
pub const BASELINE_TILT_DELTA: f32 = 0.0;
/// Intrinsic vowel F0 magnitude (Hz).
pub const MICRO_F0_HZ: f32 = 0.0;
/// Post-voiceless obstruent F0 rise (Hz).
pub const OBSTRUENT_F0_HZ: f32 = 0.0;
/// Phrase-final rhyme lengthening.
pub const FINAL_LENGTHEN: f32 = 1.0;
/// Onset-cluster compression rate.
pub const CLUSTER_SHORTEN: f32 = 0.0;
/// Duration-dependent formant undershoot fraction.
pub const UNDERSHOOT: f32 = 0.0;

/// Voiced-obstruent F0 dip as a ratio of the voiceless rise (Klatt-ish).
pub const OBSTRUENT_DIP_RATIO: f32 = 0.6;
/// Obstruent perturbation settle time (ms).
pub const MICRO_DECAY_MS: f32 = 50.0;
/// Undershoot reference duration (ms): a segment this long or longer keeps
/// full formant quality (voksa's vowels run 100–160 ms, stressed rhymes ~225).
pub const UNDERSHOOT_REF_MS: f32 = 200.0;
/// Cluster-compression floor (a 4+ cluster never compresses below this).
pub const CLUSTER_SHORTEN_FLOOR: f32 = 0.6;

/// Apply sentence prosody to a compiled schedule. Deterministic: identical
/// input and options always yield the identical schedule.
pub fn apply_prosody(schedule: UtteranceSchedule, opts: &ProsodyOptions) -> UtteranceSchedule {
    let s = apply_cluster_shortening(schedule, opts.cluster_shorten);
    let s = apply_microprosody(s, opts.micro_f0_hz, opts.obstruent_f0_hz);
    let s = stretch_stressed_spans(s, opts.stress_duration_factor);
    let s = apply_declination(s, opts.declination_start_hz, opts.declination_end_hz);
    let s = apply_stress_excursion(s, opts.stress_f0_excursion_hz, opts.stress_amp_factor);
    let s = apply_final_lengthening(s, opts.final_lengthen);
    let s = if opts.xu_rise {
        apply_xu_rise(s, opts.xu_rise_hz)
    } else {
        s
    };
    scale_rate(s, opts.rate)
}

/// Compress onset-cluster windows (Phase-11 lever 4a; Klatt-1976-ish): a span
/// with k ≥ 2 onset cluster consonants (post-buffering) has its onset window
/// `[start, start + nucleus_off)` scaled by `max(1 − shorten·(k−1),
/// CLUSTER_SHORTEN_FLOOR)`. Identity at `shorten == 0`.
fn apply_cluster_shortening(s: UtteranceSchedule, shorten: f32) -> UtteranceSchedule {
    if shorten == 0.0 {
        return s;
    }
    // RED stub (P11 N-B): implementation lands with the failing tests.
    s
}

/// Microprosody (Phase-11 lever 3): intrinsic vowel F0 (high +Δ, low −Δ) and
/// obstruent perturbations on the following vowel onset (post-voiceless +Δ,
/// post-voiced −OBSTRUENT_DIP_RATIO·Δ) settling over MICRO_DECAY_MS via an
/// inserted event. Identity when both magnitudes are 0. Runs BEFORE the
/// stress stretch (offsets are additive and ride declination untouched).
fn apply_microprosody(
    s: UtteranceSchedule,
    micro_f0_hz: f32,
    obstruent_f0_hz: f32,
) -> UtteranceSchedule {
    if micro_f0_hz == 0.0 && obstruent_f0_hz == 0.0 {
        return s;
    }
    // RED stub (P11 N-B): implementation lands with the failing tests.
    s
}

/// Phrase-final lengthening (Phase-11 lever 4b): stretch the LAST span's
/// rhyme window by `factor`. Identity at 1.0. Runs before the xu rise so the
/// rise ramps across the real (lengthened) remainder.
fn apply_final_lengthening(s: UtteranceSchedule, factor: f32) -> UtteranceSchedule {
    if factor == 1.0 {
        return s;
    }
    // RED stub (P11 N-B): implementation lands with the failing tests.
    s
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
            micro: prevailing.micro,
        },
    );
    s
}
