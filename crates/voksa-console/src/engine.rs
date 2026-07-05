//! Engine facade: direct calls into `voksa-web` (ADR 0003 — no second wasm,
//! no C-ABI marshalling) plus the status text the UI needs. Native-safe:
//! `voksa_web` is a plain rlib, so these compile and test off-wasm.

use voksa_core::compiler::CompileError;

use crate::model::Flags;

/// Render `text` (with the flag word + f32 param block) to mono f32 PCM.
pub fn render(
    text: &str,
    flags: Flags,
    params: &[f32],
    sample_rate: u32,
) -> Result<Vec<f32>, CompileError> {
    voksa_web::synth(text, flags.bits(), sample_rate, params)
}

/// The phonetic transcription of `text` under the flag word.
pub fn transcribe(text: &str, flags: Flags) -> Result<String, CompileError> {
    voksa_web::transcription(text, flags.bits())
}

/// A short human-readable reason for a [`CompileError`] — the status line and
/// error Callout show this. `CompileError` impls neither `Display` nor
/// `Error`, so the console owns its own wording.
pub fn describe(err: &CompileError) -> String {
    let _ = err;
    String::new() // stub — C3 green
}
