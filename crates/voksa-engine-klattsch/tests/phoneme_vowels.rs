//! Phase-2 vowel acceptance: EACH of the 6 vowels, rendered steady for 300 ms,
//! must measure within tolerance of the docs/formants.md targets by BOTH
//! methods (FFT peak-picking AND LPC root-finding).
//!
//! Tolerances (docs/formants.md "Test tolerances"): F1 ±5% or ±50 Hz
//! (whichever larger), F2 ±5%, F3 ±8% (the documented F2/F3 band is ±5–8%;
//! F3 sits closest to the LPC post-downsample Nyquist so it gets the wide end).

use voksa_core::phonemes::{Phoneme, Vowel};
use voksa_engine_klattsch::{SAMPLE_RATE, render_steady_phoneme};
use voksa_testkit::{
    FormantCheck, assert_formants_tol, measure_formants_fft_harmonic, measure_formants_lpc,
};

/// Must match MEASUREMENT_F0_HZ in the adapter's steady-measurement renders.
const MEASUREMENT_F0: f32 = 105.0;

const TOL: [f32; 3] = [0.05, 0.05, 0.08];
const F1_FLOOR_HZ: f32 = 50.0;

fn check_vowel(v: Vowel, f1: f32, f2: f32, f3: f32) {
    let target = FormantCheck { f1, f2, f3 };
    let samples = render_steady_phoneme(Phoneme::Vowel(v), SAMPLE_RATE, 300);
    let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    assert!(
        peak < 0.85,
        "{v:?}: render must stay linear, peak {peak:.3}"
    );
    let fft = measure_formants_fft_harmonic(&samples, SAMPLE_RATE, MEASUREMENT_F0);
    let lpc = measure_formants_lpc(&samples, SAMPLE_RATE);
    eprintln!(
        "{v:?}: FFT {:.1}/{:.1}/{:.1}  LPC {:.1}/{:.1}/{:.1}  target {f1:.0}/{f2:.0}/{f3:.0}",
        fft.f1, fft.f2, fft.f3, lpc.f1, lpc.f2, lpc.f3
    );
    assert_formants_tol(&fft, &target, TOL, F1_FLOOR_HZ);
    assert_formants_tol(&lpc, &target, TOL, F1_FLOOR_HZ);
}

#[test]
fn vowel_a_formants() {
    check_vowel(Vowel::A, 730.0, 1090.0, 2440.0);
}

#[test]
fn vowel_e_formants() {
    check_vowel(Vowel::E, 530.0, 1840.0, 2480.0);
}

#[test]
fn vowel_i_formants() {
    check_vowel(Vowel::I, 270.0, 2290.0, 3010.0);
}

#[test]
fn vowel_o_formants() {
    check_vowel(Vowel::O, 570.0, 840.0, 2410.0);
}

#[test]
fn vowel_u_formants() {
    check_vowel(Vowel::U, 300.0, 870.0, 2240.0);
}

#[test]
fn vowel_y_formants() {
    check_vowel(Vowel::Y, 500.0, 1500.0, 2500.0);
}
