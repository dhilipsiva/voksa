//! Standalone runner: the console + vendored QUINE token sheets (a real host
//! page provides its own QUINE integration; this exists for `dx serve` and
//! the CI build gate).

use dioxus::prelude::*;
use voksa_console::TuningConsole;

const TOKENS: Asset = asset!("/assets/quine-tokens.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TOKENS }
        div { class: "vx-demo-page",
            TuningConsole {}
        }
    }
}
