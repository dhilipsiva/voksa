//! ParamRow — the ×465 primitive. Grid `148px | 1fr | 96px`; a transparent
//! native `input[type=range]` overlays the custom track/thumb so drag,
//! arrow keys, focus, and screen-reader semantics come for free. Four states:
//! default / modified (ember delta-fill from the default tick) / widened ⤢ /
//! disabled. The readout is tap-to-type (Enter/blur commits and may widen;
//! Esc reverts). Rows take ONLY their flat index — values come from context,
//! so parent re-renders never cascade into the 465 rows.

use dioxus::prelude::*;

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

/// One tunable parameter row (`compact` = MatrixCell geometry).
#[component]
pub fn ParamRow(idx: usize, disabled: ReadSignal<bool>) -> Element {
    let mut store = use_context::<ParamStore>();
    let cell = store.cell(idx);
    let (label, unit, dmin, dmax, dstep, default) = {
        let desc = store.desc();
        let d = desc.get(idx);
        (d.label.clone(), d.unit, d.min, d.max, d.step, d.default)
    };
    let c = cell();
    let (min, max, step) = match c.widened {
        Some(w) => (w.min, w.max, String::from("any")),
        None => (dmin, dmax, nice(dstep)),
    };
    let dirty = {
        let desc = store.desc();
        model::is_dirty(desc.get(idx), c.value)
    };
    let vpct = pct(c.value, min, max);
    let tpct = pct(default, min, max);
    let (lo, hi) = if vpct >= tpct {
        (tpct, vpct)
    } else {
        (vpct, tpct)
    };
    let is_disabled = disabled();

    // Tap-to-type draft: local, dies with the row; never touches the store
    // until commit.
    let mut draft = use_signal(|| None::<String>);

    rsx! {
        div {
            class: "vx-srow",
            class: if dirty { "vx-mod" },
            class: if is_disabled { "vx-off" },
            style: "--vx-fill-lo:{lo}%;--vx-fill-hi:{hi}%;--vx-tick:{tpct}%;--vx-thumb:{vpct}%",
            div { class: "vx-slabel",
                span { class: "vx-dot" }
                span { class: "vx-name", "{label}" }
                if !unit.is_empty() {
                    span { class: "vx-unit", "{unit}" }
                }
            }
            div { class: "vx-slider",
                div { class: "vx-track" }
                if dirty {
                    div { class: "vx-fill" }
                }
                div { class: "vx-tickmark" }
                div { class: "vx-thumb" }
                input {
                    class: "vx-range",
                    r#type: "range",
                    min: "{min}",
                    max: "{max}",
                    step: "{step}",
                    value: "{c.value}",
                    disabled: is_disabled,
                    aria_label: "{label}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<f32>() {
                            store.set(idx, v);
                        }
                    },
                    ondoubleclick: move |_| store.reset(idx),
                }
            }
            div { class: "vx-readout",
                if dirty && !is_disabled {
                    button {
                        class: "vx-reset",
                        tabindex: "-1",
                        aria_label: "reset {label}",
                        onclick: move |_| store.reset(idx),
                        "↺"
                    }
                }
                if c.widened.is_some() {
                    span { class: "vx-widened", title: "range widened", "⤢" }
                }
                if let Some(text) = draft() {
                    input {
                        class: "vx-value vx-value-edit",
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
                                Key::Escape => {
                                    draft.set(None);
                                }
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
                        class: "vx-value",
                        disabled: is_disabled,
                        onclick: move |_| draft.set(Some(nice(cell.peek().value))),
                        {nice(c.value)}
                    }
                }
            }
        }
    }
}
