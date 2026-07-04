//! The phonetic sentence picker's curated set (www/sentences.json) must ALWAYS
//! synthesize: the demo fetches the file verbatim, so a morphologically bad
//! entry would break in every community browser. Native-only (`std::fs`); the
//! mirror image of tests/web.rs's wasm32 gate.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

#[derive(serde::Deserialize)]
struct Entry {
    slug: String,
    text: String,
    en: String,
    #[serde(default)]
    flags: Flags,
}

#[derive(Default, serde::Deserialize)]
#[serde(default)]
struct Flags {
    xu: bool,
    dotside: bool,
    buffer: bool,
}

#[test]
fn every_sentence_synthesizes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("www/sentences.json");
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let entries: Vec<Entry> = serde_json::from_str(&json).expect("valid sentences.json");
    assert!(
        entries.len() >= 12,
        "the phonetic coverage set should be substantial, got {}",
        entries.len()
    );
    for e in &entries {
        assert!(!e.slug.is_empty() && !e.en.is_empty(), "slug + en required");
        let mut flags = 0u32;
        if e.flags.xu {
            flags |= voksa_web::FLAG_XU;
        }
        if e.flags.dotside {
            flags |= voksa_web::FLAG_DOTSIDE;
        }
        if e.flags.buffer {
            flags |= voksa_web::FLAG_BUFFER;
        }
        let samples = voksa_web::synth(&e.text, flags, 48_000, &[])
            .unwrap_or_else(|err| panic!("{} ({:?}): {err:?}", e.slug, e.text));
        assert!(!samples.is_empty(), "{}: empty render", e.slug);
    }
}
