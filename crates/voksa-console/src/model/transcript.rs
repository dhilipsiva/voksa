//! Styled tokenization of the engine's phonetic transcription string. The
//! reference page set `textContent` flat; the QUINE design colors the markers
//! (stress / dot / pause / buffer / aspirate), so we classify runs of the
//! `voksa_web::transcription` output. Notation (docs: `voksa_core::transcribe`):
//! UPPERCASE = stressed syllable, `.` = syllable dot, `‖` = pause,
//! `(ɪ)` = epenthetic buffer vowel, `'` = `[h]`.

/// The visual class of a transcription run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokKind {
    /// A stressed syllable (uppercase run).
    Stress,
    /// A syllable boundary dot.
    Dot,
    /// A pause marker (`‖`).
    Pause,
    /// An epenthetic buffer vowel (`(ɪ)`).
    Buffer,
    /// An apostrophe / `[h]`.
    Aspirate,
    /// Everything else (unstressed letters, spaces, cmavo).
    Plain,
}

/// One classified run of the transcription string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tok {
    /// The run's text.
    pub text: String,
    /// Its visual class.
    pub kind: TokKind,
}

/// Classify the transcription string into styled runs (adjacent same-kind
/// characters merge; `(ɪ)` is one Buffer token).
pub fn tokenize(s: &str) -> Vec<Tok> {
    let _ = s;
    Vec::new() // stub — C3 green
}
