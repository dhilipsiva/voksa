//! CLL worked examples as fixed test vectors (section numbers cited per test).
//! Syllabifications assert voksa's documented deterministic choice among
//! CLL-valid variants (docs/phonology.md §3).

use voksa_core::classify::{WordClass, classify};
use voksa_core::letters::WordError;
use voksa_core::phonemes::Consonant;
use voksa_core::syllable::{Nucleus, syllabify, to_text};

fn syls(word: &str) -> Vec<String> {
    syllabify(word)
        .unwrap_or_else(|e| panic!("{word}: {e:?}"))
        .iter()
        .map(|s| to_text(core::slice::from_ref(s)))
        .collect()
}

// ---- syllabification (CLL §3.5, §3.9, §4.7, §4.8) ----

#[test]
fn cll_9_1_pujenaicajeba() {
    assert_eq!(syls("pujenaicajeba"), ["pu", "je", "nai", "ca", "je", "ba"]);
}

#[test]
fn cll_9_2_ninmu() {
    assert_eq!(syls("ninmu"), ["nin", "mu"]);
}

#[test]
fn cll_9_3_fitpri() {
    assert_eq!(syls("fitpri"), ["fit", "pri"]);
}

#[test]
fn cll_9_4_sairgoi() {
    // rg is not an initial pair: r stays as coda (non-syllabic variant chosen).
    assert_eq!(syls("sairgoi"), ["sair", "goi"]);
}

#[test]
fn cll_9_5_klezba() {
    // zb IS an initial pair: "normally both assigned to the following vowel".
    assert_eq!(syls("klezba"), ["kle", "zba"]);
}

#[test]
fn cll_9_6_dikyjvo() {
    assert_eq!(syls("dikyjvo"), ["di", "ky", "jvo"]);
}

#[test]
fn cll_9_7_armstrong() {
    assert_eq!(syls("armstrong"), ["arm", "strong"]);
    let s = syllabify("armstrong").unwrap();
    assert_eq!(s[0].coda, [Consonant::R, Consonant::M]);
    assert_eq!(s[1].onset, [Consonant::S, Consonant::T, Consonant::R]);
}

#[test]
fn cll_9_11_bisydja() {
    assert_eq!(syls("bisydja"), ["bi", "sy", "dja"]);
}

#[test]
fn cll_9_12_dahudja() {
    // Apostrophe forces the break and aspirates the following syllable.
    assert_eq!(syls("da'udja"), ["da", "'u", "dja"]);
    let s = syllabify("da'udja").unwrap();
    assert!(s[1].aspirated);
    assert!(!s[0].aspirated && !s[2].aspirated);
}

#[test]
fn cll_5_1_meiin_pairs_vowels_from_the_left() {
    assert_eq!(syls("meiin"), ["mei", "in"]);
}

#[test]
fn cll_5_2_me_comma_iin_overrides() {
    assert_eq!(syls("me,iin"), ["me", "iin"]);
}

#[test]
fn cll_8_6_katrin_comma_forces_syllabic_r() {
    assert_eq!(syls("kat,r,in"), ["kat", "r", "in"]);
    let s = syllabify("kat,r,in").unwrap();
    assert_eq!(s[1].nucleus, Nucleus::Syllabic(Consonant::R));
}

#[test]
fn cll_4_1_brlgan_forces_syllabic_sonorants() {
    assert_eq!(syls("brlgan"), ["br", "l", "gan"]);
    let s = syllabify("brlgan").unwrap();
    assert_eq!(s[0].nucleus, Nucleus::Syllabic(Consonant::R));
    assert_eq!(s[0].onset, [Consonant::B]);
    assert_eq!(s[1].nucleus, Nucleus::Syllabic(Consonant::L));
}

#[test]
fn cll_3_4_rl_two_syllabic_consonants() {
    assert_eq!(syls("rl"), ["r", "l"]);
}

