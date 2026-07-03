//! Render the Phase-2 phoneme inventory samples for human ears.
//! Usage: cargo run -p voksa-engine-klattsch --example render_inventory

use voksa_core::phonemes::{Consonant, Phoneme, Vowel};
use voksa_engine_klattsch::{SAMPLE_RATE, render_phonemes, render_steady_phoneme};

fn main() {
    let dir = "artifacts/listening/phase2";

    for v in Vowel::ALL {
        let samples = render_steady_phoneme(Phoneme::Vowel(v), SAMPLE_RATE, 500);
        voksa_testkit::write_wav(
            format!("{dir}/vowel_{}.wav", format!("{v:?}").to_lowercase()),
            &samples,
            SAMPLE_RATE,
        );
    }
    for c in [
        Consonant::S,
        Consonant::Z,
        Consonant::C,
        Consonant::J,
        Consonant::X,
    ] {
        let samples = render_steady_phoneme(Phoneme::Consonant(c), SAMPLE_RATE, 400);
        voksa_testkit::write_wav(
            format!("{dir}/fricative_{}.wav", format!("{c:?}").to_lowercase()),
            &samples,
            SAMPLE_RATE,
        );
    }
    // "coi" (hello): c + oi diphthong — the first word voksa can say.
    let coi = render_phonemes(
        &[
            Phoneme::Consonant(Consonant::C),
            Phoneme::Diphthong(Vowel::O, Vowel::I),
        ],
        SAMPLE_RATE,
    );
    voksa_testkit::write_wav(format!("{dir}/coi.wav"), &coi, SAMPLE_RATE);
    // "ta" (that): stop + vowel.
    let ta = render_phonemes(
        &[Phoneme::Consonant(Consonant::T), Phoneme::Vowel(Vowel::A)],
        SAMPLE_RATE,
    );
    voksa_testkit::write_wav(format!("{dir}/ta.wav"), &ta, SAMPLE_RATE);

    println!("wrote inventory WAVs to {dir}/");
}
