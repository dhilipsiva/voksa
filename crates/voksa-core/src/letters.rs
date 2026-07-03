//! Lojban orthography → letter stream.
//!
//! Strict lowercase alphabet: the 6 vowels, 17 consonants, apostrophe (= [h],
//! intervocalic only, CLL §3.3) and comma (syllable separator, never a pause,
//! CLL §3.3/§3.9). Periods and capital stress marks are word-boundary/prosody
//! concerns handled by later phases; callers pre-strip them.

use crate::alloc::vec::Vec;
use crate::phonemes::{Consonant, Vowel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Letter {
    V(Vowel),
    C(Consonant),
    Apostrophe,
    Comma,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordError {
    Empty,
    InvalidCharacter(char),
    /// Apostrophe must sit between two vowels (CLL §3.3).
    MisplacedApostrophe,
    /// Comma may not lead, trail, or double up.
    MisplacedComma,
    /// A comma-delimited stretch with no vowel and no sonorant to serve as a
    /// syllabic nucleus.
    NoNucleus,
}

pub fn vowel_from_char(ch: char) -> Option<Vowel> {
    Some(match ch {
        'a' => Vowel::A,
        'e' => Vowel::E,
        'i' => Vowel::I,
        'o' => Vowel::O,
        'u' => Vowel::U,
        'y' => Vowel::Y,
        _ => return None,
    })
}

pub fn consonant_from_char(ch: char) -> Option<Consonant> {
    Some(match ch {
        'b' => Consonant::B,
        'c' => Consonant::C,
        'd' => Consonant::D,
        'f' => Consonant::F,
        'g' => Consonant::G,
        'j' => Consonant::J,
        'k' => Consonant::K,
        'l' => Consonant::L,
        'm' => Consonant::M,
        'n' => Consonant::N,
        'p' => Consonant::P,
        'r' => Consonant::R,
        's' => Consonant::S,
        't' => Consonant::T,
        'v' => Consonant::V,
        'x' => Consonant::X,
        'z' => Consonant::Z,
        _ => return None,
    })
}

pub fn vowel_to_char(v: Vowel) -> char {
    match v {
        Vowel::A => 'a',
        Vowel::E => 'e',
        Vowel::I => 'i',
        Vowel::O => 'o',
        Vowel::U => 'u',
        Vowel::Y => 'y',
    }
}

pub fn consonant_to_char(c: Consonant) -> char {
    match c {
        Consonant::B => 'b',
        Consonant::C => 'c',
        Consonant::D => 'd',
        Consonant::F => 'f',
        Consonant::G => 'g',
        Consonant::J => 'j',
        Consonant::K => 'k',
        Consonant::L => 'l',
        Consonant::M => 'm',
        Consonant::N => 'n',
        Consonant::P => 'p',
        Consonant::R => 'r',
        Consonant::S => 's',
        Consonant::T => 't',
        Consonant::V => 'v',
        Consonant::X => 'x',
        Consonant::Z => 'z',
    }
}

/// Parse one word (no spaces/periods) into a validated letter stream.
pub fn parse_word(word: &str) -> Result<Vec<Letter>, WordError> {
    if word.is_empty() {
        return Err(WordError::Empty);
    }
    let mut letters = Vec::new();
    for ch in word.chars() {
        let letter = if let Some(v) = vowel_from_char(ch) {
            Letter::V(v)
        } else if let Some(c) = consonant_from_char(ch) {
            Letter::C(c)
        } else if ch == '\'' {
            Letter::Apostrophe
        } else if ch == ',' {
            Letter::Comma
        } else {
            return Err(WordError::InvalidCharacter(ch));
        };
        letters.push(letter);
    }
    for (i, l) in letters.iter().enumerate() {
        match l {
            Letter::Apostrophe => {
                let prev_is_vowel = i > 0 && matches!(letters[i - 1], Letter::V(_));
                let next_is_vowel = i + 1 < letters.len() && matches!(letters[i + 1], Letter::V(_));
                if !prev_is_vowel || !next_is_vowel {
                    return Err(WordError::MisplacedApostrophe);
                }
            }
            Letter::Comma
                if i == 0 || i + 1 == letters.len() || letters[i - 1] == Letter::Comma =>
            {
                return Err(WordError::MisplacedComma);
            }
            _ => {}
        }
    }
    Ok(letters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_word() {
        let l = parse_word("coi").unwrap();
        assert_eq!(l.len(), 3);
        assert!(matches!(l[0], Letter::C(Consonant::C)));
        assert!(matches!(l[2], Letter::V(Vowel::I)));
    }

    #[test]
    fn rejects_invalid_characters() {
        assert_eq!(parse_word("hello"), Err(WordError::InvalidCharacter('h')));
        assert_eq!(parse_word("Coi"), Err(WordError::InvalidCharacter('C')));
        assert_eq!(parse_word(""), Err(WordError::Empty));
    }

    #[test]
    fn apostrophe_must_be_intervocalic() {
        assert!(parse_word("da'a").is_ok());
        assert!(parse_word("y'y").is_ok());
        assert_eq!(parse_word("'a"), Err(WordError::MisplacedApostrophe));
        assert_eq!(parse_word("a'"), Err(WordError::MisplacedApostrophe));
        assert_eq!(parse_word("a''a"), Err(WordError::MisplacedApostrophe));
        assert_eq!(parse_word("ab'a"), Err(WordError::MisplacedApostrophe));
    }

    #[test]
    fn comma_placement_rules() {
        assert!(parse_word("kat,r,in").is_ok());
        assert_eq!(parse_word(",a"), Err(WordError::MisplacedComma));
        assert_eq!(parse_word("a,"), Err(WordError::MisplacedComma));
        assert_eq!(parse_word("a,,a"), Err(WordError::MisplacedComma));
    }
}
