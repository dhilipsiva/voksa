//! Native audio playback. Synthesis happens up front on the main thread; the
//! cpal output callback only pops mono samples from an `rtrb` SPSC ring and
//! zero-fills underruns — no allocation, no synthesis on the audio thread
//! (docs/research/02-architecture-v2.md §native; CLAUDE.md no-alloc rule).

use std::fmt;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};

/// Ring capacity in samples (~170 ms at 48 kHz) — enough cushion for the main
/// thread to keep the callback fed without a large startup latency.
const RING_CAPACITY: usize = 8192;

/// Fill `out` (interleaved, `channels` samples per frame) from the ring,
/// duplicating each mono sample across the channels; zero-fill any underrun.
/// Returns the number of frames taken from the ring. Allocation-free — this
/// is the audio-callback hot path (see tests/realtime.rs).
pub fn fill_frames(consumer: &mut rtrb::Consumer<f32>, out: &mut [f32], channels: usize) -> usize {
    let frames = out.len().checked_div(channels).unwrap_or(0);
    let n = consumer.slots().min(frames);
    if n > 0 {
        // read_chunk only errors when asking for more than is available; n is
        // clamped to slots(), so this never fails and never allocates.
        if let Ok(chunk) = consumer.read_chunk(n) {
            let (head, tail) = chunk.as_slices();
            for (i, &s) in head.iter().chain(tail.iter()).enumerate() {
                out[i * channels..(i + 1) * channels].fill(s);
            }
            chunk.commit_all();
        }
    }
    out[n * channels..].fill(0.0);
    n
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
fn choose_config(device: &cpal::Device) -> Result<(StreamConfig, u32, u16), PlayError> {
    let default = device
        .default_output_config()
        .map_err(|e| PlayError::Configs(e.to_string()))?;
    if default.sample_format() == SampleFormat::F32 {
        return Ok((default.config(), default.sample_rate(), default.channels()));
    }
    // Device default isn't f32: scan for any f32-capable range.
    let ranges = device
        .supported_output_configs()
        .map_err(|e| PlayError::Configs(e.to_string()))?;
    for range in ranges {
        if range.sample_format() == SampleFormat::F32 {
            let cfg = range.with_max_sample_rate();
            return Ok((cfg.config(), cfg.sample_rate(), cfg.channels()));
        }
    }
    Err(PlayError::NoF32Config)
}

/// Render the utterance at the negotiated device sample rate, then play it.
/// `render` is called once with the chosen sample rate (synthesis is exact at
/// any rate — the schedule is in milliseconds, so this is not resampling).
pub fn play(render: impl FnOnce(u32) -> Result<Vec<f32>, String>) -> Result<(), PlayError> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or(PlayError::NoDevice)?;
    let (config, sample_rate, channels) = choose_config(&device)?;
    let samples = render(sample_rate).map_err(PlayError::Render)?;
    let channels = channels as usize;

    let (mut producer, mut consumer) = rtrb::RingBuffer::<f32>::new(RING_CAPACITY);
    // Prefill before starting the stream to avoid a startup underrun.
    let mut pushed = 0usize;
    while pushed < samples.len() && producer.push(samples[pushed]).is_ok() {
        pushed += 1;
    }

    let stream = device
        .build_output_stream(
            config,
            move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                fill_frames(&mut consumer, out, channels);
            },
            |err| eprintln!("voksa: audio stream error: {err}"),
            None,
        )
        .map_err(|e| PlayError::Build(e.to_string()))?;
    stream
        .play()
        .map_err(|e| PlayError::Stream(e.to_string()))?;

    // Feed the remainder from the main thread, then wait for the ring to drain
    // plus a short tail for the device buffer to flush. A deadline of the audio
    // length + 2 s guards against a device that accepts the stream but never
    // pulls (which would otherwise spin forever).
    let audio_ms = (samples.len() as u64 * 1000) / u64::from(sample_rate.max(1));
    let deadline = Instant::now() + Duration::from_millis(audio_ms + 2000);
    while pushed < samples.len() && Instant::now() < deadline {
        match producer.push(samples[pushed]) {
            Ok(()) => pushed += 1,
            Err(_) => std::thread::sleep(Duration::from_millis(2)),
        }
    }
    while producer.slots() < RING_CAPACITY && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    std::thread::sleep(Duration::from_millis(200));
    Ok(())
}
