//! Config JSON export/load — the community sharing loop. The schema is the
//! CLI's (`voksa --config`): flat text/flags/knobs always, attitudinals and
//! phonemes DELTA-ONLY (stops nest `closure`/`burst`), plus the
//! `phonetics`/`notes`/`sampleRate`/`voksaVersion` stamps. Load = REPLACE.

use super::descriptor::Descriptors;

/// The compile/prosody flag set (mirrors the CLI flags + wasm bit layout).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    /// No prosody at all (`--flat`; disables the voice-shaping controls).
    pub flat: bool,
    /// Terminal question rise (`--xu`).
    pub xu: bool,
    /// Leading pause before every cmevla (`--dotside`).
    pub dotside: bool,
    /// Epenthetic buffer vowels (`--buffer`).
    pub buffer: bool,
}

impl Flags {
    /// The wasm `flags` bit word (flat 0x1, xu 0x2, dotside 0x4, buffer 0x8).
    pub fn bits(self) -> u32 {
        let mut b = 0;
        if self.flat {
            b |= voksa_web::FLAG_FLAT;
        }
        if self.xu {
            b |= voksa_web::FLAG_XU;
        }
        if self.dotside {
            b |= voksa_web::FLAG_DOTSIDE;
        }
        if self.buffer {
            b |= voksa_web::FLAG_BUFFER;
        }
        b
    }
}

/// Everything an export stamps into the config JSON.
#[derive(Debug, Clone, Copy)]
pub struct ExportInputs<'a> {
    /// The full 449-value parameter snapshot.
    pub values: &'a [f32],
    /// The utterance text.
    pub text: &'a str,
    /// The flag set.
    pub flags: Flags,
    /// The tuner's notes (travel inside the JSON).
    pub notes: &'a str,
    /// The phonetic transcription of `text` (what the tuner was looking at).
    pub phonetics: &'a str,
    /// The playback sample rate.
    pub sample_rate: u32,
}

/// Export the config JSON (pretty-printed). Knobs export as flat named
/// fields ALWAYS; attitudinals/phonemes export only f32-dirty values.
pub fn export(desc: &Descriptors, inp: &ExportInputs<'_>) -> String {
    let _ = (desc, inp);
    String::new() // stub — C1 green
}

/// A parsed config, resolved to REPLACE semantics: a full 449-value block
/// (engine defaults overlaid with the config's keys) plus text/flags/notes.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadPlan {
    /// The full parameter block to apply (always 449 values).
    pub values: Vec<f32>,
    /// The utterance text (`None` = keep the current text).
    pub text: Option<String>,
    /// The flag set (absent keys = false, like the CLI).
    pub flags: Flags,
    /// The notes field ("" when absent).
    pub notes: String,
}

/// Parse a config JSON into a [`LoadPlan`]. Unknown fields are ignored
/// (CLI semantics); malformed JSON or non-numeric values error without
/// touching any state.
pub fn load(desc: &Descriptors, json: &str) -> Result<LoadPlan, String> {
    let _ = (desc, json);
    Err("stub — C1 green".into())
}
