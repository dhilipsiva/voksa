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

/// Like [`measure_formants_fft`] but with sequential band exclusion: each
/// formant is searched ABOVE the previous one (F2 from max(800, F1+150) Hz,
/// F3 from max(1500, F2+200) Hz). The fixed playbook bands overlap, so for
/// front vowels (/e/ 1840, /i/ 2290) the louder F2 peak sits inside the F3
/// band and wins it; ordered search restores the F1<F2<F3 structure without
/// touching the Phase-1 harness.
pub fn measure_formants_fft_ordered(samples: &[f32], sample_rate: u32) -> FormantCheck {
    assert!(
        samples.len() >= FFT_LEN,
        "need >= {FFT_LEN} samples, got {}",
        samples.len()
    );
    let start = (samples.len() - FFT_LEN) / 2;
    let bins = spectrum_bins(&samples[start..start + FFT_LEN], sample_rate, 150.0, 3600.0);
    let f1 = band_peak(&bins, F1_BAND);
    let f2 = band_peak(&bins, (F2_BAND.0.max(f1 + 150.0), F2_BAND.1));
    let f3 = band_peak(&bins, (F3_BAND.0.max(f2 + 200.0), F3_BAND.1));
    FormantCheck { f1, f2, f3 }
}

/// Like [`measure_formants_fft_ordered`], refined for periodic sources with a
/// known F0: an FFT of a voiced sound only samples the resonance envelope at
/// source harmonics, so the band peak can sit a harmonic away from the true
/// formant. For each formant, log-parabolic interpolation across the peak
/// harmonic and its ±F0 neighbours recovers the envelope maximum BETWEEN
/// harmonics.
pub fn measure_formants_fft_harmonic(samples: &[f32], sample_rate: u32, f0: f32) -> FormantCheck {
    assert!(
        samples.len() >= FFT_LEN,
        "need >= {FFT_LEN} samples, got {}",
        samples.len()
    );
    let start = (samples.len() - FFT_LEN) / 2;
    let bins = spectrum_bins(&samples[start..start + FFT_LEN], sample_rate, 100.0, 3600.0);
    let f1 = harmonic_refined_peak(&bins, F1_BAND, f0);
    let f2 = harmonic_refined_peak(&bins, (F2_BAND.0.max(f1 + 150.0), F2_BAND.1), f0);
    let f3 = harmonic_refined_peak(&bins, (F3_BAND.0.max(f2 + 200.0), F3_BAND.1), f0);
    FormantCheck { f1, f2, f3 }
}

fn harmonic_refined_peak(bins: &[(f32, f32)], band: (f32, f32), f0: f32) -> f32 {
    let peak = band_peak(bins, band);
    // Magnitude of the strongest bin within ±f0/4 of a frequency.
    let mag_near = |f: f32| -> Option<f32> {
        bins.iter()
            .filter(|(bf, _)| (bf - f).abs() <= f0 / 4.0)
            .map(|(_, m)| *m)
            .fold(None, |acc: Option<f32>, m| {
                Some(acc.map_or(m, |a| a.max(m)))
            })
    };
    let m0 = mag_near(peak);
    let m_lo = mag_near(peak - f0);
    let m_hi = mag_near(peak + f0);
    match (m_lo, m0, m_hi) {
        (Some(lo), Some(mid), Some(hi)) if lo > 0.0 && mid > 0.0 && hi > 0.0 => {
            // Whiten the ~-12 dB/oct glottal source tilt (multiply magnitude
            // by f²  ⇒  add 2·ln f) so the parabola sees the resonance
            // envelope, not the source rolloff.
            let (l, m, h) = (
                lo.ln() + 2.0 * (peak - f0).max(1.0).ln(),
                mid.ln() + 2.0 * peak.ln(),
                hi.ln() + 2.0 * (peak + f0).ln(),
            );
            let denom = l - 2.0 * m + h;
            if denom.abs() < f32::EPSILON {
                return peak;
            }
            let delta = (0.5 * (l - h) / denom).clamp(-0.5, 0.5);
            peak + delta * f0
        }
        _ => peak,
    }
}

