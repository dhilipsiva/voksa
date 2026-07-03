//! Sentence prosody as a deterministic schedule transform (phonology.md §9).
//!
//! Composition order matters: stressed-syllable duration stretching first
//! (it re-times everything), then declination against the NEW total, then the
//! stress F0 excursion + amplitude boost inside stressed spans (additive
//! above the declination baseline), then the optional xu terminal rise.
//! Declination is applied ADDITIVELY (`f0 += baseline(t) − BASE_F0_HZ`) so
//! the Phase-10 attitudinal overlay can compose on top.

use crate::schedule::UtteranceSchedule;

/// Options for [`apply_prosody`]. Declination and stress realization are
/// always on; the xu terminal rise is per-utterance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProsodyOptions {
    pub xu_rise: bool,
}

/// Utterance-initial F0 baseline (== schedule::BASE_F0_HZ).
pub const DECLINATION_START_HZ: f32 = 120.0;
/// Utterance-final F0 baseline.
pub const DECLINATION_END_HZ: f32 = 95.0;
/// Stressed syllables stretch to 1.5× duration (CLL-derived convention,
/// docs/formants.md).
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
    let _ = (schedule, opts);
    todo!("Phase 7 red checkpoint: the transform lands after the failing tests are committed")
}
