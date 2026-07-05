//! The root component + embed contract (no site chrome; theme follows the
//! host; sticky offsets page-relative).

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use super::Ui;
use super::about::AboutPanel;
use super::help::HelpPopover;
use super::racks::TuningColumn;
use super::source::SourceColumn;
use super::speak::{Audio, use_auto_speak};
use super::store::ParamStore;
use super::title::TitleRow;
use crate::audio::AudioGraph;
use crate::model::Descriptors;

/// The stylesheet, bundled by the consumer's `dx build` via manganis.
pub const CONSOLE_CSS: Asset = asset!("/assets/console.css");

/// Props for [`TuningConsole`].
#[derive(Props, Clone, PartialEq)]
pub struct TuningConsoleProps {
    /// Initial `data-theme` ("dark"/"light"); `None` inherits the host page.
    #[props(default)]
    pub initial_theme: Option<String>,
    /// Append classes to the console root.
    #[props(default)]
    pub class: Option<String>,
    /// Inline the stylesheet instead of the manganis asset link (for
    /// consumers not building with dx).
    #[props(default)]
    pub inline_styles: bool,
}

/// The voksa tuning console. Mount once; it owns all of its state.
#[component]
pub fn TuningConsole(props: TuningConsoleProps) -> Element {
    // panic=abort is fatal-forever on wasm (ADR 0003): surface panics.
    #[cfg(target_arch = "wasm32")]
    use_hook(console_error_panic_hook::set_once);

    let store = use_hook(|| {
        let desc = Descriptors::from_defaults(&voksa_web::default_params())
            .expect("the engine's default block is the frozen 449-float layout");
        ParamStore::new(desc)
    });
    use_context_provider(|| store);
    let ui = use_hook(Ui::new);
    use_context_provider(|| ui);
    let audio = use_hook(|| Audio(Rc::new(RefCell::new(AudioGraph::new()))));
    use_context_provider(|| audio.clone());

    // Debounced auto-speak on any param/text/flag change.
    use_auto_speak(store, ui, audio);

    let theme_attr = props.initial_theme.clone().unwrap_or_default();
    let class = props.class.clone().unwrap_or_default();

    rsx! {
        if props.inline_styles {
            document::Style { {crate::assets::CONSOLE_CSS_SOURCE} }
        } else {
            document::Stylesheet { href: CONSOLE_CSS }
        }
        div {
            class: "vx-root {class}",
            "data-theme": if theme_attr.is_empty() { None } else { Some(theme_attr) },
            TitleRow {}
            div { class: "vx-main",
                SourceColumn {}
                TuningColumn {}
            }
            HelpPopover {}
            AboutPanel {}
        }
    }
}
