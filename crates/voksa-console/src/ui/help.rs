//! The `?` help affordance: a small dot per control that opens a fixed
//! popover near the click, resolving its copy from [`crate::help`]. One
//! [`HelpPopover`] is mounted at the root; every [`HelpDot`] just writes the
//! open request into `Ui.help`. Dots are always SIBLINGS of the control they
//! document (never nested inside a `<button>`), so they never make invalid
//! nested-interactive markup.

use dioxus::prelude::*;

use super::Ui;

/// An open help request: which registry key, a display title, and the
/// clamped viewport position to render the popover at.
#[derive(Debug, Clone, PartialEq)]
pub struct Help {
    /// The `crate::help` registry key to resolve.
    pub key: String,
    /// Human title shown in the popover head.
    pub title: String,
    /// Fixed-position left, px.
    pub x: f64,
    /// Fixed-position top, px.
    pub y: f64,
}

/// The viewport size, for clamping the popover on-screen (native default when
/// off-wasm — the popover only ever renders in the browser).
fn viewport() -> (f64, f64) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            let iw = w.inner_width().ok().and_then(|v| v.as_f64());
            let ih = w.inner_height().ok().and_then(|v| v.as_f64());
            if let (Some(iw), Some(ih)) = (iw, ih) {
                return (iw, ih);
            }
        }
    }
    (1280.0, 800.0)
}

/// A `?` dot that opens the help popover for the registry key `topic`
/// (`key` is a reserved Dioxus prop name). `title` heads the popover.
#[component]
pub fn HelpDot(topic: String, title: String) -> Element {
    let mut ui = use_context::<Ui>();
    let topic_c = topic.clone();
    let title_c = title.clone();
    rsx! {
        button {
            class: "vx-help",
            r#type: "button",
            tabindex: "-1",
            aria_label: "help: {title}",
            title: "help: {title}",
            onclick: move |e| {
                e.stop_propagation();
                let p = e.client_coordinates();
                let (vw, vh) = viewport();
                // Reference positioning: left of the dot, below it, clamped.
                let x = (p.x - 30.0).clamp(8.0, (vw - 300.0).max(8.0));
                let y = (p.y + 8.0).min((vh - 170.0).max(0.0));
                ui.help
                    .set(Some(Help { key: topic_c.clone(), title: title_c.clone(), x, y }));
            },
            "?"
        }
    }
}

/// The single root-mounted popover. Renders nothing until a dot is clicked;
/// dismissed by the ✕, a click on the backdrop, or opening another dot.
#[component]
pub fn HelpPopover() -> Element {
    let mut ui = use_context::<Ui>();
    let Some(h) = ui.help.read().clone() else {
        return rsx! {};
    };
    let body = crate::help::help_for(&h.key);
    rsx! {
        div {
            class: "vx-pop-backdrop",
            onclick: move |_| ui.help.set(None),
        }
        div {
            class: "vx-pop",
            style: "left:{h.x}px;top:{h.y}px",
            onclick: move |e| e.stop_propagation(),
            div { class: "vx-pop-head",
                span { class: "vx-pop-title", "{h.title}" }
                span { class: "vx-pop-spacer" }
                button {
                    class: "vx-pop-x",
                    r#type: "button",
                    aria_label: "close help",
                    onclick: move |_| ui.help.set(None),
                    "✕"
                }
            }
            div { class: "vx-pop-body", "{body}" }
            div { class: "vx-pop-key", "key: {h.key} · voksa-help-text.json" }
        }
    }
}
