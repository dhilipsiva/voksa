//! C1 wave 1: the path space and descriptor table are a total, exact mapping
//! onto the frozen 449-float layout, and widen/dirty follow the design
//! contracts. Ground truth: flat index 400 == /n/'s voicing slot — the same
//! index the P11 deep-fuzz finding minimized to.

use std::str::FromStr;

use voksa_console::model::{
    ATT_FIELD_COUNT, ATT_KIND_COUNT, Descriptors, KNOB_COUNT, PARAM_TOTAL, Path, VOICE_ITEM_COUNT,
    is_dirty, widen_for,
};

fn desc() -> Descriptors {
    Descriptors::from_defaults(&voksa_web::default_params()).expect("engine block is 449 floats")
}

#[test]
fn path_flat_index_hits_the_frozen_layout() {
    assert_eq!(PARAM_TOTAL, 449);
    assert_eq!(
        (
            KNOB_COUNT,
            ATT_KIND_COUNT,
            ATT_FIELD_COUNT,
            VOICE_ITEM_COUNT
        ),
        (16, 7, 8, 41)
    );
    // Knobs: 7 prosody at 0..7, 9 naturalness appended at 440..449.
    assert_eq!(Path::Knob(0).flat_index(), 0);
    assert_eq!(Path::Knob(6).flat_index(), 6);
    assert_eq!(Path::Knob(7).flat_index(), 440);
    assert_eq!(Path::Knob(15).flat_index(), 448);
    // Attitudinals: 7 + kind*8 + field.
    assert_eq!(Path::Att { kind: 0, field: 0 }.flat_index(), 7);
    assert_eq!(Path::Att { kind: 1, field: 7 }.flat_index(), 22);
    assert_eq!(Path::Att { kind: 6, field: 4 }.flat_index(), 59);
    // Voice: 63 + item prefix + slot, in VoiceTable::to_array order.
    assert_eq!(Path::Voice { item: 0, slot: 0 }.flat_index(), 63); // a.f1
    assert_eq!(Path::Voice { item: 6, slot: 0 }.flat_index(), 135); // ai.dur
    assert_eq!(Path::Voice { item: 22, slot: 0 }.flat_index(), 151); // p closure f1
    assert_eq!(Path::Voice { item: 24, slot: 19 }.flat_index(), 218); // k burst amp3
    assert_eq!(Path::Voice { item: 24, slot: 22 }.flat_index(), 221); // k closure_ms
    assert_eq!(Path::Voice { item: 30, slot: 11 }.flat_index(), 330); // s dur_ms
    assert_eq!(Path::Voice { item: 36, slot: 9 }.flat_index(), 400); // n voicing (fuzz ground truth)
    assert_eq!(Path::Voice { item: 39, slot: 0 }.flat_index(), 427); // ' dur
    assert_eq!(Path::Voice { item: 40, slot: 11 }.flat_index(), 439); // buffer dur
}

#[test]
fn path_from_flat_is_a_total_bijection() {
    for idx in 0..PARAM_TOTAL {
        let p = Path::from_flat(idx);
        assert_eq!(p.flat_index(), idx, "flat round-trip at {idx} via {p:?}");
    }
}

#[test]
fn path_strings_round_trip() {
    assert_eq!(Path::Knob(6).to_string(), "k.rate");
    assert_eq!(Path::Knob(7).to_string(), "k.flutter");
    assert_eq!(Path::Att { kind: 0, field: 3 }.to_string(), "a.ui.3");
    assert_eq!(Path::Att { kind: 6, field: 4 }.to_string(), "a.o'onai.4");
    assert_eq!(Path::Voice { item: 0, slot: 5 }.to_string(), "v.a.5");
    assert_eq!(Path::Voice { item: 39, slot: 0 }.to_string(), "v.'.0");
    assert_eq!(
        Path::Voice { item: 40, slot: 11 }.to_string(),
        "v.buffer.11"
    );
    for idx in 0..PARAM_TOTAL {
        let p = Path::from_flat(idx);
        let s = p.to_string();
        assert_eq!(Path::from_str(&s), Ok(p), "string round-trip for {s}");
    }
    assert!(Path::from_str("k.nope").is_err());
    assert!(Path::from_str("v.a.99").is_err());
    assert!(Path::from_str("x.y.z").is_err());
}

