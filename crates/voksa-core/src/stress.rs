//! Stress assignment (CLL §3.9, §4.8): penultimate over COUNTABLE syllables.
//!
//! Countable excludes syllables whose vowel is y — including the iy/uy
//! diphthongs, whose vowel CLL §3.4 itself defines as [ə] and which §3.9
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
    let _ = (syllables, class);
    todo!("Phase 4 red checkpoint: stress lands after the failing tests are committed")
}

/// Resolve stress given an optional explicit (capital-marked) syllable.
/// Explicit marks on uncountable syllables are errors; brivla may only be
/// marked on their computed penultimate (CLL §4.3 property 3).
pub fn resolve_stress(
    syllables: &[Syllable],
    class: WordClass,
    explicit: Option<usize>,
) -> Result<Option<usize>, WordError> {
    let _ = (syllables, class, explicit);
    todo!("Phase 4 red checkpoint")
}
