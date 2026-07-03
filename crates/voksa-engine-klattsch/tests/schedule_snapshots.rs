//! Deterministic-lowering trail: snapshot the exact events the compiler emits
//! for representative phonemes. Any change to the table or the lowering
//! conventions must show up here and be consciously accepted.

use voksa_core::phonemes::{Consonant, Phoneme, Vowel};
use voksa_engine_klattsch::{SAMPLE_RATE, lower_sequence};

fn events_of(phonemes: &[Phoneme]) -> String {
    let (schedule, total_ms) = lower_sequence(phonemes, SAMPLE_RATE);
    format!("total_ms: {total_ms}\n{:#?}", schedule.events())
}

#[test]
fn snapshot_vowel_a() {
    insta::assert_snapshot!(events_of(&[Phoneme::Vowel(Vowel::A)]));
}

#[test]
fn snapshot_sibilant_s() {
    insta::assert_snapshot!(events_of(&[Phoneme::Consonant(Consonant::S)]));
}

#[test]
fn snapshot_stop_t() {
    insta::assert_snapshot!(events_of(&[Phoneme::Consonant(Consonant::T)]));
}

#[test]
fn snapshot_diphthong_ai() {
    insta::assert_snapshot!(events_of(&[Phoneme::Diphthong(Vowel::A, Vowel::I)]));
}