#[test]
fn descriptors_cover_the_layout_with_engine_defaults() {
    let d = desc();
    assert_eq!(d.len(), 449);
    // Labels/units from the design bundle; defaults from the ENGINE.
    assert_eq!(d.get(0).label, "pitch start");
    assert_eq!(d.get(0).unit, "Hz");
    assert_eq!(d.get(0).default, 120.0);
    assert_eq!(d.get(6).label, "speaking rate");
    assert_eq!(d.get(440).label, "pitch flutter");
    assert_eq!(
        d.get(440).default,
        25.0,
        "N-D release value, via the engine"
    );
    assert_eq!(d.get(448).default, 0.35);
    assert_eq!(d.get(63).default, 730.0, "/a/ F1 seed");
    assert_eq!(d.get(63).min, 100.0);
    assert_eq!(d.get(63).max, 1200.0);
    // Stop slot labels carry the phase prefix; timing slots keep their own.
    assert_eq!(d.get(151).label, "closure F1");
    assert_eq!(d.get(151 + 11).label, "burst F1");
    assert_eq!(d.get(63 + 136 + 22).label, "closure");
    assert_eq!(d.get(63 + 136 + 23).label, "burst");
    // Help keys resolve to the shared registry keys.
    assert_eq!(d.get(0).help_key, "prosody.declination_start_hz");
    assert_eq!(d.get(440).help_key, "naturalness.flutter");
    assert_eq!(d.get(7).help_key, "att.fields.f0_mean_hz");
    assert_eq!(d.get(63).help_key, "vt.fields.f1_hz");
    assert_eq!(d.get(218).help_key, "vt.fields.amp3");
    assert_eq!(d.get(135).help_key, "vt.fields.dur_ms");
    // Spans.
    assert_eq!(d.voice_item_range(0), 63..75);
    assert_eq!(d.voice_item_range(6), 135..136);
    assert_eq!(d.voice_item_range(22), 151..175);
    assert_eq!(d.voice_item_range(40), 428..440);
    assert_eq!(d.att_range(0), 7..15);
    assert_eq!(d.att_range(6), 55..63);
    assert_eq!(d.knob_index(6), 6);
    assert_eq!(d.knob_index(7), 440);
    // Every descriptor's path agrees with its index.
    for idx in 0..d.len() {
        assert_eq!(d.get(idx).idx, idx);
        assert_eq!(d.get(idx).path.flat_index(), idx);
    }
}

#[test]
fn from_defaults_rejects_wrong_block_lengths() {
    assert!(Descriptors::from_defaults(&[0.0; 440]).is_err());
    assert!(Descriptors::from_defaults(&[]).is_err());
}

#[test]
fn widen_never_clamp() {
    let d = desc();
    let a_f1 = d.get(63); // min 100, max 1200
    let w = widen_for(a_f1, 1400.0).expect("outside → widened");
    assert_eq!((w.min, w.max), (100.0, 1400.0));
    let w = widen_for(a_f1, 50.0).expect("below → widened");
    assert_eq!((w.min, w.max), (50.0, 1200.0));
    assert_eq!(widen_for(a_f1, 800.0), None, "inside → no widen entry");
    assert_eq!(widen_for(a_f1, 1200.0), None, "boundary is inside");
}

#[test]
fn filter_plan_keeps_only_changing_writes() {
    use voksa_console::model::{WritePlan, filter_plan};
    let current = vec![1.0f32, 2.0, 3.0];
    let plan = WritePlan(vec![(0, 1.0), (1, 2.5), (2, 3.0)]);
    assert_eq!(
        filter_plan(&current, &plan),
        WritePlan(vec![(1, 2.5)]),
        "skip-if-equal keeps signal writes minimal"
    );
}

#[test]
fn reset_plans_cover_their_scope_with_defaults() {
    use voksa_console::model::reset_plan;
    let d = desc();
    // A single voice item: k's whole 24-slot span back to engine defaults.
    let plan = reset_plan(&d, d.voice_item_range(24));
    assert_eq!(plan.0.len(), 24);
    assert_eq!(plan.0[0], (151 + 48, 500.0), "k closure f1 default");
    assert_eq!(plan.0[22], (221, 60.0), "k closure_ms default");
    // An emotion's 8 fields.
    let plan = reset_plan(&d, d.att_range(0));
    assert_eq!(plan.0.len(), 8);
    assert_eq!(plan.0[0], (7, 14.0), "ui f0_mean pinned default");
}

#[test]
fn dirty_is_f32_exact() {
    let d = desc();
    let a_f1 = d.get(63);
    assert!(!is_dirty(a_f1, 730.0));
    assert!(is_dirty(a_f1, 730.5));
    let rate = d.get(6);
    assert!(!is_dirty(rate, 1.0));
    assert!(is_dirty(rate, 1.0000001));
}
