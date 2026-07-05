//! Browser audio graph: an `AudioContext` requested at 48 kHz (matches the
//! engine's render rate → no resampling / pitch shift), a Blob-URL-loaded
//! player worklet, and node replacement on each utterance.

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Array, Float32Array, Object, Reflect};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    AudioContext, AudioContextOptions, AudioContextState, AudioWorkletNode,
    AudioWorkletNodeOptions, Blob, BlobPropertyBag, MessageEvent, Url,
};

use super::PlayOutcome;

/// The engine's render rate; the context is requested at the same rate.
const SAMPLE_RATE: u32 = 48_000;
const PLAYER_JS: &str = include_str!("player.js");

/// Owns the `AudioContext`, the current player node, and its `done` closure.
/// Lives in a root `use_hook` `Rc<RefCell<…>>` — it is NOT render state.
#[derive(Default)]
pub struct AudioGraph {
    ctx: Option<AudioContext>,
    module_loaded: bool,
    node: Option<AudioWorkletNode>,
    _on_done: Option<Closure<dyn FnMut(MessageEvent)>>,
}

impl AudioGraph {
    /// The render sample rate (fixed; the context is requested to match).
    pub const SAMPLE_RATE: u32 = SAMPLE_RATE;

    /// A fresh, unstarted graph.
    pub fn new() -> Self {
        Self::default()
    }

    fn ensure_ctx(&mut self) -> Result<AudioContext, JsValue> {
        if let Some(ctx) = &self.ctx {
            return Ok(ctx.clone());
        }
        let opts = AudioContextOptions::new();
        opts.set_sample_rate(SAMPLE_RATE as f32);
        let ctx = AudioContext::new_with_context_options(&opts)?;
        self.ctx = Some(ctx.clone());
        Ok(ctx)
    }

    /// Play `pcm`, replacing any currently-playing node. Must be called from
    /// a user-gesture handler the first time (to unlock the context); returns
    /// [`PlayOutcome::NeedsGesture`] while the context is still suspended.
    /// `on_done` fires when playback drains (status → ready).
    pub fn play(
        self_rc: &Rc<RefCell<AudioGraph>>,
        pcm: &[f32],
        on_done: impl FnMut() + 'static,
    ) -> PlayOutcome {
        match Self::try_play(self_rc, pcm, on_done) {
            Ok(outcome) => outcome,
            Err(_) => PlayOutcome::Unavailable,
        }
    }

    fn try_play(
        self_rc: &Rc<RefCell<AudioGraph>>,
        pcm: &[f32],
        on_done: impl FnMut() + 'static,
    ) -> Result<PlayOutcome, JsValue> {
        let ctx = self_rc.borrow_mut().ensure_ctx()?;

        // Load the worklet module once, via a Blob URL (no served asset).
        if !self_rc.borrow().module_loaded {
            let parts = Array::new();
            parts.push(&JsValue::from_str(PLAYER_JS));
            let bag = BlobPropertyBag::new();
            bag.set_type("text/javascript");
            let blob = Blob::new_with_str_sequence_and_options(&parts, &bag)?;
            let url = Url::create_object_url_with_blob(&blob)?;
            let promise = ctx.audio_worklet()?.add_module(&url)?;
            let self_rc2 = self_rc.clone();
            let pcm_owned: Vec<f32> = pcm.to_vec();
            let on_done_cell = Rc::new(RefCell::new(Some(
                Box::new(on_done) as Box<dyn FnMut() + 'static>
            )));
            spawn_local(async move {
                let _ = JsFuture::from(promise).await;
                let _ = Url::revoke_object_url(&url);
                self_rc2.borrow_mut().module_loaded = true;
                // Now that the module exists, start playback.
                let mut taken = on_done_cell.borrow_mut().take();
                let cb = move || {
                    if let Some(f) = taken.as_mut() {
                        f();
                    }
                };
                let _ = Self::start(&self_rc2, &pcm_owned, cb);
            });
            // The context may still be suspended (no gesture yet).
            return Ok(Self::resume_state(&ctx));
        }

        Self::start(self_rc, pcm, on_done)?;
        Ok(Self::resume_state(&ctx))
    }

    /// Instantiate a fresh player node for `pcm`, disconnecting the previous
    /// one (replace-never-stack), and wire its `done` message.
    fn start(
        self_rc: &Rc<RefCell<AudioGraph>>,
        pcm: &[f32],
        mut on_done: impl FnMut() + 'static,
    ) -> Result<(), JsValue> {
        let ctx = self_rc.borrow_mut().ensure_ctx()?;
        let _ = ctx.resume(); // unlock if this call carries a gesture

        // Disconnect the outgoing node.
        if let Some(old) = self_rc.borrow_mut().node.take() {
            let _ = old.disconnect();
        }

        let opts = AudioWorkletNodeOptions::new();
        opts.set_number_of_inputs(0);
        opts.set_number_of_outputs(1);
        let out_channels = Array::new();
        out_channels.push(&JsValue::from_f64(1.0));
        opts.set_output_channel_count(&out_channels);

        // processorOptions carries the PCM (structure-cloned into the worklet;
        // a few hundred KB per utterance — a copy is fine).
        let pcm_arr = Float32Array::from(pcm);
        let proc_opts = Object::new();
        Reflect::set(&proc_opts, &JsValue::from_str("pcm"), &pcm_arr)?;
        opts.set_processor_options(Some(&proc_opts));

        let node = AudioWorkletNode::new_with_options(&ctx, "voksa-player", &opts)?;
        node.connect_with_audio_node(&ctx.destination())?;

        let done = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Some(obj) = e.data().dyn_ref::<Object>()
                && Reflect::get(obj, &JsValue::from_str("type"))
                    .ok()
                    .and_then(|v| v.as_string())
                    .as_deref()
                    == Some("done")
            {
                on_done();
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        node.port()?
            .set_onmessage(Some(done.as_ref().unchecked_ref()));

        let mut g = self_rc.borrow_mut();
        g.node = Some(node);
        g._on_done = Some(done);
        Ok(())
    }

    fn resume_state(ctx: &AudioContext) -> PlayOutcome {
        let _ = ctx.resume();
        if ctx.state() == AudioContextState::Suspended {
            PlayOutcome::NeedsGesture
        } else {
            PlayOutcome::Playing
        }
    }
}

/// Trigger a browser download of `bytes` as `filename` (mime `application/*`).
pub fn download(filename: &str, bytes: &[u8], mime: &str) -> Result<(), JsValue> {
    let arr = js_sys::Uint8Array::from(bytes);
    let parts = Array::new();
    parts.push(&arr.buffer());
    let bag = BlobPropertyBag::new();
    bag.set_type(mime);
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &bag)?;
    let url = Url::create_object_url_with_blob(&blob)?;
    let doc = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let a = doc
        .create_element("a")?
        .dyn_into::<web_sys::HtmlAnchorElement>()?;
    a.set_href(&url);
    a.set_download(filename);
    a.click();
    Url::revoke_object_url(&url)?;
    Ok(())
}
