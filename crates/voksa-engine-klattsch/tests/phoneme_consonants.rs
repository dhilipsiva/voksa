//! Phase-2 stop/nasal/liquid/[h] acceptance (docs/formants.md sections
//! "Stops", "Nasals and liquids", and the apostrophe rule).

use voksa_core::phonemes::{Consonant, Phoneme, SegmentKind, Vowel, spec};
use voksa_engine_klattsch::{SAMPLE_RATE, render_phonemes, render_steady_phoneme};
use voksa_testkit::{band_energy, band_energy_centroid, band_peak_hz};

fn ms_to_samples(ms: f32) -> usize {
    (ms * SAMPLE_RATE as f32 / 1000.0) as usize
}

fn rms(samples: &[f32]) -> f32 {
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len().max(1) as f32).sqrt()
}

fn stop_timing(c: Consonant) -> (f32, f32) {
    match spec(Phoneme::Consonant(c)).kind {
        SegmentKind::Stop {
            closure_ms,
            burst_ms,
            ..
        } => (closure_ms, burst_ms),
        _ => panic!("{c:?} is not a stop"),
    }
}

/// Interior of the closure (skipping the leading transition) and the burst.
fn stop_slices(c: Consonant) -> (Vec<f32>, Vec<f32>) {
    let (closure_ms, burst_ms) = stop_timing(c);
    let samples = render_phonemes(&[Phoneme::Consonant(c)], SAMPLE_RATE);
    let closure = samples[ms_to_samples(25.0)..ms_to_samples(closure_ms - 5.0)].to_vec();
    let burst_start = ms_to_samples(closure_ms);
    let burst = samples[burst_start..burst_start + ms_to_samples(burst_ms)].to_vec();
    (closure, burst)
}

fn check_burst_centroid(c: Consonant, lo: f32, hi: f32) {
    let (_, burst) = stop_slices(c);
    let centroid = band_energy_centroid(&burst, SAMPLE_RATE, 300.0, 12_000.0, 1024);
    eprintln!("{c:?}: burst centroid {centroid:.0} Hz (band {lo:.0}-{hi:.0})");
    assert!(
        centroid >= lo && centroid <= hi,
        "{c:?}: burst centroid {centroid:.0} Hz outside {lo:.0}-{hi:.0} Hz"
    );
}

#[test]
fn alveolar_bursts_are_high() {
    // d/t: burst energy ~alveolar region, 3000-4000 Hz emphasis.
    check_burst_centroid(Consonant::T, 3000.0, 4000.0);
    check_burst_centroid(Consonant::D, 3000.0, 4000.0);
}

#[test]
fn velar_bursts_are_compact_mid() {
    // g/k: compact mid burst near the velar locus (2000-2300 Hz ± skirts).
    check_burst_centroid(Consonant::K, 1800.0, 2600.0);
    check_burst_centroid(Consonant::G, 1800.0, 2600.0);
}

#[test]
fn bilabial_bursts_are_diffuse_low() {
    let (_, burst_b) = stop_slices(Consonant::B);
    let (_, burst_p) = stop_slices(Consonant::P);
    for (name, burst) in [("b", burst_b), ("p", burst_p)] {
        let centroid = band_energy_centroid(&burst, SAMPLE_RATE, 300.0, 12_000.0, 1024);
        eprintln!("{name}: burst centroid {centroid:.0} Hz (expect < 1500)");
        assert!(
            centroid < 1500.0,
            "{name}: bilabial burst should be low-frequency, got {centroid:.0} Hz"
        );
    }
}

#[test]
fn voiceless_closures_are_silent_voiced_have_voice_bar() {
    for c in [Consonant::P, Consonant::T, Consonant::K] {
        let (closure, _) = stop_slices(c);
        let r = rms(&closure);
        eprintln!("{c:?} closure rms {r:.5} (voiceless, expect ~0)");
        assert!(
            r < 0.005,
            "{c:?}: voiceless closure must be silent, rms {r:.5}"
        );
    }
    for c in [Consonant::B, Consonant::D, Consonant::G] {
        let (closure, _) = stop_slices(c);
        let r = rms(&closure);
        eprintln!("{c:?} closure rms {r:.5} (voiced, expect voice bar)");
        assert!(
            r > 0.01,
            "{c:?}: voiced closure needs a voice bar, rms {r:.5}"
        );
    }
}

#[test]
fn nasals_have_low_f1_and_attenuated_highs() {
    for c in [Consonant::M, Consonant::N] {
        let samples = render_steady_phoneme(Phoneme::Consonant(c), SAMPLE_RATE, 300);
        let murmur = band_peak_hz(&samples, SAMPLE_RATE, 150.0, 400.0, 4096);
        let high = band_energy(&samples, SAMPLE_RATE, 1500.0, 3500.0, 4096);
        let low = band_energy(&samples, SAMPLE_RATE, 150.0, 500.0, 4096);
        eprintln!(
            "{c:?}: murmur peak {murmur:.0} Hz, high/low energy {:.4}",
            high / low
        );
        assert!(
            (200.0..=350.0).contains(&murmur),
            "{c:?}: nasal murmur peak {murmur:.0} Hz outside 200-350 Hz"
        );
        assert!(
            high / low < 0.2,
            "{c:?}: nasal highs must be attenuated (anti-resonance approximation)"
        );
    }
}

#[test]
fn l_has_lateral_f2() {
    let samples = render_steady_phoneme(Phoneme::Consonant(Consonant::L), SAMPLE_RATE, 300);
    let f2 = band_peak_hz(&samples, SAMPLE_RATE, 1000.0, 1600.0, 4096);
    eprintln!("l: F2 {f2:.0} Hz (target 1300)");
    assert!(
        (f2 - 1300.0).abs() <= 130.0,
        "l F2 {f2:.0} Hz not within ±10% of 1300 Hz"
    );
}

#[test]
fn r_has_lowered_f3() {
    let samples = render_steady_phoneme(Phoneme::Consonant(Consonant::R), SAMPLE_RATE, 300);
    let f3 = band_peak_hz(&samples, SAMPLE_RATE, 1450.0, 2100.0, 4096);
    eprintln!("r: F3 {f3:.0} Hz (rhotic signature 1600-1900)");
    assert!(
        (1600.0..=1900.0).contains(&f3),
        "r F3 {f3:.0} Hz outside the rhotic 1600-1900 Hz band"
    );
}

#[test]
fn h_is_unvoiced_aspiration_shaped_by_following_vowel() {
    let samples = render_phonemes(&[Phoneme::H, Phoneme::Vowel(Vowel::A)], SAMPLE_RATE);
    let h_ms = spec(Phoneme::H).dur_ms;
    let h_part = &samples[ms_to_samples(15.0)..ms_to_samples(h_ms - 5.0)];
    let low = band_energy(h_part, SAMPLE_RATE, 60.0, 300.0, 1024);
    let total = band_energy(h_part, SAMPLE_RATE, 60.0, 12_000.0, 1024);
    assert!(total > 0.0, "[h] must be audible");
    assert!(
        low / total < 0.05,
        "[h] must be unvoiced (low-band ratio {:.4})",
        low / total
    );
    let centroid = band_energy_centroid(h_part, SAMPLE_RATE, 300.0, 4000.0, 1024);
    eprintln!("[h] before /a/: centroid {centroid:.0} Hz (expect near /a/ formants)");
    assert!(
        (500.0..=2000.0).contains(&centroid),
        "[h] centroid {centroid:.0} Hz should sit among /a/ formants (500-2000 Hz)"
    );
}