/// Measure F1–F3 by LPC root-finding: downsample to 16 kHz → pre-emphasis →
/// Hamming → autocorrelation → Levinson-Durbin (order 18) → Durand-Kerner
/// polynomial roots → pole frequencies. Median over three analysis frames.
///
/// Hand-rolled because loqa-voice-dsp 0.5 proved empirically broken (returned
/// garbage on textbook synthetic vowels). Only meaningful for glottal-excited,
/// vowel-like signals with formants below ~8 kHz — never use for sibilant
/// noise bands, and never feed it pure sinusoids.
pub fn measure_formants_lpc(samples: &[f32], sample_rate: u32) -> FormantCheck {
    let s16 = lpc::to_16k(samples, sample_rate);
    let frames = lpc::analysis_frames(&s16, 1024, 3);
    assert!(!frames.is_empty(), "signal too short for LPC analysis");
    let mut per_formant: [Vec<f32>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for frame in frames {
        if let Some(f) = lpc::frame_formants(frame, 16_000, 18) {
            per_formant[0].push(f[0]);
            per_formant[1].push(f[1]);
            per_formant[2].push(f[2]);
        }
    }
    assert!(
        !per_formant[0].is_empty(),
        "LPC found no stable formants in any frame"
    );
    FormantCheck {
        f1: lpc::median(&mut per_formant[0]),
        f2: lpc::median(&mut per_formant[1]),
        f3: lpc::median(&mut per_formant[2]),
    }
}

mod lpc {
    //! Classic autocorrelation LPC formant estimation (playbook §c fallback).

    /// Two cascaded RBJ low-pass biquads at 6.5 kHz, then 3:1 decimation.
    pub(super) fn to_16k(samples: &[f32], sample_rate: u32) -> Vec<f32> {
        assert_eq!(
            sample_rate, 48_000,
            "LPC path assumes the project rate of 48 kHz"
        );
        let mut lp1 = Lowpass::new(48_000.0, 6500.0);
        let mut lp2 = Lowpass::new(48_000.0, 6500.0);
        samples
            .iter()
            .map(|&x| lp2.process(lp1.process(x)))
            .collect::<Vec<f32>>()
            .into_iter()
            .step_by(3)
            .collect()
    }

    /// `n` frames of `len` samples spread across the middle of the signal.
    pub(super) fn analysis_frames(samples: &[f32], len: usize, n: usize) -> Vec<&[f32]> {
        if samples.len() < len {
            return Vec::new();
        }
        (0..n)
            .map(|i| {
                let start = (samples.len() - len) * (i + 1) / (n + 1);
                &samples[start..start + len]
            })
            .collect()
    }

    /// LPC formants of one frame: the three lowest well-damped pole
    /// frequencies. `None` if fewer than three plausible poles emerge.
    pub(super) fn frame_formants(frame: &[f32], sr: u32, order: usize) -> Option<[f32; 3]> {
        // Pre-emphasis + Hamming.
        let mut x: Vec<f64> = Vec::with_capacity(frame.len());
        let mut prev = 0.0f64;
        for &s in frame {
            let s = f64::from(s);
            x.push(s - 0.97 * prev);
            prev = s;
        }
        let n = x.len();
        for (i, v) in x.iter_mut().enumerate() {
            let w = 0.54 - 0.46 * (core::f64::consts::TAU * i as f64 / (n - 1) as f64).cos();
            *v *= w;
        }
        // Autocorrelation and Levinson-Durbin.
        let mut r = vec![0.0f64; order + 1];
        for (lag, rl) in r.iter_mut().enumerate() {
            *rl = x[..n - lag].iter().zip(&x[lag..]).map(|(a, b)| a * b).sum();
        }
        if r[0] <= 0.0 {
            return None;
        }
        let a = levinson(&r, order)?;
        // Roots of z^p + a1 z^(p-1) + ... + ap.
        let mut poly = vec![(1.0f64, 0.0f64)];
        poly.extend(a.iter().map(|&c| (c, 0.0)));
        let roots = durand_kerner(&poly);
        // Poles -> formant candidates.
        let sr = f64::from(sr);
        let mut freqs: Vec<f64> = roots
            .iter()
            .filter(|(_, im)| *im > 0.0)
            .filter_map(|&(re, im)| {
                let mag = (re * re + im * im).sqrt();
                let freq = im.atan2(re) * sr / core::f64::consts::TAU;
                let bw = -mag.ln() * sr / core::f64::consts::PI;
                (mag > 0.7 && mag < 1.0 && freq > 120.0 && freq < sr / 2.0 - 300.0 && bw < 700.0)
                    .then_some(freq)
            })
            .collect();
        freqs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (freqs.len() >= 3).then(|| [freqs[0] as f32, freqs[1] as f32, freqs[2] as f32])
    }

    /// Levinson-Durbin recursion; returns a[1..=order] of A(z).
    fn levinson(r: &[f64], order: usize) -> Option<Vec<f64>> {
        let mut a = vec![0.0f64; order + 1];
        let mut e = r[0];
        for i in 1..=order {
            let mut acc = r[i];
            for j in 1..i {
                acc += a[j] * r[i - j];
            }
            let k = -acc / e;
            if !k.is_finite() {
                return None;
            }
            a[i] = k;
            for j in 1..=i / 2 {
                let tmp = a[j] + k * a[i - j];
                a[i - j] += k * a[j];
                a[j] = tmp;
            }
            e *= 1.0 - k * k;
            if e <= 0.0 {
                return None;
            }
        }
        Some(a[1..].to_vec())
    }

    /// Durand-Kerner (Weierstrass) simultaneous root iteration for a monic
    /// real polynomial given as [(c0=1,0), (c1,0), ...] highest degree first.
    fn durand_kerner(poly: &[(f64, f64)]) -> Vec<(f64, f64)> {
        let deg = poly.len() - 1;
        let mul = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
        let sub = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0, a.1 - b.1);
        let div = |a: (f64, f64), b: (f64, f64)| {
            let d = b.0 * b.0 + b.1 * b.1;
            ((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d)
        };
        let eval = |z: (f64, f64)| {
            let mut acc = (0.0, 0.0);
            for &c in poly {
                acc = mul(acc, z);
                acc = (acc.0 + c.0, acc.1 + c.1);
            }
            acc
        };
        // Initial guesses on a spiral inside the unit circle's neighbourhood.
        let mut roots: Vec<(f64, f64)> = (0..deg)
            .map(|k| {
                let ang = 0.4 + 2.0 * core::f64::consts::PI * k as f64 / deg as f64;
                (0.9 * ang.cos(), 0.9 * ang.sin())
            })
            .collect();
        for _ in 0..200 {
            let mut max_step = 0.0f64;
            for i in 0..deg {
                let mut denom = (1.0, 0.0);
                for j in 0..deg {
                    if i != j {
                        denom = mul(denom, sub(roots[i], roots[j]));
                    }
                }
                let delta = div(eval(roots[i]), denom);
                roots[i] = sub(roots[i], delta);
                max_step = max_step.max((delta.0 * delta.0 + delta.1 * delta.1).sqrt());
            }
            if max_step < 1e-10 {
                break;
            }
        }
        roots
    }

    pub(super) fn median(values: &mut [f32]) -> f32 {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        values[values.len() / 2]
    }

    /// RBJ low-pass biquad, Q = 1/sqrt(2).
    struct Lowpass {
        b0: f64,
        b1: f64,
        b2: f64,
        a1: f64,
        a2: f64,
        x1: f64,
        x2: f64,
        y1: f64,
        y2: f64,
    }

    impl Lowpass {
        fn new(sr: f64, fc: f64) -> Self {
            let w0 = core::f64::consts::TAU * fc / sr;
            let alpha = w0.sin() / core::f64::consts::SQRT_2;
            let a0 = 1.0 + alpha;
            let cosw = w0.cos();
            Self {
                b0: (1.0 - cosw) / 2.0 / a0,
                b1: (1.0 - cosw) / a0,
                b2: (1.0 - cosw) / 2.0 / a0,
                a1: -2.0 * cosw / a0,
                a2: (1.0 - alpha) / a0,
                x1: 0.0,
                x2: 0.0,
                y1: 0.0,
                y2: 0.0,
            }
        }

        fn process(&mut self, x: f32) -> f32 {
            let x = f64::from(x);
            let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
                - self.a1 * self.y1
                - self.a2 * self.y2;
            self.x2 = self.x1;
            self.x1 = x;
            self.y2 = self.y1;
            self.y1 = y;
            y as f32
        }
    }
}

/// Like [`assert_formants`] but with per-formant fractional tolerances and an
/// absolute floor for F1 (playbook §c: F1 within ±frac OR ±floor Hz,
/// whichever is larger).
pub fn assert_formants_tol(
    got: &FormantCheck,
    target: &FormantCheck,
    frac: [f32; 3],
    f1_floor_hz: f32,
) {
    for (name, g, t, fr, floor) in [
        ("F1", got.f1, target.f1, frac[0], f1_floor_hz),
        ("F2", got.f2, target.f2, frac[1], 0.0),
        ("F3", got.f3, target.f3, frac[2], 0.0),
    ] {
        let tol = (t * fr).max(floor);
        assert!(
            (g - t).abs() <= tol,
            "{name}: measured {g:.1} Hz is not within ±{tol:.1} Hz of target {t:.1} Hz"
        );
    }
}

fn assert_pow2_window(win: usize) {
    assert!(
        win.is_power_of_two() && win <= 16384,
        "FFT window must be a power of two <= 16384, got {win}"
    );
}

fn spectrum_bins(frame: &[f32], sample_rate: u32, lo: f32, hi: f32) -> Vec<(f32, f32)> {
    let windowed = hann_window(frame);
    let spectrum = samples_fft_to_spectrum(
        &windowed,
        sample_rate,
        FrequencyLimit::Range(lo, hi),
        Some(&divide_by_N_sqrt),
    )
    .expect("FFT analysis failed");
    spectrum
        .data()
        .iter()
        .map(|(f, v)| (f.val(), v.val()))
        .collect()
}

/// Energy-weighted spectral centroid (Σ f·m² / Σ m²) over [lo, hi] Hz,
/// averaged across up to 3 non-overlapping `win`-sample frames spread over the
/// signal (Welch-style, so the number is physically meaningful, not just
/// deterministic).
pub fn band_energy_centroid(
    samples: &[f32],
    sample_rate: u32,
    lo: f32,
    hi: f32,
    win: usize,
) -> f32 {
    assert_pow2_window(win);
    assert!(samples.len() >= win, "need >= {win} samples");
    let n_frames = (samples.len() / win).clamp(1, 3);
    let mut acc = 0.0;
    for i in 0..n_frames {
        let start = if n_frames == 1 {
            (samples.len() - win) / 2
        } else {
            i * (samples.len() - win) / (n_frames - 1)
        };
        let bins = spectrum_bins(&samples[start..start + win], sample_rate, lo, hi);
        let (mut num, mut den) = (0.0f32, 0.0f32);
        for (f, m) in bins {
            num += f * m * m;
            den += m * m;
        }
        acc += if den > 0.0 { num / den } else { 0.0 };
    }
    acc / n_frames as f32
}

/// Total spectral energy (Σ m²) over [lo, hi] Hz in a centered `win` window.
pub fn band_energy(samples: &[f32], sample_rate: u32, lo: f32, hi: f32, win: usize) -> f32 {
    assert_pow2_window(win);
    assert!(samples.len() >= win, "need >= {win} samples");
    let start = (samples.len() - win) / 2;
    spectrum_bins(&samples[start..start + win], sample_rate, lo, hi)
        .iter()
        .map(|(_, m)| m * m)
        .sum()
}

/// Strongest-peak frequency (parabolic-interpolated) in [lo, hi] Hz of a
/// centered `win`-sample window.
pub fn band_peak_hz(samples: &[f32], sample_rate: u32, lo: f32, hi: f32, win: usize) -> f32 {
    assert_pow2_window(win);
    assert!(samples.len() >= win, "need >= {win} samples");
    let start = (samples.len() - win) / 2;
    let bins = spectrum_bins(&samples[start..start + win], sample_rate, lo, hi);
    band_peak(&bins, (lo, hi))
}

/// Framewise band-peak trajectory (window `win`, hop `hop`): one Hz value per
/// frame. Used for diphthong glide monotonicity checks.
pub fn track_band_peak(
    samples: &[f32],
    sample_rate: u32,
    lo: f32,
    hi: f32,
    win: usize,
    hop: usize,
) -> Vec<f32> {
    assert_pow2_window(win);
    assert!(hop > 0, "hop must be positive");
    let mut track = Vec::new();
    let mut start = 0;
    while start + win <= samples.len() {
        let bins = spectrum_bins(&samples[start..start + win], sample_rate, lo, hi);
        track.push(band_peak(&bins, (lo, hi)));
        start += hop;
    }
    track
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

    /// RBJ constant-peak-gain bandpass biquad — test-local, used to build
    /// glottal-like synthetic vowels for the LPC self-test (LPC does not
    /// round-trip pure sinusoids).
    struct Biquad {
        b0: f32,
        b2: f32,
        a1: f32,
        a2: f32,
        x1: f32,
        x2: f32,
        y1: f32,
        y2: f32,
    }

    impl Biquad {
        fn bandpass(sr: f32, f0: f32, bw: f32) -> Self {
            let w0 = TAU * f0 / sr;
            let q = f0 / bw;
            let alpha = w0.sin() / (2.0 * q);
            let a0 = 1.0 + alpha;
            Self {
                b0: alpha / a0,
                b2: -alpha / a0,
                a1: (-2.0 * w0.cos()) / a0,
                a2: (1.0 - alpha) / a0,
                x1: 0.0,
                x2: 0.0,
                y1: 0.0,
                y2: 0.0,
            }
        }

        fn process(&mut self, x: f32) -> f32 {
            let y = self.b0 * x + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
            self.x2 = self.x1;
            self.x1 = x;
            self.y2 = self.y1;
            self.y1 = y;
            y
        }
    }

    /// 120 Hz pulse train through three parallel resonators — a crude
    /// synthetic vowel with known "formants".
    fn synthetic_vowel(sr: u32, f: [f32; 3], len: usize) -> Vec<f32> {
        let period = (sr as f32 / 120.0) as usize;
        let mut bps: Vec<Biquad> = f
            .iter()
            .map(|f0| Biquad::bandpass(sr as f32, *f0, 0.12 * f0))
            .collect();
        let amps = [1.0, 0.6, 0.3];
        (0..len)
            .map(|n| {
                let x = if n % period == 0 { 1.0 } else { 0.0 };
                bps.iter_mut()
                    .zip(amps)
                    .map(|(bp, a)| a * bp.process(x))
                    .sum::<f32>()
                    * 0.5
            })
            .collect()
    }

    #[test]
    fn lpc_recovers_resonator_poles() {
        let samples = synthetic_vowel(SR, [500.0, 1500.0, 2500.0], 16384);
        let got = measure_formants_lpc(&samples, SR);
        assert_formants_tol(
            &got,
            &FormantCheck {
                f1: 500.0,
                f2: 1500.0,
                f3: 2500.0,
            },
            [0.10, 0.10, 0.10],
            50.0,
        );
    }

    #[test]
    fn centroid_of_narrowband_tone_sits_at_its_frequency() {
        let samples = tones(&[(3000.0, 1.0)], 16384);
        let c = band_energy_centroid(&samples, SR, 150.0, 12_000.0, 4096);
        assert!(
            (c - 3000.0).abs() < 120.0,
            "centroid {c:.1} Hz should be near 3000 Hz"
        );
    }

    #[test]
    fn chirp_track_is_monotonic() {
        // Linear chirp 800 -> 2000 Hz over 0.5 s: phase = 2π(f0 t + k t²/2).
        let dur = 24_000;
        let (f0, f1) = (800.0, 2000.0);
        let k = (f1 - f0) / (dur as f32 / SR as f32);
        let samples: Vec<f32> = (0..dur)
            .map(|n| {
                let t = n as f32 / SR as f32;
                (TAU * (f0 * t + 0.5 * k * t * t)).sin() * 0.5
            })
            .collect();
        let track = track_band_peak(&samples, SR, 500.0, 2500.0, 1024, 512);
        assert!(track.len() > 10);
        for w in track.windows(2) {
            assert!(
                w[1] >= w[0] - 25.0,
                "track must not fall: {} -> {}",
                w[0],
                w[1]
            );
        }
        assert!(
            track.last().unwrap() - track.first().unwrap() > 800.0,
            "track must rise ~1200 Hz: first {:.0}, last {:.0}",
            track.first().unwrap(),
            track.last().unwrap()
        );
    }
}
