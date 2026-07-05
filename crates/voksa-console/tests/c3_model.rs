//! C3 wave 1: the four pure model functions the engine/audio UI projects.
//! Transcript tokenization is NEW (the reference page set flat textContent);
//! peaks + wav_bytes replicate the reference/CLI math exactly.

use voksa_console::engine::describe;
use voksa_console::model::{Tok, TokKind, peaks, tokenize, wav_bytes};
use voksa_core::compiler::CompileError;

fn t(text: &str, kind: TokKind) -> Tok {
    Tok {
        text: text.to_string(),
        kind,
    }
}

#[test]
fn tokenize_classifies_markers() {
    use TokKind::*;
    assert_eq!(
        tokenize("coi MUN.je"),
        vec![
            t("coi ", Plain),
            t("MUN", Stress),
            t(".", Dot),
            t("je", Plain)
        ],
    );
    assert_eq!(
        tokenize("coi ‖ la DJAN. ‖ cu KLA.ma"),
        vec![
            t("coi ", Plain),
            t("‖", Pause),
            t(" la ", Plain),
            t("DJAN", Stress),
            t(".", Dot),
            t(" ", Plain),
            t("‖", Pause),
            t(" cu ", Plain),
            t("KLA", Stress),
            t(".", Dot),
            t("ma", Plain),
        ],
    );
    // Buffer vowel is one token; the stressed cluster around it splits.
    assert_eq!(
        tokenize("V(ɪ)RU.si"),
        vec![
            t("V", Stress),
            t("(ɪ)", Buffer),
            t("RU", Stress),
            t(".", Dot),
            t("si", Plain),
        ],
    );
    // Apostrophe [h] is distinct even inside a stressed syllable.
    assert_eq!(
        tokenize("SA'E"),
        vec![t("SA", Stress), t("'", Aspirate), t("E", Stress)],
    );
    assert!(tokenize("").is_empty());
}

#[test]
fn peaks_are_abs_max_per_column() {
    // step = max(1, floor(4/2)) = 2; columns are abs-max over each half.
    assert_eq!(peaks(&[0.1, -0.5, 0.9, -0.3], 2), vec![0.5, 0.9]);
    // cols > len: step = 1; columns past the end are 0.
    assert_eq!(peaks(&[0.6], 3), vec![0.6, 0.0, 0.0]);
    // empty input → all-zero columns.
    assert_eq!(peaks(&[], 4), vec![0.0; 4]);
}

#[test]
fn wav_bytes_match_the_cli_encoder() {
    let u16_at = |b: &[u8], i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    let u32_at = |b: &[u8], i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    let b = wav_bytes(&[0.0; 4], 48_000);
    assert_eq!(b.len(), 52, "44-byte header + 8 bytes of PCM");
    assert_eq!(&b[0..4], b"RIFF");
    assert_eq!(u32_at(&b, 4), 44);
    assert_eq!(&b[8..12], b"WAVE");
    assert_eq!(&b[12..16], b"fmt ");
    assert_eq!(u32_at(&b, 16), 16);
    assert_eq!(u16_at(&b, 20), 1, "PCM");
    assert_eq!(u16_at(&b, 22), 1, "mono");
    assert_eq!(u32_at(&b, 24), 48_000);
    assert_eq!(u32_at(&b, 28), 96_000, "byte rate = sr * 2");
    assert_eq!(u16_at(&b, 32), 2, "block align");
    assert_eq!(u16_at(&b, 34), 16, "bits per sample");
    assert_eq!(&b[36..40], b"data");
    assert_eq!(u32_at(&b, 40), 8);
    // Quantization + clipping.
    let q = wav_bytes(&[1.0, -1.0, 2.0, -2.0], 48_000);
    let sample = |i: usize| i16::from_le_bytes([q[44 + i * 2], q[44 + i * 2 + 1]]);
    assert_eq!(sample(0), i16::MAX);
    assert_eq!(sample(1), -i16::MAX);
    assert_eq!(sample(2), i16::MAX, "clips");
    assert_eq!(sample(3), -i16::MAX, "clips");
}

#[test]
fn describe_names_the_offending_input() {
    let word = CompileError::Word {
        word: "qwxz".to_string(),
        error: voksa_core::letters::WordError::InvalidCharacter('q'),
    };
    assert!(describe(&word).contains("qwxz"), "names the word");
    assert!(
        !describe(&CompileError::Empty).is_empty(),
        "empty has a message"
    );
}
