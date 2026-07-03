//! Property tests (playbook §c): totality, round-trip, structural invariants,
//! classifier consistency.

use proptest::prelude::*;
use voksa_core::classify::{WordClass, classify};
use voksa_core::clusters::{INITIAL_PAIRS, is_legal_onset, is_permissible_pair};
use voksa_core::letters::{consonant_to_char, vowel_to_char};
use voksa_core::phonemes::{Consonant, Vowel};
use voksa_core::syllable::{syllabify, to_text};

static PLAIN_VOWELS: [Vowel; 5] = [Vowel::A, Vowel::E, Vowel::I, Vowel::O, Vowel::U];
static ALPHABET: [char; 25] = [
    'a', 'e', 'i', 'o', 'u', 'y', 'b', 'c', 'd', 'f', 'g', 'j', 'k', 'l', 'm', 'n', 'p', 'r', 's',
    't', 'v', 'x', 'z', '\'', ',',
];

fn vowel() -> impl Strategy<Value = Vowel> {
    prop::sample::select(&PLAIN_VOWELS[..])
}

fn consonant() -> impl Strategy<Value = Consonant> {
    prop::sample::select(&Consonant::ALL[..])
}

/// A legal gismu: CVC/CV (medial pair permissible) or CCVCV (initial pair).
fn gismu() -> impl Strategy<Value = String> {
    let cvccv = (
        consonant(),
        vowel(),
        (consonant(), consonant()).prop_filter("permissible medial pair", |(a, b)| {
            is_permissible_pair(*a, *b)
        }),
        vowel(),
    )
        .prop_map(|(c1, v1, (c2, c3), v2)| {
            let mut s = String::new();
            s.push(consonant_to_char(c1));
            s.push(vowel_to_char(v1));
            s.push(consonant_to_char(c2));
            s.push(consonant_to_char(c3));
            s.push(vowel_to_char(v2));
            s
        });
    let ccvcv = (
        prop::sample::select(&INITIAL_PAIRS[..]),
        vowel(),
        consonant(),
        vowel(),
    )
        .prop_map(|((c1, c2), v1, c3, v2)| {
            let mut s = String::new();
            s.push(consonant_to_char(c1));
            s.push(consonant_to_char(c2));
            s.push(vowel_to_char(v1));
            s.push(consonant_to_char(c3));
            s.push(vowel_to_char(v2));
            s
        });
    prop_oneof![cvccv, ccvcv]
}

fn alphabet_soup() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(&ALPHABET[..]), 1..12)
        .prop_map(|chars| chars.into_iter().collect())
}

fn input_minus_commas(input: &str) -> String {
    input.chars().filter(|c| *c != ',').collect()
}

proptest! {
    /// Totality: any string over the alphabet either syllabifies or returns a
    /// typed error — never a panic. When it parses, it round-trips and every
    /// onset is legal.
    #[test]
    fn soup_is_total_and_roundtrips(word in alphabet_soup()) {
        if let Ok(syllables) = syllabify(&word) {
            prop_assert_eq!(to_text(&syllables), input_minus_commas(&word));
            for syl in &syllables {
                prop_assert!(is_legal_onset(&syl.onset), "illegal onset in {}", word);
            }
        }
        let _ = classify(&word); // total as well (Ok or Err, no panic)
    }

    /// Every legal gismu syllabifies, round-trips, and classifies as brivla.
    #[test]
    fn gismu_syllabify_and_classify(word in gismu()) {
        let syllables = syllabify(&word).expect("gismu must syllabify");
        prop_assert_eq!(to_text(&syllables), word.clone());
        prop_assert_eq!(syllables.len(), 2, "gismu are two syllables: {}", word);
        for syl in &syllables {
            prop_assert!(is_legal_onset(&syl.onset));
        }
        prop_assert_eq!(classify(&word).unwrap(), WordClass::Brivla);
    }

    /// Anything valid that ends in a consonant classifies as cmevla.
    #[test]
    fn consonant_final_is_cmevla(word in gismu(), c in consonant()) {
        let mut name = word;
        name.push(consonant_to_char(c));
        prop_assert_eq!(classify(&name).unwrap(), WordClass::Cmevla);
    }

    /// Lujvo-ish gismu+y+gismu compounds end in a vowel with an early pair.
    #[test]
    fn y_glued_compounds_are_brivla(a in gismu(), b in gismu()) {
        let mut lujvo = a;
        lujvo.push('y');
        lujvo.push_str(&b);
        prop_assert_eq!(classify(&lujvo).unwrap(), WordClass::Brivla);
    }

    /// Cy forms end in y and are cmavo, never brivla.
    #[test]
    fn cy_forms_are_cmavo(c in consonant()) {
        let mut w = String::new();
        w.push(consonant_to_char(c));
        w.push('y');
        prop_assert_eq!(classify(&w).unwrap(), WordClass::Cmavo);
    }
}
