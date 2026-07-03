//! Hand-rolled CLI argument parsing (no clap — keeps the dependency tree
//! minimal, matching the xtask style).

use std::fmt;
use std::path::PathBuf;

/// Parsed CLI arguments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CliArgs {
    /// The Lojban text to speak (non-flag tokens joined by single spaces).
    pub text: String,
    /// `--out PATH`: render a WAV to PATH instead of playing live.
    pub out: Option<PathBuf>,
    /// `--dotside`: leading pause before every cmevla.
    pub dotside: bool,
    /// `--buffer`: epenthetic buffer vowels between consonant pairs.
    pub buffer: bool,
    /// `--xu`: terminal question rise (prosodic path only).
    pub xu: bool,
    /// `--flat`: render without prosody (the Phase-5 baseline).
    pub flat: bool,
}

/// Why argument parsing failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgError {
    UnknownFlag(String),
    MissingOutPath,
    NoText,
    /// `--xu` is a prosody feature; it cannot combine with `--flat`.
    XuWithFlat,
}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgError::UnknownFlag(s) => write!(f, "unknown flag: {s}"),
            ArgError::MissingOutPath => write!(f, "--out requires a file path"),
            ArgError::NoText => write!(f, "no text to speak"),
            ArgError::XuWithFlat => write!(f, "--xu cannot be combined with --flat"),
        }
    }
}

/// Parse an argument iterator (already skipping argv[0]).
pub fn parse(args: impl Iterator<Item = String>) -> Result<CliArgs, ArgError> {
    // STUB (Phase 8 red): real parsing lands after the failing tests commit.
    let _ = args;
    Ok(CliArgs::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(s: &str) -> Result<CliArgs, ArgError> {
        parse(s.split_whitespace().map(String::from))
    }

    #[test]
    fn plain_text_joins_words() {
        let a = parse_str("coi munje").unwrap();
        assert_eq!(a.text, "coi munje");
        assert!(a.out.is_none() && !a.dotside && !a.buffer && !a.xu && !a.flat);
    }

    #[test]
    fn flags_parse_in_any_order_interleaved() {
        let a = parse_str("--dotside coi --buffer munje --xu").unwrap();
        assert_eq!(a.text, "coi munje");
        assert!(a.dotside && a.buffer && a.xu && !a.flat);
    }

    #[test]
    fn out_captures_following_path() {
        let a = parse_str("--out x.wav coi munje").unwrap();
        assert_eq!(a.out.as_deref(), Some(std::path::Path::new("x.wav")));
        assert_eq!(a.text, "coi munje");
    }

    #[test]
    fn out_without_value_errors() {
        assert_eq!(
            parse(["--out".to_string()].into_iter()),
            Err(ArgError::MissingOutPath)
        );
    }

    #[test]
    fn unknown_flag_errors() {
        assert_eq!(
            parse_str("--nope coi"),
            Err(ArgError::UnknownFlag("--nope".into()))
        );
    }

    #[test]
    fn no_text_errors() {
        assert_eq!(parse_str("--dotside"), Err(ArgError::NoText));
    }

    #[test]
    fn xu_with_flat_errors() {
        assert_eq!(parse_str("--xu --flat coi"), Err(ArgError::XuWithFlat));
    }
}
