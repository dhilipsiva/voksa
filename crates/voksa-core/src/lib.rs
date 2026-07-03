//! voksa-core: rule-based Lojban TTS engine core.
//!
//! `no_std` + `alloc`; must stay free of `std` so it can target
//! `wasm32-unknown-unknown` inside an AudioWorklet.

#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod classify;
pub mod clusters;
pub mod compiler;
pub mod letters;
pub mod normalize;
pub mod pause;
pub mod phonemes;
pub mod prosody;
pub mod schedule;
pub mod stress;
pub mod syllable;
pub mod word;

use alloc::format;
use alloc::string::String;

/// Engine version, sourced from the workspace package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Trivial greeting — exercises `alloc` end to end.
pub fn greet(name: &str) -> String {
    format!("coi {name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_allocates_and_formats() {
        assert_eq!(greet("munje"), "coi munje");
    }
}
