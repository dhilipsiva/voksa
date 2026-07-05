//! The Dioxus component layer — a thin projection over [`crate::model`].
//! Look and structure follow the QUINE design handoff
//! (`docs/design/tuning-console/`, Workbench = reference implementation).

mod console;
mod racks;
mod rows;
mod source;
mod store;
mod title;

pub use console::{TuningConsole, TuningConsoleProps};
pub use store::{Cell, ParamStore};

use dioxus::prelude::*;

use crate::model::Flags;

/// Console-wide UI state (small independent signals, provided via context).
#[derive(Clone, Copy)]
pub struct Ui {
    /// The utterance text.
    pub text: Signal<String>,
    /// The compile/prosody flags.
    pub flags: Signal<Flags>,
    /// Selected preset name ("Custom" once anything is touched).
    pub preset: Signal<String>,
    /// Selected sentence slug (empty = free text).
    pub sentence: Signal<String>,
    /// The tuner's notes (travel inside the export).
    pub notes: Signal<String>,
}

impl Ui {
    fn new() -> Self {
        Ui {
            text: Signal::new("coi munje".to_string()),
            flags: Signal::new(Flags::default()),
            preset: Signal::new("Default".to_string()),
            sentence: Signal::new(String::new()),
            notes: Signal::new(String::new()),
        }
    }
}
