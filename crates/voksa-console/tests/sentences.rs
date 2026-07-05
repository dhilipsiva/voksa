//! The sentence-picker gate: every curated coverage sentence must ALWAYS
//! synthesize — the console embeds the file verbatim, so a morphologically
//! bad entry would break in every community browser. (Ported from the
//! voksa-web gate; this copy owns the design bundle's refreshed 18-entry set.)

use serde::Deserialize;

#[derive(Deserialize)]
struct Entry {
    slug: String,
    text: String,
    en: String,
    #[serde(default)]
    flags: Flags,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Flags {
    xu: bool,
    dotside: bool,
    buffer: bool,
}

#[test]
fn every_curated_sentence_synthesizes() {
    let entries: Vec<Entry> =
        serde_json::from_str(voksa_console::assets::SENTENCES_JSON).expect("valid sentences.json");
    assert_eq!(entries.len(), 18, "the design's curated coverage set");
    for e in &entries {
        assert!(!e.slug.is_empty() && !e.en.is_empty());
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
        let pcm = voksa_web::synth(&e.text, flags, 48_000, &[])
            .unwrap_or_else(|err| panic!("{} ({:?}): {err:?}", e.slug, e.text));
        assert!(!pcm.is_empty(), "{} rendered empty", e.slug);
    }
}
