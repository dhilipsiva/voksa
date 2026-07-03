//! Phase-4 acceptance: CLL stress vectors (§3.1, §3.9, §4.8) and mandatory
//! pause rules (§4.9 r1–r7, §4.2, §17.2, §19.10), plus properties.

use proptest::prelude::*;
use voksa_core::classify::WordClass;
use voksa_core::letters::WordError;
use voksa_core::pause::{Segment, Token, insert_pauses};
use voksa_core::stress::is_countable;
use voksa_core::word::analyze_word;

fn stress_of(raw: &str) -> Option<usize> {
    analyze_word(raw)
        .unwrap_or_else(|e| panic!("{raw}: {e:?}"))
        .stress
}

fn word(raw: &str) -> Token {
    Token::Word(analyze_word(raw).unwrap_or_else(|e| panic!("{raw}: {e:?}")))
}

/// Render a segment stream compactly: "." = pause, otherwise the word text.
fn render(segments: &[Segment]) -> Vec<String> {
    segments
        .iter()
        .map(|s| match s {
            Segment::Word(w) => w.lowered.clone(),
            Segment::Foreign(t) => format!("[{t}]"),
            Segment::Pause => ".".to_string(),
        })
        .collect()
}

fn pauses(words: &[&str], dotside: bool) -> Vec<String> {
    render(&insert_pauses(
        words.iter().map(|w| word(w)).collect(),
        dotside,
    ))
}

// ---- stress: CLL worked examples ----

#[test]
fn cll_stress_y_and_syllabic_exclusions() {
    assert_eq!(stress_of("dikyjvo"), Some(0)); // DI,ky,jvo — ky not counted
    assert_eq!(stress_of("bisydja"), Some(0)); // BI,sy,dja
    assert_eq!(stress_of("kat,r,in"), Some(0)); // syllabic r skipped -> KAT
    assert_eq!(stress_of("brlgan"), Some(2)); // gan is the only countable
    assert_eq!(stress_of("djansn"), Some(0)); // single-syllable name
    assert_eq!(stress_of("rl"), None); // no countable syllable at all
}

#[test]
fn cll_stress_penultimate_defaults() {
    assert_eq!(stress_of("armstrong"), Some(0)); // .ARM,strong.
    assert_eq!(stress_of("klama"), Some(0)); // gismu
    assert_eq!(stress_of("da'udja"), Some(1)); // da'UD,ja
    assert_eq!(stress_of("djosefin"), Some(1)); // se without capitals (§3.1)
}

#[test]
fn cll_cmavo_unstressed_by_default() {
    assert_eq!(stress_of("le"), None);
    assert_eq!(stress_of("pujenaicajeba"), None); // compound cmavo
    assert_eq!(stress_of("e'u"), None);
}

#[test]
fn cll_capital_marked_stress() {
    assert_eq!(stress_of("DJOsefin"), Some(0)); // §3.1 verbatim
    assert_eq!(stress_of("eLIS"), Some(1)); // §4.8 Elise
    assert_eq!(stress_of("dyGOL"), Some(1)); // §4.8 De Gaulle
    assert_eq!(stress_of("xrucTCOF"), Some(1)); // §4.8 Khrushchev
    assert_eq!(stress_of("xuaKIN"), Some(1)); // §4.8 Joaquin
    assert_eq!(stress_of("e'U"), Some(1)); // §3.9 ex 9.13
}

#[test]
fn cll_3_1_capitalizing_the_vowel_is_sufficient() {
    assert_eq!(stress_of("djOsefin"), stress_of("DJOsefin"));
}

#[test]
fn iy_uy_syllables_are_uncountable() {
    // b + iy + n = one syllable whose vowel is [e] -- never stressed.
    assert_eq!(stress_of("biyn"), None);
}

#[test]
fn invalid_stress_marks_are_rejected() {
    assert_eq!(analyze_word("dY"), Err(WordError::InvalidStressMark)); // y syllable
    assert_eq!(analyze_word("klaMA"), Err(WordError::InvalidStressMark)); // brivla off-penultimate
    assert_eq!(analyze_word("DJoSEfin"), Err(WordError::InvalidStressMark)); // two syllables
    let ok = analyze_word("KLAma").unwrap(); // matches the default penultimate
    assert_eq!(ok.stress, Some(0));
    assert_eq!(ok.class, WordClass::Brivla);
}

// ---- pauses: CLL §4.9 rules ----

#[test]
fn rule4_pause_before_cmevla_coi_djan() {
    // phonology.md's previously-missing rule: coi is not in {la lai la'i doi}.
    assert_eq!(pauses(&["coi", "djan"], false), ["coi", ".", "djan", "."]);
}

#[test]
fn rule4_la_family_exemption() {
    assert_eq!(pauses(&["la", "lojban"], false), ["la", "lojban", "."]);
    assert_eq!(pauses(&["doi", "djan"], false), ["doi", "djan", "."]);
}

#[test]
fn dotside_drops_the_exemption() {
    assert_eq!(pauses(&["la", "djan"], true), ["la", ".", "djan", "."]);
}

