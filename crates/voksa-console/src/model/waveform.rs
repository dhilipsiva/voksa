//! Waveform peak columns — the reference page's `drawWave` math, extracted
//! pure so the SVG renderer stays a thin projection.

/// Abs-peak per display column: `step = max(1, floor(len/cols))`, and column
/// `x` is the maximum `|sample|` over `pcm[x·step .. (x+1)·step]` (0 when that
/// slice is past the end). Length is always `cols`.
pub fn peaks(pcm: &[f32], cols: usize) -> Vec<f32> {
    if cols == 0 {
        return Vec::new();
    }
    let step = (pcm.len() / cols).max(1);
    (0..cols)
        .map(|x| {
            let start = x * step;
            let end = (start + step).min(pcm.len());
            if start >= pcm.len() {
                0.0
            } else {
                pcm[start..end].iter().fold(0.0f32, |m, s| m.max(s.abs()))
            }
        })
        .collect()
}
