//! The schedule compiler: Lojban text → deterministic [`UtteranceSchedule`].
//!
//! Pipeline: tokenize (whitespace + periods; every written period is honored
//! as a pause — CLL §3.3 makes any inter-word pause legal) → analyze words
//! (Phase 4) → mandatory pause insertion (+ writer-marked pauses) → syllable/
//! phoneme expansion with optional buffering (CLL §3.8 "fully-buffered
//! dialect": a weak [ɪ] between every word-internal consonant pair) → timed
//! events + syllable spans.

use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use crate::letters::WordError;
use crate::schedule::UtteranceSchedule;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompileOptions {
    /// Force a leading pause before every cmevla (drop the la-family
    /// exemption).
    pub dotside: bool,
    /// Fully-buffered dialect: insert a weak [ɪ] between every word-internal
    /// consonant pair.
    pub buffer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// A word failed morphological analysis.
    Word { word: String, error: WordError },
    /// Digits await Phase-6 normalization (numbers → PA cmavo).
    DigitsUnsupported(String),
    /// No words in the input.
    Empty,
}

/// A raw token: a word (capitals preserved — they mark stress) or a
/// writer-marked pause (period).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawToken {
    Word(String),
    ExplicitPause,
}

/// Split text into words and explicit pause marks. Whitespace and periods
/// delimit words; consecutive pause marks merge; commas and apostrophes stay
/// inside their word.
pub fn tokenize(text: &str) -> Result<Vec<RawToken>, CompileError> {
    let _ = text;
    todo!("Phase 5 red checkpoint: tokenizer lands after the failing tests are committed")
}

/// Compile an utterance to its deterministic parameter schedule.
pub fn compile(text: &str, opts: &CompileOptions) -> Result<UtteranceSchedule, CompileError> {
    let _ = (text, opts);
    todo!("Phase 5 red checkpoint: compiler lands after the failing tests are committed")
}
