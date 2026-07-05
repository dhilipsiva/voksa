//! Phase-11 W1 stable fuzzing of the browser surface: `synth` (the whole
//! decode → compile → prosody → render path) must be total on hostile f32
//! param blocks (NaN/inf — the decode filters per index) and produce finite
//! PCM whenever it succeeds. Render-bound, so the case count is CAPPED at
//! 1024 even under `cargo xtask fuzz` (deep runs report the cap) and renders
//! run at 8 kHz on short texts. Native-only, like tests/sentences.rs.
#![cfg(not(target_arch = "wasm32"))]

use proptest::prelude::*;
use proptest::test_runner::Config;
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

proptest! {
    #![proptest_config(Config { cases: cases(), ..Config::default() })]

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
