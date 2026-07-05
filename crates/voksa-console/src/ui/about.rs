//! The "about voksa" modal (opened by the `λ` button in the title row) and
//! the canonical project links. Content mirrors
//! `docs/design/tuning-console/voksa-engine-data.json` (capabilities / sizes /
//! architecture / links) — hardcoded, not fetched.

use dioxus::prelude::*;

use super::Ui;

/// Source repository.
pub const LINK_REPO: &str = "https://github.com/dhilipsiva/voksa";
/// "New issue" — where community tunings and notes come back.
pub const LINK_ISSUES: &str = "https://github.com/dhilipsiva/voksa/issues/new";
/// Maintainer contact.
pub const LINK_MAIL: &str = "mailto:dhilipsiva@pm.me";

/// (glyph, statement) — what voksa is, five ways.
const CAPABILITIES: [(&str, &str); 5] = [
    (
        "⊢",
        "pure Rust, rule-based parallel-formant (Klatt-style) synthesis — no ML, no recorded voices, no eSpeak data",
    ),
    (
        "≡",
        "deterministic: text → parameter schedule → PCM; the same input always renders the identical schedule (snapshot-tested)",
    ),
    (
        "∀",
        "every CLL rule implemented: 17 consonants, 6 vowels, 16 diphthongs, syllabification §3.9, penultimate stress, cmevla/brivla/cmavo classification, mandatory pauses, number normalization §18",
    ),
    (
        "λ",
        "449 runtime parameters: 7 prosody + 9 naturalness knobs, 7×8 attitudinal deviation vectors, the full 377-float per-phoneme voice table",
    ),
    (
        "∴",
        "community-tunable by design: phoneme values are documented conventions, not facts — shared config JSON replays bit-identically via voksa --config",
    ),
];

/// (metric, value, note).
const SIZES: [(&str, &str, &str); 4] = [
    (
        "voksa_web_bg.wasm",
        "42 kB gzipped",
        "gate asserts < 43 kB · zero imports",
    ),
    (
        "runtime",
        "AudioWorklet",
        "no server, no network after load",
    ),
    ("render", "< 100 ms", "whole utterance, typical sentence"),
    ("MSRV", "1.85", "MIT OR Apache-2.0"),
];

/// (crate, one-line role).
const ARCHITECTURE: [(&str, &str); 6] = [
    (
        "voksa-core",
        "no_std text front-end + schedule compiler (all CLL rules)",
    ),
    (
        "voksa-engine-klattsch",
        "std adapter: schedule IR → klatt parameter events",
    ),
    (
        "klattsch-core-fork",
        "vendored klattsch-core 0.1.1 (MIT, Tony Gies) + OQ/diplophonia/flutter",
    ),
    (
        "voksa-cli",
        "native playback (cpal) + offline WAV + --config replay",
    ),
    (
        "voksa-web",
        "zero-import C-ABI wasm + AudioWorklet + this console",
    ),
    (
        "voksa-component",
        "wasm32-wasip2 WIT component (voksa:synth)",
    ),
];

/// The about modal — rendered nothing until `Ui.about_open` is set.
#[component]
pub fn AboutPanel() -> Element {
    let mut ui = use_context::<Ui>();
    if !*ui.about_open.read() {
        return rsx! {};
    }
    rsx! {
        div {
            class: "vx-about-backdrop",
            onclick: move |_| ui.about_open.set(false),
            div {
                class: "vx-about-card",
                onclick: move |e| e.stop_propagation(),
                div { class: "vx-about-head",
                    span { class: "vx-about-mark", "voksa" }
                    span { class: "vx-slash", "// about voksa" }
                    span { class: "vx-about-gloss", "x1 is the voice of x2" }
                    span { class: "vx-scopespacer" }
                    button {
                        class: "vx-pop-x",
                        r#type: "button",
                        aria_label: "close about",
                        onclick: move |_| ui.about_open.set(false),
                        "✕"
                    }
                }
                ul { class: "vx-caps",
                    for (g, t) in CAPABILITIES.iter() {
                        li { key: "{g}", class: "vx-cap",
                            span { class: "vx-cap-g", "{g}" }
                            span { class: "vx-cap-t", "{t}" }
                        }
                    }
                }
                div { class: "vx-about-sec", "sizes" }
                table { class: "vx-sizes",
                    tbody {
                        for (k, v, n) in SIZES.iter() {
                            tr { key: "{k}",
                                td { class: "vx-size-k", "{k}" }
                                td { class: "vx-size-v", "{v}" }
                                td { class: "vx-size-n", "{n}" }
                            }
                        }
                    }
                }
                div { class: "vx-about-sec", "architecture" }
                ul { class: "vx-arch",
                    for (c, d) in ARCHITECTURE.iter() {
                        li { key: "{c}", class: "vx-arch-row",
                            span { class: "vx-arch-c", "{c}" }
                            span { class: "vx-arch-d", "{d}" }
                        }
                    }
                }
                div { class: "vx-about-links",
                    a { href: LINK_REPO, target: "_blank", rel: "noopener noreferrer", "source" }
                    a { href: LINK_ISSUES, target: "_blank", rel: "noopener noreferrer", "issues" }
                    a { href: LINK_MAIL, "dhilipsiva@pm.me" }
                }
                div { class: "vx-about-foot", "One must imagine the synthesizer happy." }
            }
        }
    }
}
