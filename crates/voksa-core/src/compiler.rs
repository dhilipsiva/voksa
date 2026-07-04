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
use crate::normalize::{NumberError, number_words};
use crate::pause::{Segment, Token, insert_pauses};
use crate::phonemes::{Phoneme, SegmentSpec, buffer_spec, spec};
use crate::schedule::{
    BASE_F0_HZ, Event, Frame, PAUSE_MS, SyllableSpan, UtteranceSchedule, schedule_segment,
    silence_targets,
};
use crate::stress::is_countable;
use crate::syllable::Nucleus;
use crate::word::{WordAnalysis, analyze_word};

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
    /// A written figure could not be normalized to PA cmavo.
    MalformedNumber { figure: String, error: NumberError },
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
    fn flush(current: &mut String, tokens: &mut Vec<RawToken>) -> Result<(), CompileError> {
        if !current.is_empty() {
            let token = core::mem::take(current);
            if token.chars().any(|c| c.is_ascii_digit()) {
                // A written figure: expand to PA cmavo words (Phase 6).
                match number_words(&token) {
                    Ok(words) => {
                        tokens.extend(words.into_iter().map(|w| RawToken::Word(String::from(w))))
                    }
                    Err(error) => {
                        return Err(CompileError::MalformedNumber {
                            figure: token,
                            error,
                        });
                    }
                }
            } else {
                tokens.push(RawToken::Word(token));
            }
        }
        Ok(())
    }
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut current = String::new();
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_whitespace() {
            flush(&mut current, &mut tokens)?;
        } else if ch == '.' {
            // A period BETWEEN digits is a decimal point and stays inside the
            // figure token (3.14 → ci pi pa vo); anywhere else it delimits
            // words and marks a pause.
            let decimal = current
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_digit())
                && chars.get(i + 1).is_some_and(|c| c.is_ascii_digit());
            if decimal {
                current.push('.');
            } else {
                flush(&mut current, &mut tokens)?;
                if tokens.last() != Some(&RawToken::ExplicitPause) {
                    tokens.push(RawToken::ExplicitPause);
                }
            }
        } else {
            current.push(ch);
        }
    }
    flush(&mut current, &mut tokens)?;
    Ok(tokens)
}

enum Item {
    Word(WordAnalysis),
    Pause,
}

/// Compile an utterance to its deterministic parameter schedule.
pub fn compile(text: &str, opts: &CompileOptions) -> Result<UtteranceSchedule, CompileError> {
    // Tokenize, remembering which boundaries the writer marked with periods.
    let mut words: Vec<WordAnalysis> = Vec::new();
    let mut explicit: Vec<bool> = Vec::new();
    explicit.push(false); // before word 0
    for token in tokenize(text)? {
        match token {
            RawToken::Word(w) => {
                let analysis = analyze_word(&w).map_err(|error| CompileError::Word {
                    word: w.clone(),
                    error,
                })?;
                words.push(analysis);
                explicit.push(false);
            }
            RawToken::ExplicitPause => {
                if let Some(flag) = explicit.last_mut() {
                    *flag = true;
                }
            }
        }
    }
    if words.is_empty() {
        return Err(CompileError::Empty);
    }

    // Mandatory pauses (Phase 4), then union in the writer-marked ones.
    let segments = insert_pauses(words.into_iter().map(Token::Word).collect(), opts.dotside);
    let mut items: Vec<Item> = Vec::new();
    let mut wi = 0usize;
    for seg in segments {
        match seg {
            Segment::Pause => items.push(Item::Pause),
            Segment::Word(w) => {
                if explicit[wi] && !matches!(items.last(), Some(Item::Pause)) {
                    items.push(Item::Pause);
                }
                items.push(Item::Word(w));
                wi += 1;
            }
            // The tokenizer never produces foreign text (zoi parsing arrives
            // with normalization); pause-bracket defensively.
            Segment::Foreign(_) => items.push(Item::Pause),
        }
    }
    if explicit[wi] && !matches!(items.last(), Some(Item::Pause)) {
        items.push(Item::Pause);
    }

    // Fold into events + spans.
    let mut events: Vec<Event> = Vec::new();
    let mut spans: Vec<SyllableSpan> = Vec::new();
    let mut t_ms = 0.0f32;
    let mut word_index = 0usize;
    for item in items {
        match item {
            Item::Pause => {
                events.push(Event {
                    at_ms: t_ms,
                    transition_ms: 5.0,
                    frame: Frame::modal(BASE_F0_HZ, silence_targets()),
                });
                t_ms += PAUSE_MS;
            }
            Item::Word(w) => {
                t_ms = schedule_word(&w, word_index, opts.buffer, t_ms, &mut events, &mut spans);
                word_index += 1;
            }
        }
    }
    Ok(UtteranceSchedule {
        events,
        spans,
        total_ms: t_ms,
        // RED stub: attitudinal detection is not wired yet (Phase 10 P10-2 green).
        attitudinals: Vec::new(),
    })
}