#[test]
fn cll_7_12_bangrkorea_ea_is_not_a_diphthong() {
    assert_eq!(syls("bang,r,kore,a"), ["bang", "r", "ko", "re", "a"]);
}

#[test]
fn cll_4_7_spraile_legal_triple_onset() {
    assert_eq!(syls("spraile"), ["sprai", "le"]);
}

#[test]
fn cll_4_7_ktraile_is_unsyllabifiable() {
    // k cannot join the tr onset and is no sonorant: typed error, no panic.
    assert_eq!(syllabify("ktraile"), Err(WordError::NoNucleus));
}

#[test]
fn phonology_vectors_vecnu_pofygau_brivla_ernace_klama() {
    assert_eq!(syls("vecnu"), ["ve", "cnu"]);
    assert_eq!(syls("pofygau"), ["po", "fy", "gau"]);
    assert_eq!(syls("brivla"), ["bri", "vla"]);
    assert_eq!(syls("ernace"), ["er", "na", "ce"]);
    assert_eq!(syls("klama"), ["kla", "ma"]);
    assert_eq!(syls("lojban"), ["lo", "jban"]); // jb is an initial pair
}

// ---- classification (CLL §4.2, §4.3, §4.5, §4.7, §4.8) ----

fn class(word: &str) -> WordClass {
    classify(word).unwrap_or_else(|e| panic!("{word}: {e:?}"))
}

#[test]
fn cll_4_3_dahamei_is_compound_cmavo() {
    assert_eq!(class("da'amei"), WordClass::Cmavo);
}

#[test]
fn cll_4_3_lojban_is_cmevla() {
    assert_eq!(class("lojban"), WordClass::Cmevla);
}

#[test]
fn cll_4_3_bisycla_pair_need_not_be_permissible() {
    // The qualifying pair is "sc" — impermissible (both from cjsz) yet decisive.
    assert_eq!(class("bisycla"), WordClass::Brivla);
}

#[test]
fn cll_4_3_rohinreho_apostrophes_uncounted() {
    assert_eq!(class("ro'inre'o"), WordClass::Brivla);
}

#[test]
fn cll_4_5_soirsai_pair_at_letters_four_five() {
    assert_eq!(class("soirsai"), WordClass::Brivla);
}

#[test]
fn cll_4_7_shapes() {
    assert_eq!(class("xaceru"), WordClass::Cmavo); // no consonant pair
    assert_eq!(class("kobra"), WordClass::Brivla);
    assert_eq!(class("spageti"), WordClass::Brivla);
    assert_eq!(class("kuarka"), WordClass::Brivla);
}

#[test]
fn cll_4_5_lujvo_forms_are_brivla() {
    assert_eq!(class("brivla"), WordClass::Brivla);
    assert_eq!(class("bridyvla"), WordClass::Brivla);
    assert_eq!(class("bridyvalsi"), WordClass::Brivla);
    assert_eq!(class("vecnu"), WordClass::Brivla);
    assert_eq!(class("klama"), WordClass::Brivla);
}

#[test]
fn cll_4_2_cmavo_forms() {
    assert_eq!(class("y"), WordClass::Cmavo);
    assert_eq!(class("y'y"), WordClass::Cmavo);
    assert_eq!(class("cy"), WordClass::Cmavo); // ends in y, never brivla
    assert_eq!(class("a"), WordClass::Cmavo);
    assert_eq!(class("iseci'i"), WordClass::Cmavo);
}

#[test]
fn cll_4_8_cmevla_end_in_consonants() {
    for name in ["pav", "ralj", "djan", "djein", "djansn", "tcarlz", "arnold"] {
        assert_eq!(class(name), WordClass::Cmevla, "{name}");
    }
}

#[test]
fn classify_propagates_parse_errors() {
    assert_eq!(classify(""), Err(WordError::Empty));
    assert_eq!(classify("hello"), Err(WordError::InvalidCharacter('h')));
}