#[test]
fn rule3_vowel_initial_and_merging() {
    // .alis is vowel-initial AND a cmevla: one merged pause (CLL §4.2).
    assert_eq!(pauses(&["doi", "alis"], false), ["doi", ".", "alis", "."]);
}

#[test]
fn rule5_stress_collision_before_brivla() {
    // Both open with a pause: e'u is vowel-initial (r3, CLL writes .e'u).
    // The pause BETWEEN the words is the r5 stress-collision one.
    assert_eq!(pauses(&["e'U", "bridi"], false), [".", "e'u", ".", "bridi"]);
    assert_eq!(
        pauses(&["E'u", "bridi"], false),
        [".", "e'u", "bridi"] // stress not final: no collision pause
    );
    assert_eq!(
        pauses(&["le", "RE", "nanmu"], false),
        ["le", "re", ".", "nanmu"]
    );
    assert_eq!(
        pauses(&["le", "re", "nanmu"], false),
        ["le", "re", "nanmu"] // default cmavo stress fires nothing
    );
}

#[test]
fn hesitation_y_pauses_both_sides() {
    assert_eq!(
        pauses(&["mi", "y", "klama"], false),
        ["mi", ".", "y", ".", "klama"]
    );
}

#[test]
fn y_final_cmavo_rules() {
    assert_eq!(
        pauses(&["mi", "cy", "claxu"], false),
        ["mi", "cy", ".", "claxu"]
    );
    assert_eq!(pauses(&["cy", "by"], false), ["cy", "by"]); // Cy before Cy: none
    assert_eq!(
        pauses(&["cy", "ibu", "abu"], false),
        ["cy", ".", "ibu", ".", "abu"] // merged pause between cy/.ibu (§4.2)
    );
}

#[test]
fn foreign_text_is_pause_bracketed() {
    let tokens = vec![
        word("zoi"),
        word("gy"),
        Token::Foreign("John is a man".to_string()),
        word("gy"),
        word("cu"),
    ];
    assert_eq!(
        render(&insert_pauses(tokens, false)),
        ["zoi", "gy", ".", "[John is a man]", ".", "gy", ".", "cu"]
    );
}

#[test]
fn commas_never_pause() {
    assert_eq!(pauses(&["kat,r,in"], false), [".", "kat,r,in", "."]);
}

#[test]
fn utterance_edges() {
    // Vowel-initial first word gets a leading pause (glottal onset);
    // consonant-final last word a trailing one.
    assert_eq!(pauses(&["alis"], false), [".", "alis", "."]);
    assert_eq!(pauses(&["klama"], false), ["klama"]);
}

// ---- properties ----

fn gismu_strategy() -> impl Strategy<Value = String> {
    // CV + valid initial pair + V (a CCVCV-ish generator using the pair table).
    prop::sample::select(&voksa_core::clusters::INITIAL_PAIRS[..]).prop_flat_map(|(c1, c2)| {
        (
            prop::sample::select(&['a', 'e', 'i', 'o', 'u'][..]),
            prop::sample::select(&['b', 'd', 'k', 'l', 'm', 'n', 'r', 's', 't'][..]),
            prop::sample::select(&['a', 'e', 'i', 'o', 'u'][..]),
        )
            .prop_map(move |(v1, c3, v2)| {
                let mut s = String::new();
                s.push(voksa_core::letters::consonant_to_char(c1));
                s.push(voksa_core::letters::consonant_to_char(c2));
                s.push(v1);
                s.push(c3);
                s.push(v2);
                s
            })
    })
}

proptest! {
    /// Stress always lands on a countable syllable (or nowhere).
    #[test]
    fn stress_is_always_countable(w in gismu_strategy(), tail in prop::sample::select(&['b', 'd', 'k', 'n', 's'][..])) {
        let brivla = analyze_word(&w).unwrap();
        if let Some(i) = brivla.stress {
            prop_assert!(is_countable(&brivla.syllables[i]));
        }
        let mut name = w;
        name.push(tail);
        let cmevla = analyze_word(&name).unwrap();
        prop_assert_eq!(cmevla.class, WordClass::Cmevla);
        if let Some(i) = cmevla.stress {
            prop_assert!(is_countable(&cmevla.syllables[i]));
        }
    }

    /// Pause insertion is deterministic, never doubles pauses, and every
    /// cmevla is followed by a pause.
    #[test]
    fn pause_stream_invariants(ws in prop::collection::vec(
        prop_oneof![gismu_strategy(), Just("la".to_string()), Just("cy".to_string()), Just("djan".to_string())],
        1..6,
    )) {
        let make = || insert_pauses(ws.iter().map(|w| word(w)).collect::<Vec<_>>(), false);
        let a = make();
        prop_assert_eq!(&a, &make()); // deterministic
        for pair in a.windows(2) {
            prop_assert!(!(pair[0] == Segment::Pause && pair[1] == Segment::Pause));
        }
        for (i, seg) in a.iter().enumerate() {
            if let Segment::Word(w) = seg {
                if w.class == WordClass::Cmevla {
                    prop_assert_eq!(a.get(i + 1), Some(&Segment::Pause), "cmevla must be followed by a pause");
                }
            }
        }
    }
}
