//! Word classification (CLL §4.2/§4.3/§4.8): cmevla / brivla / cmavo.
//!
//! Minimal classifier, sufficient for stress + pause rules: ends in a
//! consonant → cmevla; ends in y → cmavo (no brivla may end in y, CLL
//! §4.1/§4.7); else a consonant PAIR — permissibility NOT required (CLL §4.3's
//! own example is the impermissible "sc" in bisycla) — within the first five
//! letters counted after deleting y and apostrophe → brivla; else cmavo.

use crate::alloc::vec::Vec;
use crate::letters::{Letter, WordError, parse_word};
use crate::phonemes::Vowel;

/// The three Lojban word shapes (CLL §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordClass {
    /// Name word: ends in a consonant; always pause-delimited.
    Cmevla,
    /// Predicate word: a consonant pair in the first five letters + ends in a
    /// vowel other than y.
    Brivla,
    /// Structure word: everything else.
    Cmavo,
}

/// Classify one word (lowercase letters + apostrophe + comma).
pub fn classify(word: &str) -> Result<WordClass, WordError> {
    let letters = parse_word(word)?;
    let core: Vec<Letter> = letters
        .into_iter()
        .filter(|l| *l != Letter::Comma)
        .collect();
    match core
        .last()
        .expect("parse_word rejects empty/trailing-comma input")
    {
        Letter::C(_) => return Ok(WordClass::Cmevla),
        Letter::V(Vowel::Y) => return Ok(WordClass::Cmavo),
        Letter::V(_) => {}
        Letter::Apostrophe | Letter::Comma => {
            unreachable!("parse_word forbids trailing apostrophe/comma")
        }
    }
    // First five letters counted after deleting y and apostrophe (CLL §4.3:
    // bisycla's "syc" counts as the pair "sc"; ro'inre'o counts "nr").
    let counted: Vec<Letter> = core
        .iter()
        .copied()
        .filter(|l| !matches!(l, Letter::Apostrophe | Letter::V(Vowel::Y)))
        .take(5)
        .collect();
    let has_pair = counted
        .windows(2)
        .any(|w| matches!((w[0], w[1]), (Letter::C(_), Letter::C(_))));
    Ok(if has_pair {
        WordClass::Brivla
    } else {
        WordClass::Cmavo
    })
}
