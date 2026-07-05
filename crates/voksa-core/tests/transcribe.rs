//! D2c acceptance: the phonetic transcription renders exactly what the
//! compiler speaks — syllables (`.`), stress (CAPS, the CLL convention),
//! pauses (`‖`, mandatory + writer periods merged), buffer vowels (`(ɪ)`),
//! and number/lerfu normalization. Vectors follow the pinned CLL behavior in
//! cll_stress_pauses.rs.

use voksa_core::compiler::CompileOptions;
use voksa_core::transcribe::transcribe;

fn t(text: &str) -> String {
    transcribe(text, &CompileOptions::default()).unwrap_or_else(|e| panic!("{text}: {e:?}"))
}

fn t_with(text: &str, opts: CompileOptions) -> String {
    transcribe(text, &opts).unwrap_or_else(|e| panic!("{text}: {e:?}"))
}

#[test]
fn stress_caps_and_syllable_dots() {
    // coi = cmavo (unstressed); munje = brivla, penultimate stress on MUN.
    assert_eq!(t("coi munje"), "coi MUN.je");
    assert_eq!(t("xu do klama"), "xu do KLA.ma");
}

#[test]
fn cmevla_pauses_and_la_exemption() {
    // djan: la-family exempts the leading pause; consonant-final forces the
    // trailing one; the writer period merges into it (CLL §4.2).
    assert_eq!(t("la djan. cu klama"), "la DJAN ‖ cu KLA.ma");
    // coi is NOT in the la-family: pause BEFORE djan too (CLL §4.9 r4).
    assert_eq!(t("coi djan. cu klama"), "coi ‖ DJAN ‖ cu KLA.ma");
}

#[test]
fn dotside_drops_the_exemption() {
    assert_eq!(
        t_with(
            "la djan. cu klama",
            CompileOptions {
                dotside: true,
                ..Default::default()
            }
        ),
        "la ‖ DJAN ‖ cu KLA.ma"
    );
}

#[test]
fn marked_stress_and_collision_pause() {
    // e'U (CLL §3.9 ex 9.13): explicit stress on the final syllable; vowel-
    // initial → leading pause; final stress before a brivla → collision pause.
    assert_eq!(t("e'U bridi"), "‖ e.'U ‖ BRI.di");
}

#[test]
fn syllabic_consonants_and_comma_drop() {
    // kat,r,in: syllabic r is its own (uncountable) syllable → stress KAT;
    // input commas are replaced by syllable dots; cmevla pauses on both sides
    // (leading exempted by la).
    assert_eq!(t("la kat,r,in. klama"), "la KAT.r.in ‖ KLA.ma");
}

#[test]
fn utterance_initial_cmevla_pauses_both_sides() {
    assert_eq!(t("kat,r,in"), "‖ KAT.r.in ‖");
}

#[test]
fn numbers_show_their_normalization() {
    assert_eq!(t("li 3.14"), "li ci pi pa vo");
}

#[test]
fn buffer_vowels_render_inline() {
    let buffered = CompileOptions {
        buffer: true,
        ..Default::default()
    };
    // CLL §3.8 ex 8.1: vrusi → v[ɪ]rusi. The buffer splits the VRU onset; the
    // marker keeps the stressed syllable's caps around it.
    assert_eq!(t_with("vrusi", buffered), "V(ɪ)RU.si");
    assert_eq!(t_with("le zdani", buffered), "le Z(ɪ)DA.ni");
    // Unbuffered baselines for contrast.
    assert_eq!(t("vrusi"), "VRU.si");
    assert_eq!(t("le zdani"), "le ZDA.ni");
}

#[test]
fn aspirated_syllables_keep_the_apostrophe() {
    // da'udja: apostrophe = [h] onto the 'u syllable; brivla penultimate
    // countable stress lands on 'u.
    assert_eq!(t("mi djica le da'udja"), "mi DJI.ca le da.'U.dja");
}

#[test]
fn transcription_is_deterministic() {
    assert_eq!(t("coi la djan. cu klama"), t("coi la djan. cu klama"));
}
