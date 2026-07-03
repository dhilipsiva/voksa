//! Number → PA-cmavo normalization (CLL §18.2/§18.3/§18.10) and letteral
//! (lerfu) words (CLL §17.2/§17.4/§17.5).
//!
//! Numbers are read digit-by-digit as separate cmavo words (CLL's own style:
//! `pa re ci` = 123); `pi` = decimal point, `ki'o` = thousands separator
//! (emitted with FULL three-digit groups — the short-group elision CLL
//! permits is never emitted), `pi'e` = compound-base separator (clock times).
//! Hex digit cmavo (dau..vai) are exported vocabulary but not auto-detected
//! in v1 — a bare `1F` is indistinguishable from a typo without a ju'u
//! context. Lerfu spelling is case-insensitive; the ga'e/to'a case shifts
//! change letter MEANING, not sound, and are not emitted.

use crate::alloc::string::String;
use crate::alloc::vec::Vec;

/// PA digit words 0–9 (CLL §18.2).
pub fn digit_word(d: char) -> Option<&'static str> {
    Some(match d {
        '0' => "no",
        '1' => "pa",
        '2' => "re",
        '3' => "ci",
        '4' => "vo",
        '5' => "mu",
        '6' => "xa",
        '7' => "ze",
        '8' => "bi",
        '9' => "so",
        _ => return None,
    })
}

/// Hex digit words A–F (CLL §18.10). Exported vocabulary; not wired into the
/// tokenizer in v1 (see module docs).
pub fn hex_word(c: char) -> Option<&'static str> {
    Some(match c.to_ascii_lowercase() {
        'a' => "dau",
        'b' => "fei",
        'c' => "gai",
        'd' => "jau",
        'e' => "rei",
        'f' => "vai",
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberError {
    /// Comma groups must be canonical: first 1–3 digits, the rest exactly 3.
    MalformedGrouping,
    /// Letters inside a figure (hex is not auto-detected in v1).
    LetterInNumber,
}

/// Convert one written figure (digits with optional `,` groups, `.` decimal
/// part, `:` compound-base separators) into PA cmavo words.
pub fn number_words(figure: &str) -> Result<Vec<&'static str>, NumberError> {
    let _ = figure;
    todo!("Phase 6 red checkpoint: number conversion lands after the failing tests are committed")
}

/// Inverse of [`number_words`] (round-trip property): PA words back to the
/// canonical written figure. `None` if a word isn't part of a number.
pub fn read_number(words: &[&str]) -> Option<String> {
    let _ = words;
    todo!("Phase 6 red checkpoint")
}

/// Lerfu word(s) naming one character (CLL §17.2/§17.4/§17.5). Multi-word
/// entries (h/q/w, denpa bu, slaka bu) are separate cmavo/gismu tokens — the
/// existing pause rules reproduce CLL's written dotted forms (.y'y.bu, ky.bu).
pub fn lerfu_words(ch: char) -> Option<&'static [&'static str]> {
    Some(match ch.to_ascii_lowercase() {
        'a' => &["abu"],
        'e' => &["ebu"],
        'i' => &["ibu"],
        'o' => &["obu"],
        'u' => &["ubu"],
        'y' => &["ybu"],
        'b' => &["by"],
        'c' => &["cy"],
        'd' => &["dy"],
        'f' => &["fy"],
        'g' => &["gy"],
        'j' => &["jy"],
        'k' => &["ky"],
        'l' => &["ly"],
        'm' => &["my"],
        'n' => &["ny"],
        'p' => &["py"],
        'r' => &["ry"],
        's' => &["sy"],
        't' => &["ty"],
        'v' => &["vy"],
        'x' => &["xy"],
        'z' => &["zy"],
        '\'' => &["y'y"],
        'h' => &["y'y", "bu"],
        'q' => &["ky", "bu"],
        'w' => &["vy", "bu"],
        '.' => &["denpa", "bu"],
        ',' => &["slaka", "bu"],
        '0' => &["no"],
        '1' => &["pa"],
        '2' => &["re"],
        '3' => &["ci"],
        '4' => &["vo"],
        '5' => &["mu"],
        '6' => &["xa"],
        '7' => &["ze"],
        '8' => &["bi"],
        '9' => &["so"],
        _ => return None,
    })
}

/// Spell arbitrary text letter-by-letter as lerfu words (whitespace skipped,
/// case-insensitive). Errors with the first unnameable character.
pub fn spell(text: &str) -> Result<Vec<&'static str>, char> {
    let _ = text;
    todo!("Phase 6 red checkpoint")
}
