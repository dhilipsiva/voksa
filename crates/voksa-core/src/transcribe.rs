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
use crate::alloc::vec::Vec;
use crate::compiler::{CompileError, CompileOptions, UtteranceItem, utterance_items};
use crate::letters::{consonant_to_char, vowel_to_char};
use crate::syllable::Nucleus;
use crate::word::WordAnalysis;

/// Transcribe `text` under `opts` (dotside changes pauses; buffer inserts
/// `(ɪ)`). Deterministic, and derived from the SAME item pipeline `compile`
/// consumes — the display cannot disagree with the audio.
pub fn transcribe(text: &str, opts: &CompileOptions) -> Result<String, CompileError> {
    let items = utterance_items(text, opts)?;
    let parts: Vec<String> = items
        .iter()
        .map(|item| match item {
            UtteranceItem::Pause => String::from("‖"),
            UtteranceItem::Word(w) => render_word(w, opts.buffer),
        })
        .collect();
    Ok(parts.join(" "))
}

/// Render one analyzed word: syllables joined by `.`, the stressed syllable
/// UPPERCASE, `'` for [h], and (when buffering) `(ɪ)` between adjacent
/// cluster consonants — the SAME adjacency the compiler buffers: onset/coda
/// consonants only, [h] and nuclei (incl. syllabic sonorants) break the pair.
fn render_word(w: &WordAnalysis, buffer: bool) -> String {
    let mut out = String::new();
    // True when the previously emitted phoneme was a cluster consonant.
    let mut prev_consonant = false;
    // Deferred syllable dot: a buffer marker between syllables lands BEFORE
    // the dot ("MUN(ɪ).je"), so the dot flushes with the next phoneme.
    let mut pending_dot = false;
    let emit = |out: &mut String, prev: &mut bool, dot: &mut bool, ch: char, is_consonant: bool| {
        if buffer && *prev && is_consonant {
            out.push_str("(ɪ)");
        }
        if *dot {
            out.push('.');
            *dot = false;
        }
        out.push(ch);
        *prev = is_consonant;
    };
    for (si, syl) in w.syllables.iter().enumerate() {
        if si > 0 {
            pending_dot = true;
        }
        let stressed = w.stress == Some(si);
        let case = |c: char| if stressed { c.to_ascii_uppercase() } else { c };
        if syl.aspirated {
            emit(&mut out, &mut prev_consonant, &mut pending_dot, '\'', false);
        }
        for c in &syl.onset {
            let ch = case(consonant_to_char(*c));
            emit(&mut out, &mut prev_consonant, &mut pending_dot, ch, true);
        }
        match syl.nucleus {
            Nucleus::Vowel(v) => {
                let ch = case(vowel_to_char(v));
                emit(&mut out, &mut prev_consonant, &mut pending_dot, ch, false);
            }
            Nucleus::Diphthong(a, b) => {
                let (ca, cb) = (case(vowel_to_char(a)), case(vowel_to_char(b)));
                emit(&mut out, &mut prev_consonant, &mut pending_dot, ca, false);
                emit(&mut out, &mut prev_consonant, &mut pending_dot, cb, false);
            }
            Nucleus::Syllabic(c) => {
                let ch = case(consonant_to_char(c));
                emit(&mut out, &mut prev_consonant, &mut pending_dot, ch, false);
            }
        }
        for c in &syl.coda {
            let ch = case(consonant_to_char(*c));
            emit(&mut out, &mut prev_consonant, &mut pending_dot, ch, true);
        }
    }
    out
}
