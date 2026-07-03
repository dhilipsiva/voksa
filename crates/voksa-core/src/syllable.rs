//! CLL §3.9 syllabification.
//!
//! Deterministic algorithm choosing CLL's "normal" readings (documented in
//! docs/phonology.md §3): nuclei are vowels/diphthongs munched left-to-right
//! (CLL §3.5), each inter-nucleus consonant run gives its maximal legal-onset
//! suffix to the following syllable (coinciding with CLL's C.C / .CC / C.CC
//! rules for all standard words), and sonorants become syllabic nuclei ONLY
//! in vowel-less regions (CLL §3.4 leaves syllabicity to the speaker; this
//! choice is stress-invariant per §3.9).

use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use crate::letters::{WordError, consonant_to_char, parse_word, vowel_to_char};
use crate::phonemes::{Consonant, Vowel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nucleus {
    Vowel(Vowel),
    Diphthong(Vowel, Vowel),
    /// A syllabic sonorant (l m n r) — never stressed, never counted (CLL §3.9).
    Syllabic(Consonant),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syllable {
    pub onset: Vec<Consonant>,
    /// True when the syllable is introduced by an apostrophe (= [h]).
    pub aspirated: bool,
    pub nucleus: Nucleus,
    pub coda: Vec<Consonant>,
}

/// Syllabify one word (lowercase letters + apostrophe + comma).
pub fn syllabify(word: &str) -> Result<Vec<Syllable>, WordError> {
    let letters = parse_word(word)?;
    let _ = letters;
    todo!("Phase 3 red checkpoint: syllabifier lands after the failing tests are committed")
}

/// Render syllables back to text (apostrophes preserved, commas dropped) —
/// the round-trip counterpart of [`syllabify`].
pub fn to_text(syllables: &[Syllable]) -> String {
    let mut out = String::new();
    for syl in syllables {
        if syl.aspirated {
            out.push('\'');
        }
        for c in &syl.onset {
            out.push(consonant_to_char(*c));
        }
        match syl.nucleus {
            Nucleus::Vowel(v) => out.push(vowel_to_char(v)),
            Nucleus::Diphthong(a, b) => {
                out.push(vowel_to_char(a));
                out.push(vowel_to_char(b));
            }
            Nucleus::Syllabic(c) => out.push(consonant_to_char(c)),
        }
        for c in &syl.coda {
            out.push(consonant_to_char(*c));
        }
    }
    out
}

