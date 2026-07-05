//! The audio graph (ADR 0003): main-thread `synth()` renders PCM; a
//! player-only AudioWorklet (Blob-URL loaded, no served asset, no wasm
//! inside) plays it. Browser-only — a native stub keeps `cargo nextest`/
//! `check` green (the model tests never touch this).

use std::cell::RefCell;
use std::rc::Rc;

/// Whether playback started. The waveform/WAV never need audio (they read the
/// main-thread render directly); only playback needs the unlocked context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayOutcome {
    /// The buffer is playing.
    Playing,
    /// The context is suspended — needs a user gesture (a ▶ speak click).
    NeedsGesture,
    /// Audio is unavailable in this environment (native build).
    Unavailable,
}

#[cfg(target_arch = "wasm32")]
mod graph;

/// Play `pcm`, replacing any currently-playing node; `on_done` fires when it
/// drains. `gesture` = the call carries a user gesture that unlocks the audio
/// context (a ▶ speak / try-example click); a non-gesture play (auto-speak)
/// reports `NeedsGesture` while the context is still suspended.
pub fn play(
    graph: &Rc<RefCell<AudioGraph>>,
    pcm: &[f32],
    on_done: impl FnMut() + 'static,
    gesture: bool,
) -> PlayOutcome {
    #[cfg(target_arch = "wasm32")]
    {
        AudioGraph::play(graph, pcm, on_done, gesture)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (graph, pcm, on_done, gesture);
        PlayOutcome::Unavailable
    }
}

/// Trigger a browser download (no-op natively).
pub fn download(filename: &str, bytes: &[u8], mime: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = graph::download(filename, bytes, mime);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (filename, bytes, mime);
    }
}

/// The engine render rate the graph requests (fixed 48 kHz — ADR 0003).
pub const SAMPLE_RATE: u32 = 48_000;

#[cfg(target_arch = "wasm32")]
pub use graph::AudioGraph;

/// Inert native stand-in so the crate compiles and unit-tests off-wasm.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct AudioGraph;

#[cfg(not(target_arch = "wasm32"))]
impl AudioGraph {
    /// A fresh, unstarted graph (native no-op).
    pub fn new() -> Self {
        AudioGraph
    }
}
