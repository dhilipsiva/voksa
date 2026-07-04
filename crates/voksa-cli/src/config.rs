//! JSON tuning config: the round-trip the browser demo produces. `voksa
//! --config <file>` replays a submitted config exactly. Field names mirror the
//! demo's f32 param layout + flags; unknown fields (notes, sampleRate,
//! timestamp, version) are ignored.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use voksa_core::attitudinal::{AttitudinalKind, AttitudinalTable};
use voksa_core::compiler::CompileOptions;
use voksa_core::prosody::ProsodyOptions;

/// One attitudinal's deviation-vector overrides (D2a advanced tab). Only the
/// fields present in the JSON override the pinned vector; the rest stay
/// default. Keys mirror `voksa_core::attitudinal::Deviation` minus the `d_`
/// prefix (the demo exports these names).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct AttitudinalOverride {
    pub f0_mean_hz: Option<f32>,
    pub f0_range_mult: Option<f32>,
    pub rate_mult: Option<f32>,
    pub oq: Option<f32>,
    pub tilt: Option<f32>,
    pub di: Option<f32>,
    pub vibrato_hz: Option<f32>,
    pub aspiration: Option<f32>,
}

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
    /// Attitudinal deviation overrides keyed by cmavo (`"ui"`, `"uu"`, `"oi"`,
    /// `"ii"`, `"o'o"`, `"au"`, `"o'onai"`). Absent = pinned defaults.
    pub attitudinals: BTreeMap<String, AttitudinalOverride>,
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
            attitudinals: BTreeMap::new(),
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
        ProsodyOptions {
            xu_rise: self.xu,
            declination_start_hz: self.declination_start_hz,
            declination_end_hz: self.declination_end_hz,
            stress_duration_factor: self.stress_duration_factor,
            stress_f0_excursion_hz: self.stress_f0_excursion_hz,
            stress_amp_factor: self.stress_amp_factor,
            xu_rise_hz: self.xu_rise_hz,
            rate: self.rate,
        }
    }

    /// Build the runtime [`AttitudinalTable`]: pinned defaults with this
    /// config's per-cmavo overrides applied (only the named fields change).
    pub fn attitudinal_table(&self) -> AttitudinalTable {
        let mut table = AttitudinalTable::default();
        for kind in AttitudinalKind::ALL {
            let Some(o) = self.attitudinals.get(kind.cmavo()) else {
                continue;
            };
            let d = &mut table.deviations[kind.index()];
            macro_rules! set {
                ($json:ident, $field:ident) => {
                    if let Some(v) = o.$json {
                        d.$field = v;
                    }
                };
            }
            set!(f0_mean_hz, f0_mean_hz);
            set!(f0_range_mult, f0_range_mult);
            set!(rate_mult, rate_mult);
            set!(oq, d_oq);
            set!(tilt, d_tilt);
            set!(di, d_di);
            set!(vibrato_hz, d_vibrato_hz);
            set!(aspiration, d_aspiration);
        }
        table
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
        assert_eq!(
            Config::default().prosody_options(),
            ProsodyOptions::default()
        );
    }

    #[test]
    fn config_without_attitudinals_is_default_table() {
        assert_eq!(
            Config::from_json("{}").unwrap().attitudinal_table(),
            AttitudinalTable::default()
        );
    }

    #[test]
    fn attitudinals_block_overrides_pinned_vector() {
        // Community JSON overrides only what it names; the rest stays pinned.
        let json = r#"{
            "text": "coi munje .ui",
            "attitudinals": {
                "ui": { "f0_mean_hz": 40.0, "di": 0.3 },
                "o'onai": { "tilt": 0.5 }
            }
        }"#;
        let t = Config::from_json(json).unwrap().attitudinal_table();
        let joy = t.get(AttitudinalKind::Joy);
        assert_eq!(joy.f0_mean_hz, 40.0);
        assert_eq!(joy.d_di, 0.3);
        assert_eq!(
            joy.f0_range_mult,
            AttitudinalKind::Joy.deviation().f0_range_mult,
            "unnamed fields stay pinned"
        );
        assert_eq!(t.get(AttitudinalKind::Anger).d_tilt, 0.5);
        assert_eq!(
            t.get(AttitudinalKind::Sadness),
            AttitudinalKind::Sadness.deviation(),
            "unnamed kinds stay pinned"
        );
    }
}
