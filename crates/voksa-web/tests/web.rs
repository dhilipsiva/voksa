//! Browser tests: run the shared `synth` on the real wasm float path via
//! `wasm-pack test --headless --chrome crates/voksa-web`. Guarded to wasm32 so
//! native `cargo nextest` (which covers the same logic in src/ unit tests)
//! skips this file.
#![cfg(target_arch = "wasm32")]

use voksa_web::{FLAG_FLAT, synth};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn coi_munje_finite_nonempty() {
    let s = synth("coi munje", 0, 48_000, &[]).expect("synth ok");
    assert!(!s.is_empty());
    assert!(s.iter().all(|x| x.is_finite()));
}

#[wasm_bindgen_test]
fn flat_vs_prosodic_differ() {
    assert_ne!(
        synth("coi munje", FLAG_FLAT, 48_000, &[]).unwrap(),
        synth("coi munje", 0, 48_000, &[]).unwrap()
    );
}

#[wasm_bindgen_test]
fn empty_text_errors() {
    assert!(synth("", 0, 48_000, &[]).is_err());
}

#[wasm_bindgen_test]
fn rate_param_changes_length() {
    let base = synth("mi tavla do", 0, 48_000, &[]).unwrap();
    let fast = synth(
        "mi tavla do",
        0,
        48_000,
        &[120.0, 95.0, 1.5, 20.0, 1.2, 25.0, 2.0],
    )
    .unwrap();
    assert!(fast.len() < base.len());
}
