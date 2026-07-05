//! The pinned source column: what is spoken (utterance + flags + sentence
//! picker), the live phonetic analysis, transport, and the share card.
//! C2 ships the utterance panel live; transcript/transport/share arrive in
//! C3/C5 (placeholders keep the layout honest).

use dioxus::prelude::*;
use serde::Deserialize;

use super::Ui;
use super::speak::{Audio, Status, speak_now};
use super::store::ParamStore;
use crate::audio;
use crate::engine;
use crate::model::{Flags, TokKind, peaks, tokenize, wav_bytes};

/// Waveform display columns.
const WAVE_COLS: usize = 240;

#[derive(Deserialize, Clone, PartialEq)]
struct Sentence {
    slug: String,
    text: String,
    en: String,
    #[serde(default)]
    flags: SentenceFlags,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(default)]
struct SentenceFlags {
    xu: bool,
    dotside: bool,
    buffer: bool,
}

fn sentences() -> &'static [Sentence] {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Vec<Sentence>> = OnceLock::new();
    CACHE.get_or_init(|| {
        serde_json::from_str(crate::assets::SENTENCES_JSON)
            .expect("sentences.json is gated by a native test")
    })
}

/// The source column wrapper.
#[component]
pub fn SourceColumn() -> Element {
    rsx! {
        div { class: "vx-left",
            UtterancePanel {}
            TranscriptCard {}
            TransportCard {}
            ShareCard {}
        }
    }
}

/// Utterance: text input + sentence picker + flag chips.
#[component]
fn UtterancePanel() -> Element {
    let mut ui = use_context::<Ui>();
    let set = sentences();
    let sel = ui.sentence.read().clone();
    let selected = set.iter().position(|s| s.slug == sel);
    let gloss = selected.map(|i| set[i].en.clone()).unwrap_or_default();
    let counter = selected
        .map(|i| format!("{:02}/{}", i + 1, set.len()))
        .unwrap_or_default();

    let mut apply_sentence = move |i: usize| {
        let s = &sentences()[i];
        ui.text.set(s.text.clone());
        ui.sentence.set(s.slug.clone());
        let mut flags = *ui.flags.peek();
        flags.xu = s.flags.xu;
        flags.dotside = s.flags.dotside;
        flags.buffer = s.flags.buffer;
        ui.flags.set(flags);
    };

    rsx! {
        section { class: "vx-card",
            div { class: "vx-cardhead", span { class: "vx-slash", "// " } "utterance" }
            input {
                class: "vx-text",
                r#type: "text",
                value: "{ui.text}",
                aria_label: "Lojban text",
                spellcheck: false,
                oninput: move |e| {
                    ui.text.set(e.value());
                    ui.sentence.set(String::new());
                },
            }
            div { class: "vx-pickrow",
                select {
                    class: "vx-select",
                    aria_label: "test sentences",
                    onchange: move |e| {
                        if let Some(i) = sentences().iter().position(|s| s.slug == e.value()) {
                            apply_sentence(i);
                        } else {
                            ui.sentence.set(String::new());
                        }
                    },
                    option { value: "", selected: selected.is_none(), "— free text —" }
                    for s in set.iter() {
                        option { value: "{s.slug}", selected: sel == s.slug, "{s.text}" }
                    }
                }
                button {
                    class: "vx-next",
                    title: "cycle the phonetic coverage set",
                    onclick: move |_| {
                        let next = sentences()
                            .iter()
                            .position(|s| s.slug == *ui.sentence.peek())
                            .map(|i| (i + 1) % sentences().len())
                            .unwrap_or(0);
                        apply_sentence(next);
                    },
                    "⟳"
                }
                span { class: "vx-counter", "{counter}" }
            }
            if !gloss.is_empty() {
                p { class: "vx-gloss", "{gloss}" }
            }
            div { class: "vx-flagrow",
                FlagChip { key: "flat", flag: FlagKind::Flat }
                FlagChip { key: "xu", flag: FlagKind::Xu }
                FlagChip { key: "dotside", flag: FlagKind::Dotside }
                FlagChip { key: "buffer", flag: FlagKind::Buffer }
            }
        }
    }
}

/// Which flag a chip controls.
#[derive(Clone, Copy, PartialEq)]
pub enum FlagKind {
    /// `--flat`.
    Flat,
    /// `--xu`.
    Xu,
    /// `--dotside`.
    Dotside,
    /// `--buffer`.
    Buffer,
}

impl FlagKind {
    fn label(self) -> (&'static str, &'static str) {
        match self {
            FlagKind::Flat => ("flat", "no prosody"),
            FlagKind::Xu => ("xu", "question rise"),
            FlagKind::Dotside => ("dotside", "name dialect"),
            FlagKind::Buffer => ("buffer", "[ɪ] in clusters"),
        }
    }
    fn get(self, f: Flags) -> bool {
        match self {
            FlagKind::Flat => f.flat,
            FlagKind::Xu => f.xu,
            FlagKind::Dotside => f.dotside,
            FlagKind::Buffer => f.buffer,
        }
    }
    fn toggle(self, f: &mut Flags) {
        match self {
            FlagKind::Flat => {
                f.flat = !f.flat;
                if f.flat {
                    f.xu = false; // xu is meaningless without prosody
                }
            }
            FlagKind::Xu => f.xu = !f.xu,
            FlagKind::Dotside => f.dotside = !f.dotside,
            FlagKind::Buffer => f.buffer = !f.buffer,
        }
    }
}

