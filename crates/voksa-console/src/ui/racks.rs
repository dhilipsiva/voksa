//! The tuning column: sticky rack nav (anchor chips + preset + reset-all)
//! and the knob racks. C2 ships racks A (prosody) and B (naturalness);
//! attitudinals and the voice table land in C4.

use dioxus::prelude::*;

use super::Ui;
use super::editors::{AttitudinalSection, VoiceTableSection};
use super::help::HelpDot;
use super::rows::ParamRow;
use super::store::ParamStore;
use crate::model::{PARAM_TOTAL, PRESETS, apply_preset, reset_plan};

/// The scrolling tuning column.
#[component]
pub fn TuningColumn() -> Element {
    let store = use_context::<ParamStore>();
    let ui = use_context::<Ui>();
    let flat: ReadSignal<bool> = use_memo(move || ui.flags.read().flat).into();
    let ab_off = ui.ab_off;
    // Naturalness rows are inert under flat OR while hearing the B (off) arm.
    let nat_disabled: ReadSignal<bool> = use_memo(move || ui.flags.read().flat || ab_off()).into();
    let prosody_dirty = use_memo(move || store.dirty_count(0..7));
    let nat_dirty = use_memo(move || store.dirty_count(440..PARAM_TOTAL));

    rsx! {
        div { class: "vx-right",
            RackNav {}
            Rack {
                id: "prosody",
                title: "prosody",
                sub: "declination · stress · rate",
                help: "prosody._section",
                dirty: prosody_dirty,
                flat,
                for k in 0..7usize {
                    ParamRow { key: "{k}", idx: k, disabled: flat }
                }
            }
            Rack {
                id: "naturalness",
                title: "naturalness",
                sub: "the micro-variation that softens the robot",
                help: "naturalness._section",
                dirty: nat_dirty,
                flat,
                AbLatch {}
                for k in 440..PARAM_TOTAL {
                    ParamRow { key: "{k}", idx: k, disabled: nat_disabled }
                }
            }
            AttitudinalSection {}
            VoiceTableSection {}
        }
    }
}

/// The A/B listening latch: A = current tuning, B = the phase-10 voice (nine
/// naturalness knobs at identity) — a render-time override, values preserved.
#[component]
fn AbLatch() -> Element {
    let ui = use_context::<Ui>();
    let mut ab = ui.ab_off;
    rsx! {
        div { class: "vx-ab",
            div { class: "vx-abrow",
                div { class: "vx-abseg",
                    button {
                        class: "vx-abbtn vx-abbtn-a",
                        class: if !ab() { "vx-abbtn-sel" },
                        onclick: move |_| ab.set(false),
                        "A · current"
                    }
                    button {
                        class: "vx-abbtn vx-abbtn-b",
                        class: if ab() { "vx-abbtn-sel" },
                        onclick: move |_| ab.set(true),
                        "B · naturalness off"
                    }
                }
                HelpDot { topic: "naturalness.ab", title: "A/B latch" }
            }
            if ab() {
                div { class: "vx-abbanner",
                    "▶ hearing B — the phase-10 voice (all nine at identity). Sliders paused; your values are kept."
                }
            }
        }
    }
}

/// Sticky nav: anchor chips with live Δ counts, preset select, reset all.
#[component]
fn RackNav() -> Element {
    let mut store = use_context::<ParamStore>();
    let ui = use_context::<Ui>();
    let mut preset = ui.preset;
    rsx! {
        nav { class: "vx-racknav",
            a { class: "vx-anchor", href: "#prosody", "A" }
            a { class: "vx-anchor", href: "#naturalness", "B" }
            a { class: "vx-anchor", href: "#attitudinals", "C" }
            a { class: "vx-anchor", href: "#voicetable", "D" }
            span { class: "vx-navspacer" }
            select {
                class: "vx-select vx-preset",
                aria_label: "preset",
                onchange: move |e| {
                    let name = e.value();
                    let plan = {
                        let desc = store.desc();
                        apply_preset(&desc, &name)
                    };
                    if let Some(plan) = plan {
                        store.apply(&plan);
                        preset.set(name);
                    }
                },
                for p in PRESETS.iter() {
                    option { value: "{p.name}", selected: *preset.read() == p.name, "{p.name}" }
                }
            }
            HelpDot { topic: "presets", title: "presets" }
            button {
                class: "vx-resetall",
                onclick: move |_| {
                    let plan = {
                        let desc = store.desc();
                        reset_plan(&desc, 0..PARAM_TOTAL)
                    };
                    store.apply(&plan);
                    preset.set("Default".to_string());
                },
                "↺ reset all"
            }
            HelpDot { topic: "reset.all", title: "reset all" }
        }
    }
}

/// A rack card: header (`// name` + Δ chip) over its rows.
#[component]
fn Rack(
    id: &'static str,
    title: &'static str,
    sub: &'static str,
    help: &'static str,
    dirty: ReadSignal<usize>,
    flat: ReadSignal<bool>,
    children: Element,
) -> Element {
    rsx! {
        section {
            class: "vx-card vx-rack",
            class: if flat() { "vx-rack-flat" },
            id: "{id}",
            div { class: "vx-cardhead",
                span { class: "vx-slash", "// " }
                "{title}"
                span { class: "vx-racksub", "{sub}" }
                if flat() {
                    span { class: "vx-chip vx-chip-amber", "flat — disabled" }
                } else if dirty() > 0 {
                    span { class: "vx-chip vx-chip-ember", "Δ {dirty()}" }
                }
                HelpDot { topic: help.to_string(), title: title.to_string() }
            }
            div { class: "vx-rackbody", {children} }
        }
    }
}