#[derive(Clone, Copy)]
struct Entry {
    seg: SegmentSpec,
    span: usize,
    /// True only for onset/coda consonants — the buffer flag inserts between
    /// adjacent pairs of these (CLL §3.8 buffers consonant clusters; syllabic
    /// nuclei and [h] are not cluster members).
    is_consonant: bool,
    /// The syllable nucleus (vowel / diphthong / syllabic consonant, or a
    /// buffer span's single entry). The stress stretch anchors here so onset
    /// consonants stay at unit rate (CP1 fix).
    is_nucleus: bool,
}

/// Expand one analyzed word into timed events and syllable spans.
fn schedule_word(
    w: &WordAnalysis,
    word_index: usize,
    buffer: bool,
    start_ms: f32,
    events: &mut Vec<Event>,
    spans_out: &mut Vec<SyllableSpan>,
) -> f32 {
    // (stressed, countable) per span; buffer spans appended as created.
    let mut metas: Vec<(bool, bool)> = Vec::new();
    let mut entries: Vec<Entry> = Vec::new();
    for (si, syl) in w.syllables.iter().enumerate() {
        let span = metas.len();
        metas.push((w.stress == Some(si), is_countable(syl)));
        let push =
            |entries: &mut Vec<Entry>, seg: SegmentSpec, is_consonant: bool, is_nucleus: bool| {
                entries.push(Entry {
                    seg,
                    span,
                    is_consonant,
                    is_nucleus,
                });
            };
        if syl.aspirated {
            push(&mut entries, spec(Phoneme::H), false, false);
        }
        for c in &syl.onset {
            push(&mut entries, spec(Phoneme::Consonant(*c)), true, false);
        }
        match syl.nucleus {
            Nucleus::Vowel(v) => push(&mut entries, spec(Phoneme::Vowel(v)), false, true),
            Nucleus::Diphthong(a, b) => {
                push(&mut entries, spec(Phoneme::Diphthong(a, b)), false, true)
            }
            // Syllabic sonorant: the consonant's steady targets serve as the
            // nucleus (vocalic envelope refinement deferred to CP1 listening).
            Nucleus::Syllabic(c) => push(&mut entries, spec(Phoneme::Consonant(c)), false, true),
        }
        for c in &syl.coda {
            push(&mut entries, spec(Phoneme::Consonant(*c)), true, false);
        }
    }

    if buffer {
        let mut buffered: Vec<Entry> = Vec::with_capacity(entries.len() * 2);
        for (i, e) in entries.iter().enumerate() {
            if i > 0 && entries[i - 1].is_consonant && e.is_consonant {
                let span = metas.len();
                metas.push((false, false));
                buffered.push(Entry {
                    seg: buffer_spec(),
                    span,
                    is_consonant: false,
                    // A buffer syllable is its own nucleus (offset 0).
                    is_nucleus: true,
                });
            }
            buffered.push(*e);
        }
        entries = buffered;
    }

    // Timing fold ([h] lookahead within the word — apostrophes are
    // intervocalic, so the following entry is always the shaping nucleus).
    let mut bounds: Vec<Option<(f32, f32)>> = Vec::new();
    bounds.resize(metas.len(), None);
    let mut nucleus_at: Vec<Option<f32>> = Vec::new();
    nucleus_at.resize(metas.len(), None);
    let mut t_ms = start_ms;
    for i in 0..entries.len() {
        let next = entries.get(i + 1).and_then(|e| e.seg.leading_targets());
        let seg_start = t_ms;
        t_ms = schedule_segment(&entries[i].seg, next, BASE_F0_HZ, t_ms, events);
        match &mut bounds[entries[i].span] {
            slot @ None => *slot = Some((seg_start, t_ms)),
            Some((_, end)) => *end = t_ms,
        }
        if entries[i].is_nucleus && nucleus_at[entries[i].span].is_none() {
            nucleus_at[entries[i].span] = Some(seg_start);
        }
    }

    let mut word_spans: Vec<SyllableSpan> = bounds
        .iter()
        .zip(&metas)
        .zip(&nucleus_at)
        .filter_map(|((b, (stressed, countable)), nucleus)| {
            b.map(|(start_ms, end_ms)| SyllableSpan {
                start_ms,
                dur_ms: end_ms - start_ms,
                nucleus_off_ms: nucleus.map_or(0.0, |n| n - start_ms),
                word_index,
                stressed: *stressed,
                countable: *countable,
            })
        })
        .collect();
    word_spans.sort_by(|a, b| {
        a.start_ms
            .partial_cmp(&b.start_ms)
            .expect("span times are finite")
    });
    spans_out.extend(word_spans);
    t_ms
}
