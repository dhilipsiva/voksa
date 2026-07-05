//! Capitals-aware word analysis: the Phase-4 entry point tying together
//! letters → syllables → class → stress.
//!
//! Capital letters mark non-default stress (CLL §3.1: "it is sufficient to
//! capitalize the vowel letter ... but it is easier on the reader to
//! capitalize the whole syllable") — so any uppercase character marks its
//! CONTAINING syllable as the explicit stress target.

use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use crate::classify::{WordClass, classify};
use crate::letters::WordError;
use crate::stress::resolve_stress;
use crate::syllable::{Nucleus, Syllable, syllabify};

/// A fully analyzed word: letters → syllables → class → stress.
#[derive(Debug, Clone, PartialEq)]
pub struct WordAnalysis {
    /// Lowercased text (commas preserved).
    pub lowered: String,
    /// cmevla / brivla / cmavo (CLL §4.2).
    pub class: WordClass,
    /// The word's syllables (CLL §3.9).
    pub syllables: Vec<Syllable>,
    /// Resolved primary-stress syllable index (None = unstressed word).
    pub stress: Option<usize>,
}

/// Analyze one raw word: lowercase + capital-stress extraction, syllabify,
/// classify, resolve stress.
pub fn analyze_word(raw: &str) -> Result<WordAnalysis, WordError> {
    let mut lowered = String::with_capacity(raw.len());
    let mut capital_positions: Vec<usize> = Vec::new();
    for (pos, ch) in raw.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            capital_positions.push(pos);
            lowered.push(ch.to_ascii_lowercase());
        } else {
            lowered.push(ch);
        }
    }
    let syllables = syllabify(&lowered)?;
    let class = classify(&lowered)?;

    // Char position → owning syllable. Round-trip guarantees the concatenated
    // syllable letters equal the input minus commas, so a lockstep walk works:
    // commas own nothing; every other char consumes one slot of the current
    // syllable's span (apostrophe + onset + nucleus + coda).
    let spans: Vec<usize> = syllables.iter().map(syllable_char_len).collect();
    let mut owner: Vec<Option<usize>> = Vec::with_capacity(lowered.len());
    let mut sidx = 0usize;
    let mut used = 0usize;
    for ch in lowered.chars() {
        if ch == ',' {
            owner.push(None);
            continue;
        }
        while sidx < spans.len() && used == spans[sidx] {
            sidx += 1;
            used = 0;
        }
        owner.push(Some(sidx));
        used += 1;
    }

    let mut explicit: Option<usize> = None;
    for pos in capital_positions {
        match owner.get(pos).copied().flatten() {
            Some(s) => match explicit {
                None => explicit = Some(s),
                Some(prev) if prev == s => {}
                Some(_) => return Err(WordError::InvalidStressMark),
            },
            None => return Err(WordError::InvalidStressMark),
        }
    }

    let stress = resolve_stress(&syllables, class, explicit)?;
    Ok(WordAnalysis {
        lowered,
        class,
        syllables,
        stress,
    })
}

/// Number of characters a syllable occupies in the (comma-stripped) text.
fn syllable_char_len(s: &Syllable) -> usize {
    let nucleus_len = match s.nucleus {
        Nucleus::Diphthong(..) => 2,
        _ => 1,
    };
    usize::from(s.aspirated) + s.onset.len() + nucleus_len + s.coda.len()
}
