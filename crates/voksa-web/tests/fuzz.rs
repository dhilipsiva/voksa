//! Phase-11 W1 stable fuzzing of the browser surface: `synth` (the whole
//! decode → compile → prosody → render path) must be total on hostile f32
//! param blocks (NaN/inf — the decode filters per index) and produce finite
//! PCM whenever it succeeds. Render-bound, so the case count is CAPPED at
//! 1024 even under `cargo xtask fuzz` (deep runs report the cap) and renders
//! run at 8 kHz on short texts. Native-only, like tests/sentences.rs.
#![cfg(not(target_arch = "wasm32"))]

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence};
use voksa_web::{FULL_PARAM_COUNT, synth, transcription};

/// 8 kHz keeps the render cost per case ~6× under the shipping 48 kHz; the
/// schedule math is in ms, so the pipeline under test is identical.
const SR: u32 = 8000;

/// Deep-run cap: PROPTEST_CASES applies to the cheap voksa-core suite in the
/// tens of thousands; a render per case caps out far earlier. Honored bound,
/// not a silent one — `cargo xtask fuzz` prints it.
fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(64)
        .min(1024)
}

/// Short Lojban-flavored soup: enough to hit pauses, numbers, attitudinals,
/// and errors, while keeping the worst-case render bounded.
static SOUP: [char; 33] = [
    'a', 'e', 'i', 'o', 'u', 'y', 'b', 'c', 'd', 'f', 'g', 'j', 'k', 'l', 'm', 'n', 'p', 'r', 's',
    't', 'v', 'x', 'z', '\'', ',', '.', ' ', 'K', '3', ':', 'q', 'h', 'w',
];

fn short_soup() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(&SOUP[..]), 0..10)
        .prop_map(|chars| chars.into_iter().collect())
}

/// Deep-fuzz regression (minimized): a zeroed block with hostile-but-finite
/// values in /n/'s voice slots — F1 amplitude −7.2e27, aspiration +3e26, a
/// 10 s clamped duration. The two huge finite factors MULTIPLY in the noise
/// branch (≈ −2e54 > f32::MAX) → inf → NaN, and a poisoned filter state
/// stays NaN for the rest of the render. Ok PCM must be entirely finite;
/// hostile params degrade to silence, never non-finite output.
#[test]
fn hostile_finite_amps_render_finite() {
    let mut block = vec![0.0f32; 403];
    block[393] = -7.212_827_5e27; // nasal /n/ F1 amplitude
    block[401] = 2.962_394_5e26; // nasal /n/ aspiration
    block[402] = 2_354_284_300.0; // nasal /n/ dur_ms (clamps to 10 s)
    let pcm = synth("na.", 6, SR, &block).unwrap();
    let bad = pcm.iter().filter(|s| !s.is_finite()).count();
    assert_eq!(bad, 0, "{bad} non-finite samples on the hostile-amp block");
}

/// Companion hostile-but-finite prosody extremes (found green, kept as
/// coverage): subnormal declination, ±1e29 amplitude factor, 1e31 xu rise.
#[test]
fn hostile_finite_prosody_knobs_render_finite() {
    let candidates: [(&str, u32, Vec<f32>); 3] = [
        // declination start/end ~0 → F0 ~0 at the engine.
        ("ib", 2, vec![-7.3e-39, 0.0, 1.0, 0.0, 1.0, 25.0, 1.0]),
        // huge negative stress amplitude factor on a stressed word.
        (
            "coi munje",
            0,
            vec![120.0, 95.0, 1.5, 20.0, -1.1e29, 25.0, 1.0],
        ),
        // huge xu rise (flags=2 sets xu_rise).
        (
            "xu do klama",
            2,
            vec![120.0, 95.0, 1.5, 20.0, 1.2, 3.8e31, 1.0],
        ),
    ];
    for (text, flags, block) in candidates {
        let pcm = synth(text, flags, SR, &block).unwrap();
        let bad = pcm.iter().filter(|s| !s.is_finite()).count();
        assert_eq!(
            bad, 0,
            "{bad} non-finite samples for {text:?} (flags {flags}, block {block:?})"
        );
    }
}

proptest! {
    // Direct persistence: the default SourceParallel lookup fails for this
    // integration-test target ("failed to find lib.rs or main.rs"), silently
    // dropping failing seeds — the deep-fuzz finding had to be re-minimized.
    #![proptest_config(Config {
        cases: cases(),
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "tests/proptest-regressions/fuzz.txt",
        ))),
        ..Config::default()
    })]

    /// Totality + finiteness: any flag bits, any f32 block (NaN/inf included),
    /// any short soup — synth returns Ok or a typed error, never panics, and
    /// Ok PCM is entirely finite.
    #[test]
    fn synth_is_total_on_hostile_blocks(
        text in short_soup(),
        flags in 0u32..16,
        block in prop::collection::vec(any::<f32>(), 0..=FULL_PARAM_COUNT + 8),
    ) {
        if let Ok(pcm) = synth(&text, flags, SR, &block) {
            prop_assert!(
                pcm.iter().all(|s| s.is_finite()),
                "non-finite sample in {:?} (flags {})", text, flags
            );
        }
    }

    /// The demo contract: the phonetics line appears exactly for the texts
    /// that speak. transcription() and synth() must agree Ok/Err for every
    /// flag combination.
    #[test]
    fn transcription_agrees_with_synth(text in short_soup(), flags in 0u32..16) {
        prop_assert_eq!(
            transcription(&text, flags).is_ok(),
            synth(&text, flags, SR, &[]).is_ok(),
            "transcription/synth disagree on {:?} (flags {})", text, flags
        );
    }
}
