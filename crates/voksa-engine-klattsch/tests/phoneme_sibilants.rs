//! Phase-2 sibilant/fricative acceptance: the frication-energy centroid of
//! each rendered fricative must fall in its documented band (docs/formants.md
//! "Noise sources"). The centroid is measured above 1000 Hz so a voiced
//! fricative's voicing bar does not drag it out of the frication band;
//! voicing itself is asserted separately via low-band energy ratio.

use voksa_core::phonemes::{Consonant, Phoneme};
use voksa_engine_klattsch::{SAMPLE_RATE, render_steady_phoneme};
use voksa_testkit::{band_energy, band_energy_centroid};

const WIN: usize = 4096;

fn render(c: Consonant) -> Vec<f32> {
    render_steady_phoneme(Phoneme::Consonant(c), SAMPLE_RATE, 300)
}

fn frication_centroid(samples: &[f32]) -> f32 {
    band_energy_centroid(samples, SAMPLE_RATE, 1000.0, 12_000.0, WIN)
}

/// Fraction of spectral energy below 300 Hz — the voicing-bar region at
/// F0 = 120 Hz. Voiced fricatives show it; unvoiced ones do not.
fn low_band_ratio(samples: &[f32]) -> f32 {
    let low = band_energy(samples, SAMPLE_RATE, 60.0, 300.0, WIN);
    let total = band_energy(samples, SAMPLE_RATE, 60.0, 12_000.0, WIN);
    low / total.max(f32::EPSILON)
}

fn check_centroid(c: Consonant, lo: f32, hi: f32) {
    let samples = render(c);
    let centroid = frication_centroid(&samples);
    eprintln!("{c:?}: frication centroid {centroid:.0} Hz (band {lo:.0}-{hi:.0})");
    assert!(
        centroid >= lo && centroid <= hi,
        "{c:?}: centroid {centroid:.0} Hz outside band {lo:.0}-{hi:.0} Hz"
    );
}

#[test]
fn s_centroid_high_band() {
    check_centroid(Consonant::S, 4000.0, 8000.0);
}

#[test]
fn z_centroid_high_band() {
    check_centroid(Consonant::Z, 4000.0, 8000.0);
}

#[test]
fn c_centroid_postalveolar_band() {
    check_centroid(Consonant::C, 2500.0, 3500.0);
}

#[test]
fn j_centroid_postalveolar_band() {
    check_centroid(Consonant::J, 2500.0, 3500.0);
}

#[test]
fn x_centroid_velar_band() {
    check_centroid(Consonant::X, 1500.0, 2500.0);
}

#[test]
fn voicing_split_matches_phonology() {
    // z j voiced; s c x unvoiced (docs/phonology.md §1).
    let voiced = [Consonant::Z, Consonant::J];
    let unvoiced = [Consonant::S, Consonant::C, Consonant::X];
    for c in voiced {
        let r = low_band_ratio(&render(c));
        eprintln!("{c:?}: low-band ratio {r:.4} (voiced, expect high)");
        assert!(
            r > 0.05,
            "{c:?} is voiced: expected low-band energy, got {r:.4}"
        );
    }
    for c in unvoiced {
        let r = low_band_ratio(&render(c));
        eprintln!("{c:?}: low-band ratio {r:.4} (unvoiced, expect low)");
        assert!(
            r < 0.02,
            "{c:?} is unvoiced: unexpected low-band energy {r:.4}"
        );
    }
}

#[test]
fn f_and_v_render_weak_broadband() {
    // docs/formants.md: f = weak flat broadband; v = as f + voicing.
    let f = render(Consonant::F);
    let v = render(Consonant::V);
    let f_total = band_energy(&f, SAMPLE_RATE, 500.0, 12_000.0, WIN);
    let s_total = band_energy(&render(Consonant::S), SAMPLE_RATE, 500.0, 12_000.0, WIN);
    assert!(f_total > 0.0, "f must be audible");
    assert!(
        f_total < s_total,
        "f is WEAK broadband: expected less frication energy than s"
    );
    assert!(low_band_ratio(&f) < 0.02, "f is unvoiced");
    assert!(low_band_ratio(&v) > 0.05, "v is voiced");
}
