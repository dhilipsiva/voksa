//! Rack C (attitudinals) + rack D (voice table): the emotion chips → editor
//! and the phoneme grid → per-phoneme editor. Only ONE emotion editor and ONE
//! phoneme editor are mounted at a time (swapped via `key`); selection is a
//! cheap `use_memo` per chip/keycap.

use dioxus::prelude::*;

use super::Ui;
use super::help::HelpDot;
use super::rows::{MatrixCell, ParamRow};
use super::speak::{Audio, speak_now};
use super::store::ParamStore;
use crate::model::{ATT_KINDS, ItemKind, SECTIONS, VOICE_ITEMS, VoiceItem, reset_plan};

/// Grid/editor display for a voice item (buffer shows its IPA symbol).
fn disp(vi: &VoiceItem) -> &str {
    if vi.key == "buffer" { "ɪ" } else { vi.key }
}

/// The amber `∴` note shown above a dur-only or buffer editor.
fn item_note(item: usize) -> Option<&'static str> {
    let vi = &VOICE_ITEMS[item];
    match vi.key {
        "buffer" => Some(
            "the epenthetic vowel the buffer flag inserts inside clusters — never written, never stressed",
        ),
        "'" => Some(
            "aspiration shapes itself from the NEXT vowel's formants — duration is the only free parameter",
        ),
        _ if SECTIONS[vi.section as usize].id == "diphthongs" => {
            Some("glide endpoints follow the tuned vowels — duration is the only free parameter")
        }
        _ => None,
    }
}

// ─── rack C: attitudinals ─────────────────────────────────────────────────

/// The attitudinal rack: theorem callout + 7 emotion chips + one editor.
#[component]
pub fn AttitudinalSection() -> Element {
    let store = use_context::<ParamStore>();
    let ui = use_context::<Ui>();
    let att_dirty = use_memo(move || store.dirty_count(7..63));
    let sel = ui.sel_emotion;
    rsx! {
        section { class: "vx-card vx-rack", id: "attitudinals",
            div { class: "vx-cardhead",
                span { class: "vx-slash", "// " }
                "attitudinals"
                span { class: "vx-racksub", "cnima'o — colour a word with affect" }
                if att_dirty() > 0 {
                    span { class: "vx-chip vx-chip-ember", "Δ {att_dirty()}" }
                }
                HelpDot { topic: "att._section", title: "attitudinals" }
            }
            div { class: "vx-theorem",
                "∴ invented, non-normative — the CLL defines what an attitudinal "
                em { "means" }
                ", never how it sounds. These vectors are voksa's own convention, which is exactly why your tuning matters. "
                span { class: "vx-mono", "cai / sai / ru'e" }
                " scale a vector; "
                span { class: "vx-mono", "nai" }
                " flips it."
            }
            div { class: "vx-md",
                div { class: "vx-emochips vx-mdrail",
                    for (k, a) in ATT_KINDS.iter().enumerate() {
                        EmotionChip { key: "{a.key}", kind: k }
                    }
                }
                EmotionEditor { key: "{sel}", kind: sel() }
            }
        }
    }
}

#[component]
fn EmotionChip(kind: usize) -> Element {
    let store = use_context::<ParamStore>();
    let mut ui = use_context::<Ui>();
    let a = &ATT_KINDS[kind];
    let selected = use_memo(move || *ui.sel_emotion.read() == kind);
    let mod_n = use_memo(move || {
        let range = store.desc().att_range(kind);
        store.dirty_count(range)
    });
    rsx! {
        button {
            class: "vx-emochip",
            class: if selected() { "vx-emochip-sel" },
            onclick: move |_| ui.sel_emotion.set(kind),
            if mod_n() > 0 {
                span { class: "vx-emodot" }
            }
            span { class: "vx-emolabel", "{a.label}" }
            span { class: "vx-emosub", "{a.sub}" }
        }
    }
}

#[component]
fn EmotionEditor(kind: usize) -> Element {
    let mut store = use_context::<ParamStore>();
    let mut ui = use_context::<Ui>();
    let audio = use_context::<Audio>();
    let flat: ReadSignal<bool> = use_memo(move || ui.flags.read().flat).into();
    let a = &ATT_KINDS[kind];
    let range = store.desc().att_range(kind);
    let example = a.example.to_string();
    let reset_range = range.clone();
    rsx! {
        div { class: "vx-scopeeditor",
            div { class: "vx-scopehead",
                span { class: "vx-scopelabel", "{a.label}" }
                span { class: "vx-scopesub", "{a.sub}" }
                HelpDot { topic: "att.emotion.{a.key}", title: a.label.to_string() }
                span { class: "vx-scopespacer" }
                button {
                    class: "vx-try",
                    disabled: flat(),
                    onclick: move |_| {
                        ui.text.set(example.clone());
                        ui.sentence.set(String::new());
                        speak_now(store, ui, audio.clone(), true);
                    },
                    "▶ try example"
                }
                button {
                    class: "vx-reset-scope",
                    r#type: "button",
                    aria_label: "reset {a.label}",
                    onclick: move |_| {
                        let plan = {
                            let d = store.desc();
                            reset_plan(&d, reset_range.clone())
                        };
                        store.apply(&plan);
                    },
                    "↺"
                }
            }
            for i in range.clone() {
                ParamRow { key: "{i}", idx: i, disabled: flat }
            }
        }
    }
}

// ─── rack D: voice table ──────────────────────────────────────────────────

