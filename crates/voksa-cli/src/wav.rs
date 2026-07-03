//! Minimal RIFF/WAVE writer (mono, 16-bit PCM). hound stays a
//! dev-dependency; the shipping CLI must not link it, so we hand-roll the
//! 44-byte header. Quantization matches voksa-testkit::write_wav so CLI
//! output is bit-comparable with the listening battery.

use std::io;
use std::path::Path;

/// Encode mono f32 samples ([-1, 1]) to a 16-bit PCM WAV byte buffer.
pub fn wav_bytes(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    // STUB (Phase 8 red): the real header + PCM encoding lands after red.
    let _ = (samples, sample_rate);
    Vec::new()
}

/// Write mono f32 samples to a WAV file at `path`.
pub fn write_wav(path: impl AsRef<Path>, samples: &[f32], sample_rate: u32) -> io::Result<()> {
    std::fs::write(path, wav_bytes(samples, sample_rate))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u16_at(b: &[u8], i: usize) -> u16 {
        u16::from_le_bytes([b[i], b[i + 1]])
    }
    fn u32_at(b: &[u8], i: usize) -> u32 {
        u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
    }

    #[test]
    fn header_golden_bytes() {
        let b = wav_bytes(&[0.0; 4], 48_000);
        assert_eq!(b.len(), 52, "44-byte header + 8 bytes of PCM");
        assert_eq!(&b[0..4], b"RIFF");
        assert_eq!(u32_at(&b, 4), 44, "RIFF size = 36 + data(8)");
        assert_eq!(&b[8..12], b"WAVE");
        assert_eq!(&b[12..16], b"fmt ");
        assert_eq!(u32_at(&b, 16), 16, "fmt chunk size");
        assert_eq!(u16_at(&b, 20), 1, "PCM");
        assert_eq!(u16_at(&b, 22), 1, "mono");
        assert_eq!(u32_at(&b, 24), 48_000, "sample rate");
        assert_eq!(u32_at(&b, 28), 96_000, "byte rate = sr * 2");
        assert_eq!(u16_at(&b, 32), 2, "block align");
        assert_eq!(u16_at(&b, 34), 16, "bits per sample");
        assert_eq!(&b[36..40], b"data");
        assert_eq!(u32_at(&b, 40), 8, "data length");
    }

    #[test]
    fn quantization_and_clipping() {
        let b = wav_bytes(&[1.0, -1.0, 2.0, -2.0], 48_000);
        let sample = |i: usize| i16::from_le_bytes([b[44 + i * 2], b[44 + i * 2 + 1]]);
        assert_eq!(sample(0), i16::MAX, "1.0 -> 32767");
        assert_eq!(sample(1), -i16::MAX, "-1.0 -> -32767");
        assert_eq!(sample(2), i16::MAX, "2.0 clips to 32767");
        assert_eq!(sample(3), -i16::MAX, "-2.0 clips to -32767");
    }
}
