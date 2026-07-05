//! C1 wave 2: export is delta-only with the CLI schema (stops nest
//! closure/burst), load is REPLACE with widening, presets reset + override.

use voksa_console::model::{
    Descriptors, ExportInputs, Flags, PARAM_TOTAL, apply_preset, export, load, widen_for,
};

/// The design bundle's demo config, verbatim (docs/design/tuning-console/
/// voksa-engine-data.json → demoConfig) — includes the deliberately
/// out-of-range `a.f1_hz = 1400` that must widen, never clamp.
const DEMO_CONFIG: &str = r#"{
  "text": "coi la djan. cu klama",
  "xu": false, "dotside": false, "buffer": false, "flat": false,
  "rate": 1.15,
  "flutter": 40,
  "declination_start_hz": 132,
  "attitudinals": { "ui": { "f0_mean_hz": 31 }, "uu": { "aspiration": 0.28, "rate_mult": 1.3 } },
  "phonemes": { "a": { "f1_hz": 1400 }, "s": { "dur_ms": 96 }, "k": { "burst": { "amp3": 1.0 }, "closure_ms": 52 } },
  "notes": "4/5 — la .alis.: stress reads natural; s a touch long; pushed a's F1 past the slider for testing",
  "sampleRate": 48000,
  "voksaVersion": "0.1.0"
}"#;

fn desc() -> Descriptors {
    Descriptors::from_defaults(&voksa_web::default_params()).unwrap()
}

fn defaults() -> Vec<f32> {
    voksa_web::default_params()
}

fn exported(values: &[f32]) -> serde_json::Value {
    let d = desc();
    let json = export(
        &d,
        &ExportInputs {
            values,
            text: "coi munje",
            flags: Flags::default(),
            notes: "note text",
            phonetics: "coi MUN.je",
            sample_rate: 48_000,
        },
    );
    serde_json::from_str(&json).expect("export emits valid JSON")
}

