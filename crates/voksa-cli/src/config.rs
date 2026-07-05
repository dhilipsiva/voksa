//! JSON tuning config: the round-trip the browser demo produces. `voksa
//! --config <file>` replays a submitted config exactly. Field names mirror the
//! demo's f32 param layout + flags; unknown fields (notes, sampleRate,
//! timestamp, version) are ignored.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use voksa_core::attitudinal::{AttitudinalKind, AttitudinalTable};
use voksa_core::compiler::CompileOptions;
use voksa_core::phonemes::VoiceTable;
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

/// One Targets' worth of overrides (D2b): field names mirror
/// `voksa_core::phonemes::Targets::to_array` order.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct TargetsOverride {
    pub f1_hz: Option<f32>,
    pub bw1_hz: Option<f32>,
    pub amp1: Option<f32>,
    pub f2_hz: Option<f32>,
    pub bw2_hz: Option<f32>,
    pub amp2: Option<f32>,
    pub f3_hz: Option<f32>,
    pub bw3_hz: Option<f32>,
    pub amp3: Option<f32>,
    pub voicing: Option<f32>,
    pub aspiration: Option<f32>,
}

/// One phoneme's deviation from the pinned voice table (D2b advanced tab).
/// Steady phonemes (vowels, fricatives, nasals, liquids, `"buffer"`) use the
/// flattened targets fields + `dur_ms`; stops use the nested `closure`/`burst`
/// objects + `closure_ms`/`burst_ms`; diphthongs (`"ai"`…) and `"'"` ([h]) use
/// `dur_ms` only. Class-irrelevant fields simply never apply.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct PhonemeOverride {
    #[serde(flatten)]
    pub steady: TargetsOverride,
    pub dur_ms: Option<f32>,
    pub closure: TargetsOverride,
    pub burst: TargetsOverride,
    pub closure_ms: Option<f32>,
    pub burst_ms: Option<f32>,
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
    /// Per-phoneme voice overrides keyed by phoneme letter (`"a"`…`"y"`,
    /// `"p"`…`"z"`), diphthong (`"ai"`…`"uy"`, durations only), `"'"` ([h],
    /// duration only), or `"buffer"`. Absent = pinned defaults.
    pub phonemes: BTreeMap<String, PhonemeOverride>,
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
            phonemes: BTreeMap::new(),
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
            // The Phase-11 naturalness keys join the config in N-C; until
            // then they ride the defaults.
            ..Default::default()
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

    /// Build the runtime [`VoiceTable`]: pinned defaults with this config's
    /// per-phoneme overrides applied (only the named fields change).
    pub fn voice_table(&self) -> VoiceTable {
        use voksa_core::phonemes::{
            DIPHTHONGS, FRICATIVE_ORDER, LIQUID_ORDER, NASAL_ORDER, STOP_ORDER, Vowel,
        };
        let mut t = VoiceTable::default();
        for v in Vowel::ALL {
            if let Some(o) = self.phonemes.get(vowel_letter(v)) {
                o.apply_steady(&mut t.vowels[v.index()]);
            }
        }
        for (i, &(a, b)) in DIPHTHONGS.iter().enumerate() {
            let key = format!("{}{}", vowel_letter(a), vowel_letter(b));
            if let Some(d) = self.phonemes.get(&key).and_then(|o| o.dur_ms) {
                t.diphthong_dur_ms[i] = d;
            }
        }
        for (i, c) in STOP_ORDER.iter().enumerate() {
            if let Some(o) = self.phonemes.get(consonant_letter(*c)) {
                o.apply_stop(&mut t.stops[i]);
            }
        }
        for (i, c) in FRICATIVE_ORDER.iter().enumerate() {
            if let Some(o) = self.phonemes.get(consonant_letter(*c)) {
                o.apply_steady(&mut t.fricatives[i]);
            }
        }
        for (i, c) in NASAL_ORDER.iter().enumerate() {
            if let Some(o) = self.phonemes.get(consonant_letter(*c)) {
                o.apply_steady(&mut t.nasals[i]);
            }
        }
        for (i, c) in LIQUID_ORDER.iter().enumerate() {
            if let Some(o) = self.phonemes.get(consonant_letter(*c)) {
                o.apply_steady(&mut t.liquids[i]);
            }
        }
        if let Some(d) = self.phonemes.get("'").and_then(|o| o.dur_ms) {
            t.h_dur_ms = d;
        }
        if let Some(o) = self.phonemes.get("buffer") {
            o.apply_steady(&mut t.buffer);
        }
        t
    }
}

fn vowel_letter(v: voksa_core::phonemes::Vowel) -> &'static str {
    use voksa_core::phonemes::Vowel;
    match v {
        Vowel::A => "a",
        Vowel::E => "e",
        Vowel::I => "i",
        Vowel::O => "o",
        Vowel::U => "u",
        Vowel::Y => "y",
    }
}

