//! Phase-6 acceptance: number → PA-cmavo conversion (CLL §18.2/§18.3/§18.10
//! vectors), the round-trip property, the lerfu table (§17.2/§17.4/§17.5),
//! and pause-rule reproduction of CLL's written lerfu forms.

use proptest::prelude::*;
use voksa_core::compiler::{CompileError, CompileOptions, compile};
use voksa_core::normalize::{NumberError, hex_word, lerfu_words, number_words, read_number, spell};
use voksa_core::pause::{Segment, Token, insert_pauses};
use voksa_core::word::analyze_word;

fn nums(figure: &str) -> Vec<&'static str> {
    number_words(figure).unwrap_or_else(|e| panic!("{figure}: {e:?}"))
}

// ---- number vectors (CLL-cited) ----

#[test]
fn cll_18_2_digit_by_digit() {
    assert_eq!(nums("42"), ["vo", "re"]);
    assert_eq!(nums("0"), ["no"]);
    assert_eq!(nums("10"), ["pa", "no"]); // CLL 18.2.2
    assert_eq!(nums("123"), ["pa", "re", "ci"]); // CLL 18.2.1
    assert_eq!(nums("007"), ["no", "no", "ze"]);
}

#[test]
fn cll_18_3_decimal_pi() {
    assert_eq!(nums("3.14"), ["ci", "pi", "pa", "vo"]);
    assert_eq!(nums("0.5"), ["no", "pi", "mu"]);
}

#[test]
fn cll_18_3_kiho_thousands() {
    // Full-group emission policy (never the elided short-group form).
    assert_eq!(nums("1,000"), ["pa", "ki'o", "no", "no", "no"]);
    assert_eq!(nums("2,000"), ["re", "ki'o", "no", "no", "no"]);
    // CLL 18.3.7 verbatim.
    assert_eq!(
        nums("1,234,567"),
        ["pa", "ki'o", "re", "ci", "vo", "ki'o", "mu", "xa", "ze"]
    );
    assert_eq!(
        nums("1,023,004"),
        ["pa", "ki'o", "no", "re", "ci", "ki'o", "no", "no", "vo"]
    );
}

#[test]
fn cll_18_10_pihe_clock_times() {
    assert_eq!(nums("3:30"), ["ci", "pi'e", "ci", "no"]);
    // CLL 18.10's clock example (there written compounded: ci pi'e rere pi'e vono).
    assert_eq!(
        nums("3:22:40"),
        ["ci", "pi'e", "re", "re", "pi'e", "vo", "no"]
    );
}

#[test]
fn number_errors() {
    assert_eq!(number_words("1,00"), Err(NumberError::MalformedGrouping));
    assert_eq!(number_words("1,0000"), Err(NumberError::MalformedGrouping));
    assert_eq!(number_words("1F"), Err(NumberError::LetterInNumber));
    assert_eq!(number_words("x2"), Err(NumberError::LetterInNumber));
}

#[test]
fn hex_vocabulary_exported_but_not_auto_detected() {
    assert_eq!(hex_word('a'), Some("dau"));
    assert_eq!(hex_word('F'), Some("vai"));
    assert_eq!(hex_word('g'), None);
    // v1 policy: figures with letters are errors, not hex.
    assert_eq!(number_words("FF"), Err(NumberError::LetterInNumber));
    assert_eq!(number_words("0xFF"), Err(NumberError::LetterInNumber));
}

// ---- inverse + round-trip ----

#[test]
fn read_number_inverse_units() {
    assert_eq!(read_number(&["vo", "re"]).as_deref(), Some("42"));
    assert_eq!(
        read_number(&["pa", "ki'o", "no", "no", "no"]).as_deref(),
        Some("1,000")
    );
    assert_eq!(
        read_number(&["ci", "pi", "pa", "vo"]).as_deref(),
        Some("3.14")
    );
    assert_eq!(
        read_number(&["ci", "pi'e", "ci", "no"]).as_deref(),
        Some("3:30")
    );
    assert_eq!(read_number(&["klama"]), None);
}

static DIGIT_CHARS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

fn digits(len: core::ops::Range<usize>) -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(&DIGIT_CHARS[..]), len)
        .prop_map(|v| v.into_iter().collect())
}

fn canonical_figure() -> impl Strategy<Value = String> {
    // int with optional comma groups, optional decimal part, 1-2 pi'e parts.
    let int = prop_oneof![
        digits(1..7),
        (digits(1..4), prop::collection::vec(digits(3..4), 1..3)).prop_map(|(head, groups)| {
            let mut s = head;
            for g in groups {
                s.push(',');
                s.push_str(&g);
            }
            s
        }),
    ];
    let part = (int, prop::option::of(digits(1..4))).prop_map(|(i, frac)| match frac {
        Some(f) => alloc_format(&i, &f),
        None => i,
    });
    (part, prop::option::of(digits(1..3))).prop_map(|(p, extra)| match extra {
        Some(e) => {
            let mut s = p;
            s.push(':');
            s.push_str(&e);
            s
        }
        None => p,
    })
}

