//! Stress assignment (CLL §3.9, §4.8): penultimate over COUNTABLE syllables.
//!
//! Countable excludes syllables whose vowel is y — including the iy/uy
//! diphthongs, whose vowel CLL §3.4 itself defines as `[ə]` and which §3.9
//! requires to carry weak stress — and syllabic-consonant syllables. (Buffer
//! vowel syllables join the exclusion list with the Phase-5 --buffer flag.)

use crate::classify::WordClass;
use crate::letters::WordError;
use crate::phonemes::Vowel;
use crate::syllable::{Nucleus, Syllable};

/// Does this syllable count for stress placement?
pub fn is_countable(syl: &Syllable) -> bool {
    !matches!(
        syl.nucleus,
        Nucleus::Vowel(Vowel::Y) | Nucleus::Diphthong(_, Vowel::Y) | Nucleus::Syllabic(_)
    )
}

/// Default stress: brivla and cmevla take the penultimate countable syllable
/// (a single countable syllable takes it; none countable → unstressed);
/// cmavo are unstressed by default (CLL §4.2).
pub fn default_stress(syllables: &[Syllable], class: WordClass) -> Option<usize> {
    if class == WordClass::Cmavo {
        return None;
    }
    let countable: crate::alloc::vec::Vec<usize> = syllables
        .iter()
        .enumerate()
        .filter(|(_, s)| is_countable(s))
        .map(|(i, _)| i)
        .collect();
    match countable.len() {
        0 => None,
        1 => Some(countable[0]),
        n => Some(countable[n - 2]),
    }
}

/// Resolve stress given an optional explicit (capital-marked) syllable.
/// Explicit marks on uncountable syllables are errors; brivla may only be
/// marked on their computed penultimate (CLL §4.3 property 3).
pub fn resolve_stress(
    syllables: &[Syllable],
    class: WordClass,
    explicit: Option<usize>,
) -> Result<Option<usize>, WordError> {
    let Some(idx) = explicit else {
        return Ok(default_stress(syllables, class));
    };
    if idx >= syllables.len() || !is_countable(&syllables[idx]) {
        return Err(WordError::InvalidStressMark);
    }
    if class == WordClass::Brivla && Some(idx) != default_stress(syllables, class) {
        return Err(WordError::InvalidStressMark);
    }
    Ok(Some(idx))
}
