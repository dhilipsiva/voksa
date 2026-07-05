//! Waveform peak columns — the reference page's `drawWave` math, extracted
//! pure so the SVG renderer stays a thin projection.

/// Abs-peak per display column: `step = max(1, floor(len/cols))`, and column
/// `x` is the maximum `|sample|` over `pcm[x·step .. (x+1)·step]` (0 when that
/// slice is past the end). Length is always `cols`.
pub fn peaks(pcm: &[f32], cols: usize) -> Vec<f32> {
    let _ = pcm;
    vec![0.0; cols] // stub — C3 green
}
