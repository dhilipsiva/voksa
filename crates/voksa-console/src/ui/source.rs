//! The pinned source column: what is spoken (utterance + flags + sentence
//! picker), the live phonetic analysis, transport, and the share card.
//! C2 ships the utterance panel live; transcript/transport/share arrive in
//! C3/C5 (placeholders keep the layout honest).

use dioxus::html::HasFileData;
use dioxus::prelude::*;
use serde::Deserialize;

use super::Ui;
use super::about::{LINK_ISSUES, LINK_MAIL};
use super::help::HelpDot;
use super::speak::{Audio, Status, speak_now};
use super::store::ParamStore;
use crate::audio;
use crate::engine;
use crate::model::{
    self, ExportInputs, Flags, PARAM_TOTAL, TokKind, WritePlan, peaks, tokenize, wav_bytes,
};

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
            div { class: "vx-cardhead",
                span { class: "vx-slash", "// " }
                "utterance"
                span { class: "vx-scopespacer" }
                HelpDot { topic: "input.text", title: "utterance" }
            }
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
                HelpDot { topic: "transcript.line", title: "phonetic analysis" }
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
                HelpDot { topic: "transport.speak", title: "speak" }
                label { class: "vx-switch",
                    input {
                        r#type: "checkbox",
                        checked: auto(),
                        onchange: move |e| auto.set(e.checked()),
                    }
                    span { "speak on change" }
                }
                HelpDot { topic: "transport.autospeak", title: "speak on change" }
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
                span { class: "vx-scopespacer" }
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
                HelpDot { topic: "transport.wav", title: "wav download" }
            }
        }
    }
}

/// A one-line share-card status (load confirmation or a bad-JSON error).
#[derive(Clone, PartialEq)]
struct ShareMsg {
    text: String,
    err: bool,
}

/// Apply a parsed config with REPLACE semantics (a clean slate, not a merge):
/// overlay the full 449-value block, then adopt its text/flags/notes. Out-of-
/// range values widen their sliders (the store recomputes widen per cell).
/// Malformed JSON errors before any state changes.
fn apply_loaded(
    json: &str,
    name: &str,
    mut store: ParamStore,
    mut ui: Ui,
    mut loaded_name: Signal<Option<String>>,
    mut msg: Signal<Option<ShareMsg>>,
) {
    let parsed = {
        let desc = store.desc();
        model::load(&desc, json)
    };
    match parsed {
        Ok(plan) => {
            let write = WritePlan(plan.values.iter().copied().enumerate().collect());
            store.apply(&write);
            if let Some(text) = plan.text {
                ui.text.set(text);
            }
            ui.flags.set(plan.flags);
            ui.notes.set(plan.notes);
            ui.preset.set("Custom".to_string());
            ui.sentence.set(String::new());
            loaded_name.set(Some(name.to_string()));
            msg.set(Some(ShareMsg {
                text: format!("config loaded — {name}"),
                err: false,
            }));
        }
        Err(e) => msg.set(Some(ShareMsg {
            text: format!("bad config JSON — {e}"),
            err: true,
        })),
    }
}

