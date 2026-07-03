//! Mandatory pause insertion (CLL §4.9 rules 1–7, §4.2 extras, §17.2).
//!
//! One merged Pause per word boundary (CLL §4.2 sanctions merging: "the pause
//! after cy. merges with the pause before .ibu"). The comma never pauses.
//! `--dotside` drops the la/lai/la'i/doi exemption so every cmevla gets a
//! leading pause.

use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use crate::word::WordAnalysis;

/// Input token: an analyzed Lojban word, or pre-marked non-Lojban text
/// (zoi/la'o payload — the tokenizer marks these in later phases).
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(WordAnalysis),
    Foreign(String),
}

/// Output stream: words/foreign chunks with mandatory pauses inserted.
#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    Word(WordAnalysis),
    Foreign(String),
    Pause,
}

/// Insert every mandatory pause into a word sequence.
pub fn insert_pauses(tokens: Vec<Token>, dotside: bool) -> Vec<Segment> {
    let _ = (tokens, dotside);
    todo!("Phase 4 red checkpoint: pause rules land after the failing tests are committed")
}
