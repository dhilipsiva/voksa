//! voksa as a WebAssembly component (wasm32-wasip2, WIT world `voksa:synth`).
//!
//! A thin typed wrapper over the SAME `voksa_web::synth` / `transcription`
//! functions the browser C-ABI exports — parity by construction (ADR 0002).
//! Rust ≥ 1.82 emits a component directly for wasm32-wasip2; `cargo xtask
//! component` builds, validates, and checks WIT drift.

#![warn(missing_docs)]

// Generated bindings live in their own module: rustdoc lints don't apply to
// machine output (same reasoning as the vendored-fork exclusion).
#[allow(missing_docs)]
mod bindings {
    wit_bindgen::generate!({
        world: "voksa",
        path: "wit",
    });
}
use bindings::Guest;

struct Voksa;

impl Guest for Voksa {
    fn synthesize(text: String, flag_bits: u32, sample_rate: u32) -> Result<Vec<f32>, String> {
        voksa_web::synth(&text, flag_bits, sample_rate, &[]).map_err(|e| format!("{e:?}"))
    }

    fn transcribe(text: String, flag_bits: u32) -> Result<String, String> {
        voksa_web::transcription(&text, flag_bits).map_err(|e| format!("{e:?}"))
    }

    fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

bindings::export!(Voksa with_types_in bindings);

#[cfg(test)]
mod tests {
    use super::{Guest, Voksa};

    /// Browser parity by construction: the component surface must agree with
    /// the C-ABI surface (both delegate to voksa_web) on success and error.
    #[test]
    fn component_agrees_with_web_surface() {
        let pcm = Voksa::synthesize("coi munje".into(), 0, 48_000).expect("valid text");
        assert_eq!(pcm, voksa_web::synth("coi munje", 0, 48_000, &[]).unwrap());
        assert!(Voksa::synthesize("coi qqq!".into(), 0, 48_000).is_err());

        let line = Voksa::transcribe("coi munje".into(), 0).expect("valid text");
        assert_eq!(line, voksa_web::transcription("coi munje", 0).unwrap());
    }

    #[test]
    fn version_is_the_crate_version() {
        assert_eq!(Voksa::version(), env!("CARGO_PKG_VERSION"));
    }
}