/// One compile/prosody flag as a toggle chip.
#[component]
fn FlagChip(flag: FlagKind) -> Element {
    let mut ui = use_context::<Ui>();
    let flags = ui.flags;
    let on = flag.get(flags());
    let disabled = flag == FlagKind::Xu && flags().flat;
    let (label, sub) = flag.label();
    rsx! {
        button {
            class: "vx-flag",
            class: if on { "vx-flag-on" },
            disabled,
            aria_pressed: on,
            onclick: move |_| {
                let mut f = *ui.flags.peek();
                flag.toggle(&mut f);
                ui.flags.set(f);
            },
            span { class: "vx-flagdot" }
            span { class: "vx-flagname", "{label}" }
            span { class: "vx-flagsub", "{sub}" }
        }
    }
}

fn tok_class(kind: TokKind) -> &'static str {
    match kind {
        TokKind::Stress => "vx-t-stress",
        TokKind::Dot => "vx-t-dot",
        TokKind::Pause => "vx-t-pause",
        TokKind::Buffer => "vx-t-buffer",
        TokKind::Aspirate => "vx-t-asp",
        TokKind::Plain => "vx-t-plain",
    }
}

/// Live phonetic analysis — recomputed on every text/flag change (a cheap
/// sync engine call; covers programmatic changes too via the memo).
#[component]
fn TranscriptCard() -> Element {
    let ui = use_context::<Ui>();
    let toks = use_memo(move || {
        let text = ui.text.read().clone();
        let flags = *ui.flags.read();
        engine::transcribe(&text, flags).map(|s| tokenize(&s)).ok()
    });
    rsx! {
        section { class: "vx-card",
            div { class: "vx-cardhead",
                span { class: "vx-slash", "// " }
                "phonetic analysis"
                span { class: "vx-legend", "CAPS stress · ‖ pause · (ɪ) buffer" }
            }
            div { class: "vx-transcript",
                match toks() {
                    Some(list) if !list.is_empty() => rsx! {
                        span { class: "vx-tprefix", "→ " }
                        for (i, tok) in list.iter().enumerate() {
                            span { key: "{i}", class: tok_class(tok.kind), "{tok.text}" }
                        }
                    },
                    Some(_) => rsx! { span { class: "vx-tfaint", "—" } },
                    None => rsx! { span { class: "vx-tfaint", "can't transcribe that" } },
                }
            }
        }
    }
}

fn wave_path(pcm: &[f32]) -> String {
    let cols = peaks(pcm, WAVE_COLS);
    let mut d = String::new();
    for (x, &p) in cols.iter().enumerate() {
        let h = (p.clamp(0.0, 1.0) * 48.0).max(0.35);
        d.push_str(&format!("M{x} {:.2}L{x} {:.2}", 50.0 - h, 50.0 + h));
    }
    d
}

/// Transport: speak, auto-speak toggle, waveform, status, WAV download.
#[component]
fn TransportCard() -> Element {
    let store = use_context::<ParamStore>();
    let ui = use_context::<Ui>();
    let audio = use_context::<Audio>();
    let mut auto = ui.auto_speak;

    let path = use_memo(move || ui.pcm.read().as_ref().map(|p| wave_path(p)));
    let has_pcm = use_memo(move || ui.pcm.read().is_some());
    let status = ui.status.read().clone();
    let (status_text, status_kind) = match &status {
        Status::Booting => ("booting…".to_string(), "boot"),
        Status::Ready => ("ready".to_string(), "ok"),
        Status::Speaking => ("speaking…".to_string(), "alive"),
        Status::NeedsGesture => ("tap ▶ speak to enable audio".to_string(), "boot"),
        Status::Error(m) => (m.clone(), "err"),
    };

    let audio_speak = audio.clone();
    rsx! {
        section { class: "vx-card",
            div { class: "vx-transportrow",
                button {
                    class: "vx-speak",
                    onclick: move |_| speak_now(store, ui, audio_speak.clone()),
                    "▶ speak"
                }
                label { class: "vx-switch",
                    input {
                        r#type: "checkbox",
                        checked: auto(),
                        onchange: move |e| auto.set(e.checked()),
                    }
                    span { "speak on change" }
                }
            }
            div { class: "vx-wave",
                svg {
                    class: "vx-wavesvg",
                    view_box: "0 0 {WAVE_COLS} 100",
                    preserve_aspect_ratio: "none",
                    line {
                        class: "vx-wavemid",
                        x1: "0",
                        y1: "50",
                        x2: "{WAVE_COLS}",
                        y2: "50",
                    }
                    if let Some(d) = path() {
                        path { class: "vx-waveline", d: "{d}" }
                    }
                }
            }
            div { class: "vx-transportfoot",
                div { class: "vx-status vx-status-{status_kind}", "{status_text}" }
                button {
                    class: "vx-wav",
                    disabled: !has_pcm(),
                    onclick: move |_| {
                        if let Some(pcm) = ui.pcm.peek().as_ref() {
                            let bytes = wav_bytes(pcm, audio::SAMPLE_RATE);
                            audio::download("voksa.wav", &bytes, "audio/wav");
                        }
                    },
                    "⤓ wav"
                }
            }
        }
    }
}

/// Share the tuning (export/load — wired in C5).
#[component]
fn ShareCard() -> Element {
    let ui = use_context::<Ui>();
    let mut notes = ui.notes;
    rsx! {
        section { class: "vx-card",
            div { class: "vx-cardhead", span { class: "vx-slash", "// " } "share the tuning" }
            textarea {
                class: "vx-notes",
                placeholder: "notes travel inside the exported JSON",
                value: "{notes}",
                oninput: move |e| notes.set(e.value()),
            }
            div { class: "vx-status", "export / load land in C5" }
        }
    }
}
