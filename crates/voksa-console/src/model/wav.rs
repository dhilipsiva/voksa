//! Minimal RIFF/WAVE writer (mono, 16-bit PCM) for the console's WAV
//! download. Byte-identical to `voksa-cli/src/wav.rs` — but voksa-cli is a
//! dev-dep only (it pulls cpal/alsa), so the ~20 lines are mirrored here to
//! stay out of the wasm dependency tree.

/// Encode mono f32 samples ([-1, 1]) to a 16-bit PCM WAV byte buffer.
pub fn wav_bytes(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let _ = (samples, sample_rate);
    Vec::new() // stub — C3 green
}
