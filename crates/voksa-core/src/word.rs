//! Capitals-aware word analysis: the Phase-4 entry point tying together
//! letters → syllables → class → stress.
//!
//! Capital letters mark non-default stress (CLL §3.1: "it is sufficient to
//! capitalize the vowel letter ... but it is easier on the reader to
//! capitalize the whole syllable") — so any uppercase character marks its
//! CONTAINING syllable as the explicit stress target.

use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use crate::classify::WordClass;
use crate::letters::WordError;
use crate::syllable::Syllable;

#[derive(Debug, Clone, PartialEq)]
pub struct WordAnalysis {
    /// Lowercased text (commas preserved).
    pub lowered: String,
    pub class: WordClass,
    pub syllables: Vec<Syllable>,
    /// Resolved primary-stress syllable index (None = unstressed word).
    pub stress: Option<usize>,
}

/// Analyze one raw word: lowercase + capital-stress extraction, syllabify,
/// classify, resolve stress.
pub fn analyze_word(raw: &str) -> Result<WordAnalysis, WordError> {
    let _ = raw;
    todo!("Phase 4 red checkpoint: analysis lands after the failing tests are committed")
}
