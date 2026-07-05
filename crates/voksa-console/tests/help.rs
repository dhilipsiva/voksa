//! Help-registry completeness: every `?` affordance the console mounts must
//! resolve to authored copy, so a shipped build never shows the
//! `// help pending` fallback. This is the gate that forces all help entries
//! to be written (C5) and keeps them written as the UI grows.

use std::collections::HashMap;

use voksa_console::assets::HELP_TEXT_JSON;
use voksa_console::help::help_for;
use voksa_console::model::Descriptors;

/// The registry keys the components hardcode that are NOT descriptor
/// `help_key`s (those are covered by [`descriptor_help_keys_all_resolve`]).
/// Kept in lockstep with the `?` dots the source/racks/editors mount.
const UI_KEYS: &[&str] = &[
    // utterance + transcript
    "input.text",
    "input.sentences",
    "transcript.line",
    // flags
    "flags.flat",
    "flags.xu",
    "flags.dotside",
    "flags.buffer",
    // transport
    "transport.speak",
    "transport.autospeak",
    "transport.waveform",
    "transport.wav",
    // share loop
    "share.export",
    "share.load",
    "share.notes",
    "share.cta",
    "presets",
    "reset.all",
    // rack + section heads
    "prosody._section",
    "naturalness._section",
    "naturalness.ab",
    "att._section",
    "vt._section",
    "vt.changedonly",
    // attitudinal chips
    "att.emotion.ui",
    "att.emotion.uu",
    "att.emotion.oi",
    "att.emotion.ii",
    "att.emotion.o'o",
    "att.emotion.au",
    "att.emotion.o'onai",
    // voice-table manner classes
    "vt.class.vowels",
    "vt.class.diphthongs",
    "vt.class.stops",
    "vt.class.fricatives",
    "vt.class.sonorants",
    "vt.class.other",
];

/// Every entry in the shipped registry must be authored (no empty strings).
/// `_`-prefixed keys are metadata (the `_readme`) and are exempt.
#[test]
fn every_registry_entry_is_authored() {
    let map: HashMap<String, String> =
        serde_json::from_str(HELP_TEXT_JSON).expect("help-text.json is valid JSON");
    let mut empty: Vec<&String> = map
        .iter()
        .filter(|(k, v)| !k.starts_with('_') && v.trim().is_empty())
        .map(|(k, _)| k)
        .collect();
    empty.sort();
    assert!(
        empty.is_empty(),
        "{} help entries are unauthored (empty): {:?}",
        empty.len(),
        empty
    );
}

/// Every descriptor's `help_key` (the field-level keys shared across all
/// phonemes/emotions) resolves to authored copy, not the pending fallback.
#[test]
fn descriptor_help_keys_all_resolve() {
    let desc = Descriptors::from_defaults(&voksa_web::default_params())
        .expect("engine default block is the frozen 449-float layout");
    for i in 0..desc.len() {
        let key = &desc.get(i).help_key;
        assert!(
            !help_for(key).starts_with("// help pending"),
            "descriptor {i} help_key `{key}` has no authored entry"
        );
    }
}

/// Every non-descriptor UI key the components hardcode resolves to authored
/// copy, not the pending fallback.
#[test]
fn ui_keys_all_resolve() {
    for key in UI_KEYS {
        assert!(
            !help_for(key).starts_with("// help pending"),
            "UI key `{key}` has no authored entry"
        );
    }
}

/// The mechanism contract: an unknown key yields the pending fallback verbatim.
#[test]
fn unknown_key_yields_pending_fallback() {
    assert_eq!(
        help_for("does.not.exist"),
        "// help pending — does.not.exist"
    );
}