fn consonant_letter(c: voksa_core::phonemes::Consonant) -> &'static str {
    use voksa_core::phonemes::Consonant;
    match c {
        Consonant::B => "b",
        Consonant::C => "c",
        Consonant::D => "d",
        Consonant::F => "f",
        Consonant::G => "g",
        Consonant::J => "j",
        Consonant::K => "k",
        Consonant::L => "l",
        Consonant::M => "m",
        Consonant::N => "n",
        Consonant::P => "p",
        Consonant::R => "r",
        Consonant::S => "s",
        Consonant::T => "t",
        Consonant::V => "v",
        Consonant::X => "x",
        Consonant::Z => "z",
    }
}

impl TargetsOverride {
    /// Apply the named fields onto `t` (unnamed fields stay).
    fn apply(&self, t: &mut voksa_core::phonemes::Targets) {
        macro_rules! set {
            ($json:ident, $slot:expr) => {
                if let Some(v) = self.$json {
                    $slot = v;
                }
            };
        }
        set!(f1_hz, t.formants[0].freq_hz);
        set!(bw1_hz, t.formants[0].bw_hz);
        set!(amp1, t.formants[0].amp);
        set!(f2_hz, t.formants[1].freq_hz);
        set!(bw2_hz, t.formants[1].bw_hz);
        set!(amp2, t.formants[1].amp);
        set!(f3_hz, t.formants[2].freq_hz);
        set!(bw3_hz, t.formants[2].bw_hz);
        set!(amp3, t.formants[2].amp);
        set!(voicing, t.voicing);
        set!(aspiration, t.aspiration);
    }
}

impl PhonemeOverride {
    fn apply_steady(&self, sv: &mut voksa_core::phonemes::SteadyVoice) {
        self.steady.apply(&mut sv.targets);
        if let Some(d) = self.dur_ms {
            sv.dur_ms = d;
        }
    }

    fn apply_stop(&self, st: &mut voksa_core::phonemes::StopVoice) {
        self.closure.apply(&mut st.closure);
        self.burst.apply(&mut st.burst);
        if let Some(v) = self.closure_ms {
            st.closure_ms = v;
        }
        if let Some(v) = self.burst_ms {
            st.burst_ms = v;
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

    #[test]
    fn naturalness_keys_map_onto_prosody_options() {
        // The nine Phase-11 knobs round-trip through the config JSON.
        let json = r#"{
            "flutter": 30.0, "breath_aspiration": 0.08,
            "baseline_oq_delta": 0.12, "baseline_tilt_delta": -0.2,
            "micro_f0_hz": 5.0, "obstruent_f0_hz": 7.0,
            "final_lengthen": 1.4, "cluster_shorten": 0.2, "undershoot": 0.4
        }"#;
        let p = Config::from_json(json).unwrap().prosody_options();
        assert_eq!(p.flutter, 30.0);
        assert_eq!(p.breath_aspiration, 0.08);
        assert_eq!(p.baseline_oq_delta, 0.12);
        assert_eq!(p.baseline_tilt_delta, -0.2);
        assert_eq!(p.micro_f0_hz, 5.0);
        assert_eq!(p.obstruent_f0_hz, 7.0);
        assert_eq!(p.final_lengthen, 1.4);
        assert_eq!(p.cluster_shorten, 0.2);
        assert_eq!(p.undershoot, 0.4);
    }

    #[test]
    fn config_without_phonemes_is_default_table() {
        assert_eq!(
            Config::from_json("{}").unwrap().voice_table(),
            VoiceTable::default()
        );
    }

    #[test]
    fn phonemes_block_overrides_pinned_table() {
        // Community JSON overrides only what it names; the rest stays pinned.
        use voksa_core::phonemes::{Consonant, STOP_ORDER, Vowel};
        let json = r#"{
            "text": "mi klama",
            "phonemes": {
                "a":  { "f1_hz": 900.0, "dur_ms": 200.0 },
                "t":  { "burst": { "f3_hz": 3200.0 }, "closure_ms": 45.0 },
                "ai": { "dur_ms": 260.0 },
                "'":  { "dur_ms": 90.0 }
            }
        }"#;
        let t = Config::from_json(json).unwrap().voice_table();
        let a = t.vowels[Vowel::A.index()];
        assert_eq!(a.targets.formants[0].freq_hz, 900.0);
        assert_eq!(a.dur_ms, 200.0);
        assert_eq!(
            a.targets.formants[1].freq_hz, 1090.0,
            "unnamed fields stay pinned"
        );
        let ti = STOP_ORDER.iter().position(|&c| c == Consonant::T).unwrap();
        assert_eq!(t.stops[ti].burst.formants[2].freq_hz, 3200.0);
        assert_eq!(t.stops[ti].closure_ms, 45.0);
        assert_eq!(t.stops[ti].burst_ms, 25.0, "unnamed timing stays pinned");
        assert_eq!(t.diphthong_dur_ms[0], 260.0, "ai is DIPHTHONGS[0]");
        assert_eq!(t.h_dur_ms, 90.0);
        assert_eq!(
            t.vowels[Vowel::E.index()],
            VoiceTable::default().vowels[Vowel::E.index()],
            "unnamed phonemes stay pinned"
        );
    }
}