/// Share the tuning: Δ summary, notes, export/load (+ drag-drop), and the
/// "send it back" call to action. Load REPLACES all state.
#[component]
fn ShareCard() -> Element {
    let store = use_context::<ParamStore>();
    let ui = use_context::<Ui>();
    let mut notes = ui.notes;
    let dirty = use_memo(move || store.dirty_count(0..PARAM_TOTAL));
    let mut drop_active = use_signal(|| false);
    let loaded_name = use_signal(|| None::<String>);
    let msg = use_signal(|| None::<ShareMsg>);

    rsx! {
        section {
            class: "vx-card vx-sharecard",
            ondragover: move |e| {
                e.prevent_default();
                if !*drop_active.peek() {
                    drop_active.set(true);
                }
            },
            ondragleave: move |_| drop_active.set(false),
            ondrop: move |e| {
                e.prevent_default();
                drop_active.set(false);
                if let Some(file) = e.files().into_iter().next() {
                    let name = file.name();
                    spawn(async move {
                        match file.read_string().await {
                            Ok(json) => apply_loaded(&json, &name, store, ui, loaded_name, msg),
                            Err(_) => {
                                let mut msg = msg;
                                msg.set(Some(ShareMsg { text: "couldn't read that file".into(), err: true }));
                            }
                        }
                    });
                }
            },
            div { class: "vx-cardhead",
                span { class: "vx-slash", "// " }
                "share the tuning"
                span { class: "vx-scopespacer" }
                HelpDot { topic: "share.cta", title: "share the tuning" }
            }

            if dirty() > 0 {
                div { class: "vx-sharedelta vx-sharedelta-on", "Δ {dirty()} deviations from engine defaults" }
            } else {
                div { class: "vx-sharedelta", "at engine defaults — nothing to send yet" }
            }

            textarea {
                class: "vx-notes",
                placeholder: "notes travel inside the exported JSON — say what you heard",
                value: "{notes}",
                oninput: move |e| notes.set(e.value()),
            }

            div { class: "vx-sharebtns",
                button {
                    class: "vx-sharebtn",
                    r#type: "button",
                    onclick: move |_| {
                        let values = store.snapshot();
                        let text = ui.text.peek().clone();
                        let flags = *ui.flags.peek();
                        let notes_s = ui.notes.peek().clone();
                        let phonetics = engine::transcribe(&text, flags).unwrap_or_default();
                        let json = {
                            let desc = store.desc();
                            model::export(
                                &desc,
                                &ExportInputs {
                                    values: &values,
                                    text: &text,
                                    flags,
                                    notes: &notes_s,
                                    phonetics: &phonetics,
                                    sample_rate: audio::SAMPLE_RATE,
                                },
                            )
                        };
                        audio::download("voksa-config.json", json.as_bytes(), "application/json");
                    },
                    "⤒ export config"
                }
                HelpDot { topic: "share.export", title: "export config" }
                label { class: "vx-sharebtn", r#for: "vx-load-input", "⤓ load config" }
                HelpDot { topic: "share.load", title: "load config" }
                input {
                    id: "vx-load-input",
                    class: "vx-fileinput",
                    r#type: "file",
                    accept: ".json,application/json",
                    onchange: move |e| {
                        if let Some(file) = e.files().into_iter().next() {
                            let name = file.name();
                            spawn(async move {
                                match file.read_string().await {
                                    Ok(json) => apply_loaded(&json, &name, store, ui, loaded_name, msg),
                                    Err(_) => {
                                        let mut msg = msg;
                                        msg.set(Some(ShareMsg { text: "couldn't read that file".into(), err: true }));
                                    }
                                }
                            });
                        }
                    },
                }
            }

            div { class: "vx-sharefoot",
                if let Some(name) = loaded_name.read().as_ref() {
                    span { class: "vx-loadchip", "◈ {name}" }
                }
                match msg.read().as_ref() {
                    Some(m) => rsx! {
                        span {
                            class: if m.err { "vx-status vx-status-err" } else { "vx-status vx-status-load" },
                            "{m.text}"
                        }
                    },
                    None => rsx! {},
                }
            }

            div { class: "vx-cta",
                div { class: "vx-cta-head", "send it back" }
                p { class: "vx-cta-body",
                    "your ears are the ground truth — the JSON carries your notes and the phonetics you were reading. It replays bit-identically via "
                    span { class: "vx-mono", "voksa --config" }
                    "."
                }
                div { class: "vx-cta-links",
                    a {
                        class: "vx-cta-link",
                        href: LINK_ISSUES,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "open an issue"
                    }
                    a { class: "vx-cta-link", href: LINK_MAIL, "email a config" }
                }
            }

            if drop_active() {
                div { class: "vx-droplay",
                    span { "drop config JSON — replaces all state" }
                }
            }
        }
    }
}
