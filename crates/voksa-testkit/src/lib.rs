//! voksa-testkit: dev-only acoustic verification helpers.
//!
//! FFT formant measurement per docs/research/03-implementation-playbook.md §c:
//! Hann-windowed power-of-two slice, band-limited peak-picking, parabolic
//! sub-bin interpolation. Reused by the engine spike (Phase 1), the per-vowel
//! phoneme tests (Phase 2), and the prosody checks (Phase 7).

use std::path::Path;

use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};

/// Analysis window length. At 48 kHz this is 85 ms of audio, 11.72 Hz/bin.
pub const FFT_LEN: usize = 4096;

/// Formant search bands in Hz (playbook §c).
const F1_BAND: (f32, f32) = (200.0, 1000.0);
const F2_BAND: (f32, f32) = (800.0, 2500.0);
const F3_BAND: (f32, f32) = (1500.0, 3500.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormantCheck {
    pub f1: f32,
    pub f2: f32,
    pub f3: f32,
}

/// Measure F1/F2/F3 of a steady vowel by FFT peak-picking.
///
/// Takes a centered [`FFT_LEN`] slice so onset/offset transients are excluded.
/// Panics if fewer than [`FFT_LEN`] samples are provided.
pub fn measure_formants_fft(samples: &[f32], sample_rate: u32) -> FormantCheck {
    assert!(
        samples.len() >= FFT_LEN,
        "need >= {FFT_LEN} samples, got {}",
        samples.len()
    );
    let start = (samples.len() - FFT_LEN) / 2;
    let windowed = hann_window(&samples[start..start + FFT_LEN]);
    let spectrum = samples_fft_to_spectrum(
        &windowed,
        sample_rate,
        FrequencyLimit::Range(150.0, 3600.0),
        Some(&divide_by_N_sqrt),
    )
    .expect("FFT analysis failed");
    let bins: Vec<(f32, f32)> = spectrum
        .data()
        .iter()
        .map(|(f, v)| (f.val(), v.val()))
        .collect();
    FormantCheck {
        f1: band_peak(&bins, F1_BAND),
        f2: band_peak(&bins, F2_BAND),
        f3: band_peak(&bins, F3_BAND),
    }
}

/// Strongest spectral peak inside [lo, hi], refined by parabolic interpolation
/// over the peak bin and its neighbours (playbook §c formula).
fn band_peak(bins: &[(f32, f32)], (lo, hi): (f32, f32)) -> f32 {
    let idx = bins
        .iter()
        .enumerate()
        .filter(|(_, (f, _))| *f >= lo && *f <= hi)
        .max_by(|(_, (_, a)), (_, (_, b))| a.partial_cmp(b).expect("NaN magnitude"))
        .map(|(i, _)| i)
        .expect("frequency band contains no FFT bins");
    if idx == 0 || idx + 1 >= bins.len() {
        return bins[idx].0;
    }
    let (m0, m1, m2) = (bins[idx - 1].1, bins[idx].1, bins[idx + 1].1);
    let denom = m0 - 2.0 * m1 + m2;
    if denom.abs() < f32::EPSILON {
        return bins[idx].0;
    }
    let delta = (0.5 * (m0 - m2) / denom).clamp(-0.5, 0.5);
    let bin_hz = bins[idx + 1].0 - bins[idx].0;
    bins[idx].0 + delta * bin_hz
}

/// Assert each measured formant is within `tol_frac` (e.g. 0.10 = ±10%) of its
/// target. Panics with the measured-vs-target values on failure.
pub fn assert_formants(got: &FormantCheck, target: &FormantCheck, tol_frac: f32) {
    for (name, g, t) in [
        ("F1", got.f1, target.f1),
        ("F2", got.f2, target.f2),
        ("F3", got.f3, target.f3),
    ] {
        let tol = t * tol_frac;
        assert!(
            (g - t).abs() <= tol,
            "{name}: measured {g:.1} Hz is not within ±{tol:.1} Hz of target {t:.1} Hz"
        );
    }
}

/// Write mono f32 samples as a 16-bit PCM WAV, creating parent directories.
pub fn write_wav(path: impl AsRef<Path>, samples: &[f32], sample_rate: u32) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create WAV parent directory");
    }
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("create WAV");
    for &s in samples {
        let clamped = (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        writer.write_sample(clamped).expect("write WAV sample");
    }
    writer.finalize().expect("finalize WAV");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    const SR: u32 = 48_000;

    fn tones(freqs_amps: &[(f32, f32)], len: usize) -> Vec<f32> {
        (0..len)
            .map(|n| {
                let t = n as f32 / SR as f32;
                freqs_amps
                    .iter()
                    .map(|(f, a)| a * (TAU * f * t).sin())
                    .sum::<f32>()
                    * 0.2
            })
            .collect()
    }

    #[test]
    fn finds_three_known_peaks_in_formant_bands() {
        // Formant-like triple at the /a/ targets with falling amplitudes.
        let samples = tones(&[(730.0, 1.0), (1090.0, 0.6), (2440.0, 0.3)], 8192);
        let got = measure_formants_fft(&samples, SR);
        assert_formants(
            &got,
            &FormantCheck {
                f1: 730.0,
                f2: 1090.0,
                f3: 2440.0,
            },
            0.02,
        );
    }

    #[test]
    fn parabolic_interpolation_refines_off_bin_tone() {
        // 1000 Hz sits between bins at 48 kHz / 4096 (11.72 Hz spacing).
        let samples = tones(&[(1000.0, 1.0)], 8192);
        let got = measure_formants_fft(&samples, SR);
        assert!(
            (got.f1 - 1000.0).abs() < 5.0,
            "interpolated peak {:.2} Hz should be within 5 Hz of 1000 Hz",
            got.f1
        );
    }

    #[test]
    #[should_panic(expected = "not within")]
    fn assert_formants_rejects_out_of_band() {
        assert_formants(
            &FormantCheck {
                f1: 500.0,
                f2: 1090.0,
                f3: 2440.0,
            },
            &FormantCheck {
                f1: 730.0,
                f2: 1090.0,
                f3: 2440.0,
            },
            0.10,
        );
    }
}
