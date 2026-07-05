//! Mandatory pause insertion (CLL §4.9 rules 1–7, §4.2 extras, §17.2).
//!
//! One merged Pause per word boundary (CLL §4.2 sanctions merging: "the pause
//! after cy. merges with the pause before .ibu"). The comma never pauses.
//! `--dotside` drops the la/lai/la'i/doi exemption so every cmevla gets a
//! leading pause.

use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use crate::classify::WordClass;
use crate::letters::consonant_from_char;
use crate::word::WordAnalysis;

/// Input token: an analyzed Lojban word, or pre-marked non-Lojban text
/// (zoi/la'o payload — the tokenizer marks these in later phases).
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// An analyzed Lojban word.
    Word(WordAnalysis),
    /// Pre-marked non-Lojban text (zoi/la'o payload).
    Foreign(String),
}

/// Output stream: words/foreign chunks with mandatory pauses inserted.
#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    /// An analyzed Lojban word.
    Word(WordAnalysis),
    /// Non-Lojban text, always pause-bracketed.
    Foreign(String),
    /// One merged mandatory pause (boundaries never stack two).
    Pause,
}

/// Insert every mandatory pause into a word sequence.
pub fn insert_pauses(tokens: Vec<Token>, dotside: bool) -> Vec<Segment> {
    let profiles: Vec<Profile> = tokens.iter().map(Profile::of).collect();
    let mut out = Vec::with_capacity(tokens.len() * 2);
    for (i, token) in tokens.into_iter().enumerate() {
        let needs_pause = if i == 0 {
            // Utterance start: vowel-initial words open with a glottal onset
            // (§3.3); cmevla are "preceded by a pause" with nothing to exempt
            // them; foreign text and hesitation-y are pause-bracketed.
            profiles[0].vowel_initial
                || profiles[0].cmevla
                || profiles[0].foreign
                || profiles[0].hesitation
        } else {
            boundary_needs_pause(&profiles[i - 1], &profiles[i], dotside)
        };
        if needs_pause {
            out.push(Segment::Pause);
        }
        out.push(match token {
            Token::Word(w) => Segment::Word(w),
            Token::Foreign(t) => Segment::Foreign(t),
        });
    }
    if let Some(last) = profiles.last() {
        // r2 (consonant-final), hesitation-y trailing pause, foreign bracket.
        if last.consonant_final || last.hesitation || last.foreign {
            out.push(Segment::Pause);
        }
    }
    out
}

/// Pause-relevant summary of one token (CLL §4.9 vocabulary).
struct Profile {
    foreign: bool,
    vowel_initial: bool,
    consonant_final: bool,
    y_final_cmavo: bool,
    hesitation: bool,
    final_stressed: bool,
    first_stressed: bool,
    la_family: bool,
    cmevla: bool,
    brivla: bool,
}

impl Profile {
    fn of(token: &Token) -> Profile {
        let Token::Word(w) = token else {
            return Profile {
                foreign: true,
                vowel_initial: false,
                consonant_final: false,
                y_final_cmavo: false,
                hesitation: false,
                final_stressed: false,
                first_stressed: false,
                la_family: false,
                cmevla: false,
                brivla: false,
            };
        };
        let first = w.lowered.chars().next();
        let last = w.lowered.chars().next_back();
        Profile {
            foreign: false,
            vowel_initial: matches!(first, Some('a' | 'e' | 'i' | 'o' | 'u' | 'y')),
            consonant_final: last.is_some_and(|c| consonant_from_char(c).is_some()),
            y_final_cmavo: w.class == WordClass::Cmavo && last == Some('y'),
            hesitation: w.lowered == "y",
            final_stressed: !w.syllables.is_empty() && w.stress == Some(w.syllables.len() - 1),
            first_stressed: w.stress == Some(0),
            la_family: matches!(w.lowered.as_str(), "la" | "lai" | "la'i" | "doi"),
            cmevla: w.class == WordClass::Cmevla,
            brivla: w.class == WordClass::Brivla,
        }
    }
}

/// CLL §4.9 rules 2–7 (+§4.2 extras) folded over one word boundary; any rule
/// demanding a pause here is satisfied by the single merged Pause.
fn boundary_needs_pause(prev: &Profile, next: &Profile, dotside: bool) -> bool {
    prev.foreign                                     // r7: after foreign text
        || next.foreign                              // r7: before foreign text
        || prev.consonant_final                      // r2: after cmevla
        || next.vowel_initial                        // r3: before vowel-initial
        || (next.cmevla && (dotside || !prev.la_family)) // r4 (+dotside)
        || (prev.final_stressed && next.brivla)      // r5: stress collision
        || (prev.final_stressed && next.first_stressed) // §4.2 stressed+stressed
        || (prev.y_final_cmavo && !next.y_final_cmavo) // r6 generalized (§17.2)
        || prev.hesitation                           // §4.2: pause after .y.
        || next.hesitation // §4.2: pause before .y. (also r3)
}
