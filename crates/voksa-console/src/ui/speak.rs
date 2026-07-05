//! The speak pipeline: render on the main thread, drive the waveform + WAV
//! from the PCM, and play through the audio graph. Native-safe — `audio::play`
//! no-ops off-wasm, so the whole function compiles for `cargo check`.

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use super::Ui;
use super::store::ParamStore;
use crate::audio::{self, AudioGraph, PlayOutcome};
use crate::engine;

/// Playback / render status shown in the transport line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// The engine hasn't finished booting.
    Booting,
    /// Idle, ready to speak.
    Ready,
    /// Currently playing.
    Speaking,
    /// Rendered, but the audio context needs a ▶ speak gesture to unlock.
    NeedsGesture,
    /// A render error (already human-worded via `engine::describe`).
    Error(String),
}

/// The shared main-thread audio graph (provided via context; `Rc` is Clone).
#[derive(Clone)]
pub struct Audio(pub Rc<RefCell<AudioGraph>>);

/// Render the current utterance + params, update the waveform/WAV PCM, and
/// play. `gesture` = this call carries a user gesture (a ▶ speak / try-example
/// click) that unlocks the audio context; the debounced auto-speak passes
/// `false` (so a still-locked context correctly reports "tap ▶ speak").
pub fn speak_now(store: ParamStore, ui: Ui, audio: Audio, gesture: bool) {
    let text = ui.text.peek().clone();
    let flags = *ui.flags.peek();
    // The A/B latch is a render-time override: when hearing B, the nine
    // naturalness knobs render at identity without touching stored values.
    let params = crate::model::ab_effective(&store.desc(), &store.snapshot(), *ui.ab_off.peek());
    let mut status = ui.status;
    let mut pcm_sig = ui.pcm;
    match engine::render(&text, flags, &params, audio::SAMPLE_RATE) {
        Ok(pcm) => {
            let rc = Rc::new(pcm);
            pcm_sig.set(Some(rc.clone())); // waveform + WAV, no gesture needed
            let mut done_status = status;
            let outcome = audio::play(
                &audio.0,
                &rc,
                move || done_status.set(Status::Ready),
                gesture,
            );
            status.set(match outcome {
                PlayOutcome::Playing => Status::Speaking,
                PlayOutcome::NeedsGesture => Status::NeedsGesture,
                PlayOutcome::Unavailable => Status::Ready,
            });
        }
        Err(e) => status.set(Status::Error(engine::describe(&e))),
    }
}

/// Debounced auto-speak: on any parameter/text/flag change, wait 400 ms then
/// speak — but only if no newer change arrived (an epoch token collapses
/// slider-drag streams without needing to cancel the spawned timer).
pub fn use_auto_speak(store: ParamStore, ui: Ui, audio: Audio) {
    let mut epoch = use_signal(|| 0u64);
    use_effect(move || {
        // Subscribe to every mutation source.
        let _ = store.generation.read();
        let _ = ui.text.read();
        let _ = ui.flags.read();
        let _ = ui.ab_off.read();
        if !*ui.auto_speak.peek() {
            return;
        }
        let mine = *epoch.peek() + 1;
        epoch.set(mine);
        let audio = audio.clone();
        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(400).await;
            if *epoch.peek() == mine {
                speak_now(store, ui, audio, false);
            }
        });
    });
}
