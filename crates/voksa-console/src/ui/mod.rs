//! The Dioxus component layer — a thin projection over [`crate::model`].
//! Look and structure follow the QUINE design handoff
//! (`docs/design/tuning-console/`, Workbench = reference implementation).

mod about;
mod console;
mod editors;
mod help;
mod racks;
mod rows;
mod source;
mod speak;
mod store;
mod title;

pub use console::{TuningConsole, TuningConsoleProps};
pub use speak::{Audio, Status};
pub use store::{Cell, ParamStore};

use std::rc::Rc;

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
    /// Auto-speak on change (default ON).
    pub auto_speak: Signal<bool>,
    /// Playback / render status.
    pub status: Signal<Status>,
    /// The last rendered PCM (drives the waveform + WAV download).
    pub pcm: Signal<Option<Rc<Vec<f32>>>>,
    /// Selected voice-table item (index into `VOICE_ITEMS`; 0 = /a/).
    pub sel_phoneme: Signal<usize>,
    /// Selected attitudinal (index into `ATT_KINDS`; 0 = .ui).
    pub sel_emotion: Signal<usize>,
    /// Voice-table "changed only" view (dims untouched keycaps).
    pub vt_changed_only: Signal<bool>,
    /// Naturalness A/B latch: `true` renders with the nine knobs at identity
    /// (the phase-10 voice) without touching stored values.
    pub ab_off: Signal<bool>,
    /// The open help popover request (`None` = closed).
    pub help: Signal<Option<help::Help>>,
    /// Whether the "about voksa" modal is open.
    pub about_open: Signal<bool>,
}

impl Ui {
    fn new() -> Self {
        Ui {
            text: Signal::new("coi munje".to_string()),
            flags: Signal::new(Flags::default()),
            preset: Signal::new("Default".to_string()),
            sentence: Signal::new(String::new()),
            notes: Signal::new(String::new()),
            auto_speak: Signal::new(true),
            status: Signal::new(Status::Ready),
            pcm: Signal::new(None),
            sel_phoneme: Signal::new(0),
            sel_emotion: Signal::new(0),
            vt_changed_only: Signal::new(false),
            ab_off: Signal::new(false),
            help: Signal::new(None),
            about_open: Signal::new(false),
        }
    }
}
