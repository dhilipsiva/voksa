//! ParamRow — the ×465 primitive — and MatrixCell, its compact label-above
//! variant for the 3×3 formant matrices. A transparent native
//! `input[type=range]` overlays the custom track/thumb so drag, arrow keys,
//! focus, and screen-reader semantics come for free. Four states: default /
//! modified (ember delta-fill from the default tick) / widened ⤢ / disabled.
//! The readout is tap-to-type (Enter/blur commits and may widen; Esc reverts).
//! Rows take ONLY their flat index — values come from context, so parent
//! re-renders never cascade into the rows.

use dioxus::prelude::*;

use super::help::HelpDot;
use super::store::ParamStore;
use crate::model;

/// Where a value sits inside a (possibly widened) range, as a 0..100 percent.
fn pct(v: f32, min: f32, max: f32) -> f32 {
    if max > min {
        ((v - min) / (max - min) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// Shortest decimal readout that round-trips the f32 (the reference page's
/// `f32nice`): trims float noise without lying about the value.
pub fn nice(v: f32) -> String {
    for p in 1..=9 {
        let s = format!("{v:.*}", p);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        if s.parse::<f32>().map(|r| r == v).unwrap_or(false) {
            return s.to_string();
        }
    }
    v.to_string()
}

/// The reactive read of one cell + its descriptor — shared by both row layouts
/// so they can never drift.
struct CellView {
    value: f32,
    label: String,
    help_key: String,
    unit: &'static str,
    min: f32,
    max: f32,
    step: String,
    dirty: bool,
    widened: bool,
    lo: f32,
    hi: f32,
    tpct: f32,
    vpct: f32,
}

fn cell_view(store: &ParamStore, idx: usize) -> CellView {
    let c = store.cell(idx)();
    let desc = store.desc();
    let d = desc.get(idx);
    let (min, max, step) = match c.widened {
        Some(w) => (w.min, w.max, String::from("any")),
        None => (d.min, d.max, nice(d.step)),
    };
    let vpct = pct(c.value, min, max);
    let tpct = pct(d.default, min, max);
    let (lo, hi) = if vpct >= tpct {
        (tpct, vpct)
    } else {
        (vpct, tpct)
    };
    CellView {
        value: c.value,
        label: d.label.clone(),
        help_key: d.help_key.clone(),
        unit: d.unit,
        min,
        max,
        step,
        dirty: model::is_dirty(d, c.value),
        widened: c.widened.is_some(),
        lo,
        hi,
        tpct,
        vpct,
    }
}

/// The tap-to-type value control (Enter/blur commits via `store.set` — may
/// widen; Esc reverts). Own draft signal, so it dies with the cell.
#[component]
fn ValueEditor(idx: usize, disabled: bool, class: String) -> Element {
    let mut store = use_context::<ParamStore>();
    let cell = store.cell(idx);
    let value = cell().value;
    let mut draft = use_signal(|| None::<String>);
    rsx! {
        if let Some(text) = draft() {
            input {
                class: "{class} vx-value-edit",
                r#type: "text",
                inputmode: "decimal",
                value: "{text}",
                autofocus: true,
                oninput: move |e| draft.set(Some(e.value())),
                onkeydown: move |e| {
                    match e.key() {
                        Key::Enter => {
                            if let Some(t) = draft.take()
                                && let Ok(v) = t.trim().parse::<f32>()
                            {
                                store.set(idx, v);
                            }
                        }
                        Key::Escape => draft.set(None),
                        _ => {}
                    }
                },
                onblur: move |_| {
                    if let Some(t) = draft.take()
                        && let Ok(v) = t.trim().parse::<f32>()
                    {
                        store.set(idx, v);
                    }
                },
            }
        } else {
            button {
                class: "{class}",
                disabled,
                onclick: move |_| draft.set(Some(nice(cell.peek().value))),
                {nice(value)}
            }
        }
    }
}

/// The custom slider visuals + the transparent native range overlay.
#[component]
fn Slider(
    idx: usize,
    view_min: f32,
    view_max: f32,
    view_step: String,
    dirty: bool,
    disabled: bool,
    label: String,
    value: f32,
    extra: String,
) -> Element {
    let mut store = use_context::<ParamStore>();
    rsx! {
        div { class: "vx-slider {extra}",
            div { class: "vx-track" }
            if dirty {
                div { class: "vx-fill" }
            }
            div { class: "vx-tickmark" }
            div { class: "vx-thumb" }
            input {
                class: "vx-range",
                r#type: "range",
                min: "{view_min}",
                max: "{view_max}",
                step: "{view_step}",
                value: "{value}",
                disabled,
                aria_label: "{label}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<f32>() {
                        store.set(idx, v);
                    }
                },
                ondoubleclick: move |_| store.reset(idx),
            }
        }
    }
}

/// One full-width tunable parameter row.
#[component]
pub fn ParamRow(idx: usize, disabled: ReadSignal<bool>) -> Element {
    let mut store = use_context::<ParamStore>();
    let v = cell_view(&store, idx);
    let is_disabled = disabled();
    rsx! {
        div {
            class: "vx-srow",
            class: if v.dirty { "vx-mod" },
            class: if is_disabled { "vx-off" },
            style: "--vx-fill-lo:{v.lo}%;--vx-fill-hi:{v.hi}%;--vx-tick:{v.tpct}%;--vx-thumb:{v.vpct}%",
            div { class: "vx-slabel",
                span { class: "vx-dot" }
                span { class: "vx-name", "{v.label}" }
                if !v.unit.is_empty() {
                    span { class: "vx-unit", "{v.unit}" }
                }
                HelpDot { topic: v.help_key.clone(), title: v.label.clone() }
            }
            Slider {
                idx,
                view_min: v.min,
                view_max: v.max,
                view_step: v.step,
                dirty: v.dirty,
                disabled: is_disabled,
                label: v.label.clone(),
                value: v.value,
                extra: String::new(),
            }
            div { class: "vx-readout",
                if v.dirty && !is_disabled {
                    button {
                        class: "vx-reset",
                        tabindex: "-1",
                        aria_label: "reset {v.label}",
                        onclick: move |_| store.reset(idx),
                        "↺"
                    }
                }
                if v.widened {
                    span { class: "vx-widened", title: "range widened", "⤢" }
                }
                ValueEditor { idx, disabled: is_disabled, class: "vx-value" }
            }
        }
    }
}

/// The compact label-above matrix cell (3×3 formant grid). Same store wiring,
/// 11 px thumb, ember-mix border when modified.
#[component]
pub fn MatrixCell(idx: usize, disabled: ReadSignal<bool>) -> Element {
    let store = use_context::<ParamStore>();
    let v = cell_view(&store, idx);
    let is_disabled = disabled();
    // The matrix already carries a "closure"/"burst" header, so the cell shows
    // just the field name (F1, F1 bw, …).
    let short = v
        .label
        .strip_prefix("closure ")
        .or_else(|| v.label.strip_prefix("burst "))
        .unwrap_or(&v.label)
        .to_string();
    rsx! {
        div {
            class: "vx-mcell",
            class: if v.dirty { "vx-mod" },
            class: if is_disabled { "vx-off" },
            style: "--vx-fill-lo:{v.lo}%;--vx-fill-hi:{v.hi}%;--vx-tick:{v.tpct}%;--vx-thumb:{v.vpct}%",
            title: "{v.label}",
            div { class: "vx-mtop",
                span { class: "vx-mlabel", "{short}" }
                if v.widened {
                    span { class: "vx-widened", title: "range widened", "⤢" }
                }
            }
            Slider {
                idx,
                view_min: v.min,
                view_max: v.max,
                view_step: v.step,
                dirty: v.dirty,
                disabled: is_disabled,
                label: v.label.clone(),
                value: v.value,
                extra: "vx-mslider".to_string(),
            }
            ValueEditor { idx, disabled: is_disabled, class: "vx-mvalue" }
        }
    }
}
