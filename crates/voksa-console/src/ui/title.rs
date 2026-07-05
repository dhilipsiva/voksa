//! The non-sticky page-title row (embed contract: the HOST owns the navbar;
//! this row scrolls with the page).

use dioxus::prelude::*;

use super::store::ParamStore;
use crate::model::PARAM_TOTAL;

/// Title row: wordmark + label + version chip + live Δ chip.
#[component]
pub fn TitleRow() -> Element {
    let store = use_context::<ParamStore>();
    let dirty = use_memo(move || store.dirty_count(0..PARAM_TOTAL));
    rsx! {
        header { class: "vx-titlerow",
            span { class: "vx-wordmark", "voksa" }
            span { class: "vx-label", "tuning console" }
            span { class: "vx-chip", "v{env!(\"CARGO_PKG_VERSION\")}" }
            if dirty() > 0 {
                span { class: "vx-chip vx-chip-ember", "Δ {dirty()}" }
            }
        }
    }
}
