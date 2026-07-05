//! Phonetic transcription: render the EXACT syllable/stress/pause analysis the
//! compiler speaks, as a human-checkable string (demo D2c — the community
//! validates voksa's phonetics by comparing what they hear to this line).
//!
//! Notation (CLL-flavored): syllables joined by `.` within a word; the
//! stressed syllable UPPERCASE (CLL's capitals-for-stress convention); pauses
//! as ` ‖ ` (mandatory + writer periods, merged per CLL §4.2); apostrophe [h]
//! kept; input commas dropped (syllable dots replace them); epenthetic buffer
//! vowels as `(ɪ)` between the consonants they split; digits/lerfu appear as
//! their normalized cmavo. Examples: `coi MUN.je`, `la DJAN ‖ cu KLA.ma`,
//! `V(ɪ)RU.si`, `li ci pi pa vo`.

use crate::alloc::string::String;
use crate::compiler::{CompileError, CompileOptions};

/// Transcribe `text` under `opts` (dotside changes pauses; buffer inserts
/// `(ɪ)`). Deterministic, and derived from the SAME item pipeline `compile`
/// consumes — the display cannot disagree with the audio.
pub fn transcribe(text: &str, opts: &CompileOptions) -> Result<String, CompileError> {
    // RED stub (D2c): rendering lands with the failing tests.
    let _ = (text, opts);
    Ok(String::new())
}
