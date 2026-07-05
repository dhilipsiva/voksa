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

fn classify(c: char) -> TokKind {
    match c {
        '‖' => TokKind::Pause,
        '.' => TokKind::Dot,
        '\'' => TokKind::Aspirate,
        _ if c.is_uppercase() => TokKind::Stress,
        _ => TokKind::Plain,
    }
}

/// Classify the transcription string into styled runs (adjacent same-kind
/// characters merge; `(ɪ)` is one Buffer token).
pub fn tokenize(s: &str) -> Vec<Tok> {
    let chars: Vec<char> = s.chars().collect();
    let mut toks: Vec<Tok> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        // The buffer vowel is a fixed 3-char unit; detect it before per-char
        // classification so it never merges into a neighboring run.
        if chars[i] == '(' && chars.get(i + 1) == Some(&'ɪ') && chars.get(i + 2) == Some(&')') {
            toks.push(Tok {
                text: "(ɪ)".to_string(),
                kind: TokKind::Buffer,
            });
            i += 3;
            continue;
        }
        let kind = classify(chars[i]);
        match toks.last_mut() {
            Some(last) if last.kind == kind => last.text.push(chars[i]),
            _ => toks.push(Tok {
                text: chars[i].to_string(),
                kind,
            }),
        }
        i += 1;
    }
    toks
}
