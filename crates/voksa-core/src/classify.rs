//! Word classification (CLL §4.2/§4.3/§4.8): cmevla / brivla / cmavo.
//!
//! Minimal classifier, sufficient for stress + pause rules: ends in a
//! consonant → cmevla; ends in y → cmavo (no brivla may end in y, CLL
//! §4.1/§4.7); else a consonant PAIR — permissibility NOT required (CLL §4.3's
//! own example is the impermissible "sc" in bisycla) — within the first five
//! letters counted after deleting y and apostrophe → brivla; else cmavo.

use crate::letters::WordError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordClass {
    Cmevla,
    Brivla,
    Cmavo,
}

/// Classify one word (lowercase letters + apostrophe + comma).
pub fn classify(word: &str) -> Result<WordClass, WordError> {
    let _ = word;
    todo!("Phase 3 red checkpoint: classifier lands after the failing tests are committed")
}