#[test]
fn export_of_defaults_is_flat_knobs_only() {
    let v = exported(&defaults());
    let obj = v.as_object().unwrap();
    assert_eq!(v["text"], "coi munje");
    for f in ["flat", "xu", "dotside", "buffer"] {
        assert_eq!(v[f], false);
    }
    // All 16 knobs present as flat named fields, at their engine defaults.
    assert_eq!(v["rate"], 1.0);
    assert_eq!(v["declination_start_hz"], 120.0);
    assert_eq!(v["flutter"], 25.0);
    assert_eq!(v["undershoot"].as_f64().unwrap() as f32, 0.35);
    // Delta sections absent when nothing deviates.
    assert!(!obj.contains_key("attitudinals"), "no attitudinal deltas");
    assert!(!obj.contains_key("phonemes"), "no phoneme deltas");
    // Stamps.
    assert_eq!(v["phonetics"], "coi MUN.je");
    assert_eq!(v["notes"], "note text");
    assert_eq!(v["sampleRate"], 48_000);
    assert_eq!(v["voksaVersion"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn export_deltas_are_delta_only_and_stops_nest() {
    let mut values = defaults();
    values[63] = 1400.0; // a.f1_hz (widened territory)
    values[218] = 1.0; // k burst amp3
    values[221] = 52.0; // k closure_ms
    values[7] = 31.0; // ui f0_mean_hz
    let v = exported(&values);
    assert_eq!(v["phonemes"]["a"]["f1_hz"], 1400.0);
    assert_eq!(v["phonemes"]["k"]["burst"]["amp3"], 1.0);
    assert_eq!(v["phonemes"]["k"]["closure_ms"], 52.0);
    assert!(
        v["phonemes"]["k"].get("closure").is_none(),
        "untouched closure block stays absent"
    );
    assert!(
        v["phonemes"].get("s").is_none(),
        "untouched phonemes absent"
    );
    assert_eq!(v["attitudinals"]["ui"]["f0_mean_hz"], 31.0);
    assert_eq!(
        v["attitudinals"]["ui"].as_object().unwrap().len(),
        1,
        "only the dirty field exports"
    );
    assert!(v["attitudinals"].get("uu").is_none());
}

#[test]
fn demo_config_loads_as_replace_with_widening() {
    let d = desc();
    let plan = load(&d, DEMO_CONFIG).expect("demo config parses");
    assert_eq!(plan.values.len(), PARAM_TOTAL);
    assert_eq!(plan.text.as_deref(), Some("coi la djan. cu klama"));
    assert_eq!(plan.flags, Flags::default());
    assert!(plan.notes.starts_with("4/5"));
    // Knobs.
    assert_eq!(plan.values[6], 1.15);
    assert_eq!(plan.values[440], 40.0);
    assert_eq!(plan.values[0], 132.0);
    assert_eq!(
        plan.values[1], 95.0,
        "absent knob stays at the engine default"
    );
    // Attitudinal deltas.
    assert_eq!(plan.values[7], 31.0); // ui f0_mean
    assert_eq!(plan.values[22], 0.28); // uu aspiration
    assert_eq!(plan.values[17], 1.3); // uu rate_mult
    assert_eq!(
        plan.values[8], 1.4,
        "untouched ui field keeps its pinned default"
    );
    // Phoneme deltas, incl. the out-of-range F1 that must widen.
    assert_eq!(plan.values[63], 1400.0);
    assert!(widen_for(d.get(63), plan.values[63]).is_some());
    assert_eq!(plan.values[330], 96.0); // s dur_ms
    assert_eq!(plan.values[218], 1.0); // k burst amp3
    assert_eq!(plan.values[221], 52.0); // k closure_ms
    assert_eq!(
        plan.values[64], 90.0,
        "untouched a.bw1 keeps the engine default"
    );
}

#[test]
fn load_minimal_config_defaults_everything_else() {
    let d = desc();
    let plan = load(&d, r#"{"text":"coi","rate":2.0}"#).unwrap();
    assert_eq!(plan.text.as_deref(), Some("coi"));
    assert_eq!(plan.flags, Flags::default(), "absent flags = false");
    assert_eq!(plan.notes, "");
    for (i, &d) in defaults().iter().enumerate() {
        let want = if i == 6 { 2.0 } else { d };
        assert_eq!(plan.values[i], want, "REPLACE semantics at {i}");
    }
}

#[test]
fn load_rejects_malformed_json_and_ignores_unknown_keys() {
    let d = desc();
    assert!(load(&d, "not json").is_err());
    assert!(
        load(&d, r#"{"rate": "fast"}"#).is_err(),
        "non-numeric knob errors"
    );
    let plan = load(&d, r#"{"some_future_key": 5, "rate": 1.5}"#).unwrap();
    assert_eq!(
        plan.values[6], 1.5,
        "unknown keys ignored, known keys applied"
    );
}

#[test]
fn presets_reset_everything_then_override_knobs() {
    let d = desc();
    let plan = apply_preset(&d, "Naturalness off").expect("known preset");
    assert_eq!(
        plan.0.len(),
        PARAM_TOTAL,
        "a preset plan rewrites every cell"
    );
    let val = |idx: usize| plan.0.iter().find(|(i, _)| *i == idx).unwrap().1;
    assert_eq!(val(440), 0.0, "flutter → identity");
    assert_eq!(val(446), 1.0, "final_lengthen → identity");
    assert_eq!(val(6), 1.0, "non-override knob → engine default");
    assert_eq!(val(63), 730.0, "voice table → engine default");
    let sing = apply_preset(&d, "Sing-song").unwrap();
    let val = |idx: usize| sing.0.iter().find(|(i, _)| *i == idx).unwrap().1;
    assert_eq!(val(0), 150.0);
    assert_eq!(val(440), 25.0, "naturalness stays at the release default");
    assert!(apply_preset(&d, "No Such Preset").is_none());
}