/// The voice-table rack: changed-only switch, table reset, phoneme grid, one
/// per-phoneme editor.
#[component]
pub fn VoiceTableSection() -> Element {
    let mut store = use_context::<ParamStore>();
    let ui = use_context::<Ui>();
    let voice_dirty = use_memo(move || store.dirty_count(63..440));
    let mut changed = ui.vt_changed_only;
    let sel = ui.sel_phoneme;
    rsx! {
        section { class: "vx-card vx-rack", id: "voicetable",
            div { class: "vx-cardhead",
                span { class: "vx-slash", "// " }
                "voice table"
                span { class: "vx-racksub", "lo voksa selci — every phoneme's acoustics" }
                if voice_dirty() > 0 {
                    span { class: "vx-chip vx-chip-ember", "Δ {voice_dirty()}" }
                }
                HelpDot { topic: "vt._section", title: "voice table" }
            }
            div { class: "vx-vttools",
                label { class: "vx-switch",
                    input {
                        r#type: "checkbox",
                        checked: changed(),
                        onchange: move |e| changed.set(e.checked()),
                    }
                    span { "changed only" }
                }
                HelpDot { topic: "vt.changedonly", title: "changed only" }
                span { class: "vx-scopespacer" }
                button {
                    class: "vx-resetall",
                    onclick: move |_| {
                        let plan = {
                            let d = store.desc();
                            reset_plan(&d, d.voice_range())
                        };
                        store.apply(&plan);
                    },
                    "↺ reset table"
                }
            }
            div { class: "vx-md vx-md-vt",
                PhonemeGrid {}
                PhonemeEditor { key: "{sel}", item: sel() }
            }
        }
    }
}

#[component]
fn PhonemeGrid() -> Element {
    rsx! {
        div { class: "vx-phgrid",
            for (si, sec) in SECTIONS.iter().enumerate() {
                div { class: "vx-phsection",
                    div { class: "vx-phseclabel",
                        "{sec.label}"
                        if !sec.lojban.is_empty() {
                            span { class: "vx-phlojban", " · {sec.lojban}" }
                        }
                    }
                    div { class: "vx-phkeys",
                        for (item, vi) in VOICE_ITEMS.iter().enumerate() {
                            if vi.section as usize == si {
                                PhonemeKey { key: "{vi.key}", item }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PhonemeKey(item: usize) -> Element {
    let store = use_context::<ParamStore>();
    let mut ui = use_context::<Ui>();
    let vi = &VOICE_ITEMS[item];
    let selected = use_memo(move || *ui.sel_phoneme.read() == item);
    let mod_n = use_memo(move || {
        let range = store.desc().voice_item_range(item);
        store.dirty_count(range)
    });
    let changed_only = use_memo(move || *ui.vt_changed_only.read());
    let flat = use_memo(move || ui.flags.read().flat);
    let dimmed = changed_only() && mod_n() == 0 && !selected();
    rsx! {
        button {
            class: "vx-phkey",
            class: if selected() { "vx-phkey-sel" },
            class: if dimmed { "vx-phkey-dim" },
            class: if mod_n() > 0 { "vx-phkey-mod" },
            disabled: flat(),
            onclick: move |_| ui.sel_phoneme.set(item),
            if mod_n() > 0 {
                span { class: "vx-phdot" }
            }
            "{disp(vi)}"
        }
    }
}

#[component]
fn PhonemeEditor(item: usize) -> Element {
    let mut store = use_context::<ParamStore>();
    let ui = use_context::<Ui>();
    let flat: ReadSignal<bool> = use_memo(move || ui.flags.read().flat).into();
    let vi = &VOICE_ITEMS[item];
    let base = store.desc().voice_item_range(item).start;
    let note = item_note(item);
    let label = disp(vi).to_string();
    let class_key = format!("vt.class.{}", SECTIONS[vi.section as usize].id);
    rsx! {
        div { class: "vx-scopeeditor",
            div { class: "vx-scopehead",
                span { class: "vx-scopelabel", "{label}" }
                span { class: "vx-scopesub", "{vi.ipa}" }
                HelpDot { topic: class_key, title: label.clone() }
                span { class: "vx-scopespacer" }
                button {
                    class: "vx-reset-scope",
                    onclick: move |_| {
                        let plan = {
                            let d = store.desc();
                            reset_plan(&d, d.voice_item_range(item))
                        };
                        store.apply(&plan);
                    },
                    "↺ reset {label}"
                }
            }
            if let Some(n) = note {
                div { class: "vx-note", "∴ {n}" }
            }
            match vi.kind {
                ItemKind::Steady => rsx! {
                    div { class: "vx-matrix",
                        for slot in 0..9usize {
                            MatrixCell { key: "{base + slot}", idx: base + slot, disabled: flat }
                        }
                    }
                    ParamRow { idx: base + 9, disabled: flat }
                    ParamRow { idx: base + 10, disabled: flat }
                    ParamRow { idx: base + 11, disabled: flat }
                },
                ItemKind::Stop => rsx! {
                    div { class: "vx-matrixlabel", "closure — silence or voice bar" }
                    div { class: "vx-matrix",
                        for slot in 0..9usize {
                            MatrixCell { key: "{base + slot}", idx: base + slot, disabled: flat }
                        }
                    }
                    div { class: "vx-matrixlabel", "release burst" }
                    div { class: "vx-matrix",
                        for slot in 11..20usize {
                            MatrixCell { key: "{base + slot}", idx: base + slot, disabled: flat }
                        }
                    }
                    ParamRow { idx: base + 9, disabled: flat }
                    ParamRow { idx: base + 21, disabled: flat }
                    ParamRow { idx: base + 22, disabled: flat }
                    ParamRow { idx: base + 23, disabled: flat }
                },
                ItemKind::Dur => rsx! {
                    ParamRow { idx: base, disabled: flat }
                },
            }
        }
    }
}
