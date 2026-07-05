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

/// Why a written figure could not be normalized to PA cmavo.
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
    if figure.chars().any(|c| c.is_ascii_alphabetic()) {
        return Err(NumberError::LetterInNumber);
    }
    let mut out = Vec::new();
    for (part_index, part) in figure.split(':').enumerate() {
        if part_index > 0 {
            out.push("pi'e");
        }
        let mut halves = part.splitn(2, '.');
        let int = halves.next().unwrap_or("");
        let frac = halves.next();
        push_integer(int, &mut out)?;
        if let Some(frac) = frac {
            if frac.is_empty() {
                return Err(NumberError::MalformedGrouping);
            }
            out.push("pi");
            for d in frac.chars() {
                out.push(digit_word(d).ok_or(NumberError::MalformedGrouping)?);
            }
        }
    }
    Ok(out)
}

/// Integer part: plain digits, or canonical comma groups (first 1–3 digits,
/// every subsequent group exactly 3) emitted as ki'o-separated FULL groups.
fn push_integer(int: &str, out: &mut Vec<&'static str>) -> Result<(), NumberError> {
    if int.is_empty() {
        return Err(NumberError::MalformedGrouping);
    }
    let groups: Vec<&str> = int.split(',').collect();
    if groups.len() == 1 {
        // Plain digit run: any length (CLL 18.2.3 strings ten digits).
        if groups[0].is_empty() {
            return Err(NumberError::MalformedGrouping);
        }
        for d in groups[0].chars() {
            out.push(digit_word(d).ok_or(NumberError::MalformedGrouping)?);
        }
        return Ok(());
    }
    // Comma-grouped: canonical grouping only (head 1-3, rest exactly 3).
    if groups[0].is_empty() || groups[0].len() > 3 {
        return Err(NumberError::MalformedGrouping);
    }
    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            if group.len() != 3 {
                return Err(NumberError::MalformedGrouping);
            }
            out.push("ki'o");
        }
        for d in group.chars() {
            out.push(digit_word(d).ok_or(NumberError::MalformedGrouping)?);
        }
    }
    Ok(())
}

/// Inverse of [`number_words`] (round-trip property): PA words back to the
/// canonical written figure. `None` if a word isn't part of a number.
pub fn read_number(words: &[&str]) -> Option<String> {
    let mut out = String::new();
    for word in words {
        match *word {
            "pi" => out.push('.'),
            "ki'o" => out.push(','),
            "pi'e" => out.push(':'),
            other => out.push(digit_char(other)?),
        }
    }
    Some(out)
}

fn digit_char(word: &str) -> Option<char> {
    Some(match word {
        "no" => '0',
        "pa" => '1',
        "re" => '2',
        "ci" => '3',
        "vo" => '4',
        "mu" => '5',
        "xa" => '6',
        "ze" => '7',
        "bi" => '8',
        "so" => '9',
        _ => return None,
    })
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
    let mut out = Vec::new();
    for ch in text.chars() {
        if ch.is_whitespace() {
            continue;
        }
        match lerfu_words(ch) {
            Some(words) => out.extend_from_slice(words),
            None => return Err(ch),
        }
    }
    Ok(out)
}
