//! Native audio playback. Synthesis happens up front on the main thread; the
//! cpal output callback only pops mono samples from an `rtrb` SPSC ring and
//! zero-fills underruns — no allocation, no synthesis on the audio thread
//! (docs/research/02-architecture-v2.md §native; CLAUDE.md no-alloc rule).

use std::fmt;

#[allow(unused_imports)]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};

/// Fill `out` (interleaved, `channels` samples per frame) from the ring,
/// duplicating each mono sample across the channels; zero-fill any underrun.
/// Returns the number of frames taken from the ring. Allocation-free — this
/// is the audio-callback hot path (see tests/realtime.rs).
pub fn fill_frames(consumer: &mut rtrb::Consumer<f32>, out: &mut [f32], channels: usize) -> usize {
    // STUB (Phase 8 red): real pop + fan-out + zero-fill lands after red.
    let _ = (consumer, out, channels);
    0
}

/// Mono convenience wrapper over [`fill_frames`].
pub fn fill_from_ring(consumer: &mut rtrb::Consumer<f32>, out: &mut [f32]) -> usize {
    fill_frames(consumer, out, 1)
}

/// Why live playback could not run. Every variant except [`PlayError::Render`]
/// renders a stable "no usable audio output device" message so headless
/// environments (CI, WSL) can detect and skip playback.
#[derive(Debug)]
pub enum PlayError {
    NoDevice,
    NoF32Config,
    Configs(String),
    Build(String),
    Stream(String),
    /// The utterance failed to compile/render (message already formatted).
    Render(String),
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let PlayError::Render(m) = self {
            return write!(f, "{m}");
        }
        write!(
            f,
            "no usable audio output device (use --out FILE to render a WAV instead)"
        )?;
        match self {
            PlayError::NoF32Config => write!(f, " [no f32 output format]"),
            PlayError::Configs(m) | PlayError::Build(m) | PlayError::Stream(m) => {
                write!(f, " [{m}]")
            }
            _ => Ok(()),
        }
    }
}

/// Negotiate an f32 output config: prefer the device default, else the first
/// f32-capable range. Returns (config, sample_rate, channels).
#[allow(dead_code)]
fn choose_config(device: &cpal::Device) -> Result<(StreamConfig, u32, u16), PlayError> {
    let _ = (device, SampleFormat::F32);
    Err(PlayError::NoF32Config)
}

/// Render the utterance at the negotiated device sample rate, then play it.
/// `render` is called once with the chosen sample rate (synthesis is exact at
/// any rate — the schedule is in milliseconds).
pub fn play(render: impl FnOnce(u32) -> Result<Vec<f32>, String>) -> Result<(), PlayError> {
    // STUB (Phase 8 red): real cpal negotiation + streaming lands after red.
    let _ = render;
    Err(PlayError::NoDevice)
}
