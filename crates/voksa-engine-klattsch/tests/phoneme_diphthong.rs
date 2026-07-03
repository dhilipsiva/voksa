//! Phase-2 diphthong acceptance: in /ai/, F2 must move monotonically from the
//! /a/ locus (1090 Hz) toward the /i/ locus (2290 Hz) across the glide.

use voksa_core::phonemes::{Phoneme, Vowel};
use voksa_engine_klattsch::{SAMPLE_RATE, render_phonemes};
use voksa_testkit::track_band_peak;

#[test]
fn ai_f2_glides_monotonically_from_a_to_i() {
    let samples = render_phonemes(
        &[Phoneme::diphthong(Vowel::A, Vowel::I).expect("ai is a valid diphthong")],
        SAMPLE_RATE,
    );
    let track = track_band_peak(&samples, SAMPLE_RATE, 800.0, 2600.0, 1024, 512);
    eprintln!(
        "F2 track: {:?}",
        track.iter().map(|f| f.round()).collect::<Vec<_>>()
    );
    assert!(track.len() >= 8, "need enough frames across 200 ms");

    // Skip the first two frames (onset transition from engine defaults while
    // amplitudes ramp up from silence).
    let steady = &track[2..];
    let first = steady.first().unwrap();
    let last = steady.last().unwrap();
    assert!(
        (first - 1090.0).abs() < 250.0,
        "glide must start near /a/ F2 1090 Hz, got {first:.0}"
    );
    assert!(
        (last - 2290.0).abs() < 250.0,
        "glide must end near /i/ F2 2290 Hz, got {last:.0}"
    );
    assert!(last - first > 900.0, "total F2 rise must be ~1200 Hz");
    for w in steady.windows(2) {
        assert!(
            w[1] >= w[0] - 60.0,
            "F2 must move monotonically toward /i/ (frame noise epsilon 60 Hz): {:.0} -> {:.0}",
            w[0],
            w[1]
        );
    }
}
