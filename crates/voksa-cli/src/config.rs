//! JSON tuning config: the round-trip the browser demo produces. `voksa
//! --config <file>` replays a submitted config exactly. Field names mirror the
//! demo's f32 param layout + flags; unknown fields (notes, sampleRate,
//! timestamp, version) are ignored.

use std::path::Path;

use serde::Deserialize;
use voksa_core::compiler::CompileOptions;
use voksa_core::prosody::ProsodyOptions;

/// A deserialized tuning config. Missing fields fall back to the defaults.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    pub text: String,
    pub flat: bool,
    pub xu: bool,
    pub dotside: bool,
    pub buffer: bool,
    pub declination_start_hz: f32,
    pub declination_end_hz: f32,
    pub stress_duration_factor: f32,
    pub stress_f0_excursion_hz: f32,
    pub stress_amp_factor: f32,
    pub xu_rise_hz: f32,
    pub rate: f32,
}

impl Default for Config {
    fn default() -> Self {
        let p = ProsodyOptions::default();
        Self {
            text: String::new(),
            flat: false,
            xu: false,
            dotside: false,
            buffer: false,
            declination_start_hz: p.declination_start_hz,
            declination_end_hz: p.declination_end_hz,
            stress_duration_factor: p.stress_duration_factor,
            stress_f0_excursion_hz: p.stress_f0_excursion_hz,
            stress_amp_factor: p.stress_amp_factor,
            xu_rise_hz: p.xu_rise_hz,
            rate: p.rate,
        }
    }
}

impl Config {
    /// Parse a config from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }

    /// Load a config from a JSON file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let s = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::from_json(&s)
    }

    pub fn compile_options(&self) -> CompileOptions {
        CompileOptions {
            dotside: self.dotside,
            buffer: self.buffer,
        }
    }

    /// Map the tunable fields onto [`ProsodyOptions`].
    pub fn prosody_options(&self) -> ProsodyOptions {
        // STUB (D1 CLI red): the real mapping lands after the failing test.
        ProsodyOptions {
            xu_rise: self.xu,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_known_fields_and_ignores_extras() {
        let json = r#"{
            "text": "coi munje", "dotside": true, "rate": 2.0,
            "declination_end_hz": 70.0,
            "notes": "sounds nice", "sampleRate": 48000, "voksaVersion": "0.0.1"
        }"#;
        let c = Config::from_json(json).expect("valid config");
        assert_eq!(c.text, "coi munje");
        assert!(c.dotside && !c.buffer);
        assert_eq!(c.rate, 2.0);
        assert_eq!(c.declination_end_hz, 70.0);
        // Unset fields keep their defaults.
        assert_eq!(c.declination_start_hz, 120.0);
    }

    #[test]
    fn maps_onto_prosody_options() {
        let json = r#"{ "xu": true, "rate": 2.0, "xu_rise_hz": 60.0, "stress_amp_factor": 1.9 }"#;
        let p = Config::from_json(json).unwrap().prosody_options();
        assert!(p.xu_rise);
        assert_eq!(p.rate, 2.0);
        assert_eq!(p.xu_rise_hz, 60.0);
        assert_eq!(p.stress_amp_factor, 1.9);
    }

    #[test]
    fn default_config_maps_to_default_prosody() {
        assert_eq!(Config::default().prosody_options(), ProsodyOptions::default());
    }
}