fn alloc_format(i: &str, f: &str) -> String {
    let mut s = String::from(i);
    s.push('.');
    s.push_str(f);
    s
}

proptest! {
    /// The playbook's round-trip property: digits → PA cmavo → digits.
    #[test]
    fn number_round_trips(figure in canonical_figure()) {
        let words = number_words(&figure).expect("canonical figures convert");
        let back = read_number(&words);
        prop_assert_eq!(back.as_deref(), Some(figure.as_str()));
    }
}

// ---- lerfu ----

#[test]
fn lerfu_table_is_complete() {
    for c in "bcdfgjklmnprstvxz".chars() {
        let words = lerfu_words(c).unwrap_or_else(|| panic!("{c}"));
        assert_eq!(words.len(), 1);
        assert!(words[0].ends_with('y'), "{c} -> {words:?}");
    }
    for v in "aeiouy".chars() {
        let words = lerfu_words(v).unwrap();
        assert_eq!(words.len(), 1);
        assert!(words[0].ends_with("bu"), "{v} -> {words:?}");
    }
    assert_eq!(lerfu_words('\''), Some(&["y'y"][..]));
    assert_eq!(lerfu_words('h'), Some(&["y'y", "bu"][..])); // CLL §17.5
    assert_eq!(lerfu_words('q'), Some(&["ky", "bu"][..]));
    assert_eq!(lerfu_words('w'), Some(&["vy", "bu"][..]));
    assert_eq!(lerfu_words('.'), Some(&["denpa", "bu"][..]));
    assert_eq!(lerfu_words(','), Some(&["slaka", "bu"][..]));
}

#[test]
fn spell_vectors() {
    assert_eq!(spell("abc").unwrap(), ["abu", "by", "cy"]);
    assert_eq!(
        spell("hq w").unwrap(),
        ["y'y", "bu", "ky", "bu", "vy", "bu"]
    );
    assert_eq!(spell("B2").unwrap(), ["by", "re"]); // case-insensitive; digits ok
    assert_eq!(spell("a!b"), Err('!'));
}

// ---- pause-rule reproduction of CLL's written lerfu forms ----

fn word(raw: &str) -> Token {
    Token::Word(analyze_word(raw).unwrap_or_else(|e| panic!("{raw}: {e:?}")))
}

fn rendered(words: &[&str]) -> Vec<String> {
    insert_pauses(words.iter().map(|w| word(w)).collect(), false)
        .iter()
        .map(|s| match s {
            Segment::Word(w) => w.lowered.clone(),
            Segment::Pause => ".".to_string(),
            Segment::Foreign(t) => t.clone(),
        })
        .collect()
}

#[test]
fn pause_rules_reproduce_cll_written_lerfu() {
    // ky.bu — the y-final rule supplies the dot.
    assert_eq!(rendered(&["ky", "bu"]), ["ky", ".", "bu"]);
    // .y'y.bu — vowel-initial + y-final rules supply both dots.
    assert_eq!(rendered(&["y'y", "bu"]), [".", "y'y", ".", "bu"]);
    // denpa bu — "No pause is required between denpa (or slaka) and bu".
    assert_eq!(rendered(&["denpa", "bu"]), ["denpa", "bu"]);
    // Spelled "abc": .abu (vowel-initial). No rule fires between abu and by
    // (abu is vowel-final, by is consonant-initial), nor between by and cy
    // (both y-final cmavo).
    assert_eq!(rendered(&["abu", "by", "cy"]), [".", "abu", "by", "cy"]);
}

// ---- end-to-end through compile ----

#[test]
fn compile_speaks_figures() {
    let s = compile("mi 42 klama", &CompileOptions::default()).unwrap();
    // mi + vo + re + klama = 4 words; klama alone is stressed (brivla).
    let word_indices: Vec<usize> = s.spans.iter().map(|sp| sp.word_index).collect();
    assert_eq!(word_indices.iter().max(), Some(&3));
    assert_eq!(s.spans.iter().filter(|sp| sp.stressed).count(), 1);
    insta::assert_debug_snapshot!(s);
}

#[test]
fn compile_speaks_decimals_via_tokenizer_lookahead() {
    // "3.14" must NOT split at the period.
    let s = compile("li 3.14", &CompileOptions::default()).unwrap();
    // li ci pi pa vo = 5 words.
    assert_eq!(s.spans.iter().map(|sp| sp.word_index).max(), Some(4usize));
    assert_eq!(
        compile("li 3.14", &CompileOptions::default()).unwrap(),
        s,
        "deterministic"
    );
}

#[test]
fn compile_reports_malformed_figures() {
    assert_eq!(
        compile("mi 1,00 klama", &CompileOptions::default()),
        Err(CompileError::MalformedNumber {
            figure: "1,00".into(),
            error: NumberError::MalformedGrouping,
        })
    );
}
