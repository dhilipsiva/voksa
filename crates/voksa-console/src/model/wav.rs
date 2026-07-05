//! Minimal RIFF/WAVE writer (mono, 16-bit PCM) for the console's WAV
//! download. Byte-identical to `voksa-cli/src/wav.rs` — but voksa-cli is a
//! dev-dep only (it pulls cpal/alsa), so the ~20 lines are mirrored here to
//! stay out of the wasm dependency tree.

/// Encode mono f32 samples ([-1, 1]) to a 16-bit PCM WAV byte buffer.
pub fn wav_bytes(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_len = samples.len() * 2; // mono, 2 bytes/sample
    let byte_rate = sample_rate * 2;
    let mut buf = Vec::with_capacity(44 + data_len);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&((36 + data_len) as u32).to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // PCM fmt chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // audio format = PCM
    buf.extend_from_slice(&1u16.to_le_bytes()); // channels = mono
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&(data_len as u32).to_le_bytes());
    for &s in samples {
        let q = (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        buf.extend_from_slice(&q.to_le_bytes());
    }
    buf
}
