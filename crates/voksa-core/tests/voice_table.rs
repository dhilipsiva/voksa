//! D2b acceptance: the per-phoneme acoustic table is a RUNTIME parameter.
//! `VoiceTable::default()` must be byte-identical to the pinned
//! docs/formants.md seeds (so every snapshot stays valid), and a tuned table
//! must actually change the compiled schedule.

use voksa_core::compiler::{CompileOptions, compile, compile_with};
use voksa_core::phonemes::{
    Consonant, DIPHTHONGS, Phoneme, STOP_ORDER, VoiceTable, Vowel, buffer_spec, buffer_spec_with,
    spec, spec_with,
};

#[test]
fn default_table_matches_formants_md_spotchecks() {
    // Independent oracle: literals from docs/formants.md (NOT read back
    // through spec(), so this cannot become tautological).
    let t = VoiceTable::default();
    let a = t.vowels[Vowel::A.index()];
    assert_eq!(a.targets.formants[0].freq_hz, 730.0);
    assert_eq!(a.targets.formants[1].freq_hz, 1090.0);
    assert_eq!(a.targets.formants[2].freq_hz, 2440.0);
    assert_eq!(a.dur_ms, 160.0);
    let y = t.vowels[Vowel::Y.index()];
    assert_eq!(y.dur_ms, 100.0);
    let ti = STOP_ORDER.iter().position(|&c| c == Consonant::T).unwrap();
    assert_eq!(t.stops[ti].burst.formants[2].freq_hz, 3500.0);
    assert_eq!(t.stops[ti].burst.formants[2].bw_hz, 1000.0);
    assert_eq!(t.stops[ti].burst.formants[2].amp, 0.85);
    assert_eq!(t.stops[ti].closure_ms, 60.0);
    assert_eq!(t.stops[ti].burst_ms, 25.0);
    assert!(t.diphthong_dur_ms.iter().all(|&d| d == 200.0));
    assert_eq!(t.h_dur_ms, 70.0);
    assert_eq!(t.buffer.targets.formants[0].freq_hz, 400.0);
    assert_eq!(t.buffer.targets.formants[1].freq_hz, 1900.0);
    assert_eq!(t.buffer.targets.formants[2].freq_hz, 2600.0);
    assert_eq!(t.buffer.dur_ms, 35.0);
}

#[test]
fn voice_array_round_trip() {
    // The 377-float crossing must be lossless.
    let t = VoiceTable::default();
    assert_eq!(VoiceTable::from_array(t.to_array()), t);
}

#[test]
fn spec_with_default_equals_spec_for_every_phoneme() {
    // Byte-identity guard: after the table-reading implementation lands, the
    // default table must reproduce the pinned spec() for the whole inventory.
    let t = VoiceTable::default();
    for v in Vowel::ALL {
        assert_eq!(spec_with(Phoneme::Vowel(v), &t), spec(Phoneme::Vowel(v)));
    }
    for c in Consonant::ALL {
        assert_eq!(
            spec_with(Phoneme::Consonant(c), &t),
            spec(Phoneme::Consonant(c))
        );
    }
    for (a, b) in DIPHTHONGS {
        assert_eq!(
            spec_with(Phoneme::Diphthong(a, b), &t),
            spec(Phoneme::Diphthong(a, b))
        );
    }
    assert_eq!(spec_with(Phoneme::H, &t), spec(Phoneme::H));
    assert_eq!(buffer_spec_with(&t), buffer_spec());
}

#[test]
fn compile_with_default_equals_compile() {
    // The whole compile pipeline is byte-identical under the default table
    // (buffer flag on, so buffer_spec_with is covered too).
    let opts = CompileOptions {
        buffer: true,
        ..Default::default()
    };
    assert_eq!(
        compile_with("mi nelci le zdani", &opts, &VoiceTable::default()).unwrap(),
        compile("mi nelci le zdani", &opts).unwrap(),
    );
}

#[test]
fn tuned_vowel_changes_schedule() {
    // The community's core power: retuning /a/'s F1 must change every
    // a-bearing schedule.
    let opts = CompileOptions::default();
    let mut tuned = VoiceTable::default();
    tuned.vowels[Vowel::A.index()].targets.formants[0].freq_hz = 900.0;
    assert_ne!(
        compile_with("mi klama", &opts, &tuned).unwrap(),
        compile("mi klama", &opts).unwrap(),
        "a tuned /a/ F1 must change the compiled schedule"
    );
}

#[test]
fn tuned_duration_changes_timing_and_negative_is_clamped() {
    let opts = CompileOptions::default();
    // Longer /a/ stretches the utterance.
    let mut longer = VoiceTable::default();
    longer.vowels[Vowel::A.index()].dur_ms = 300.0;
    let base = compile("mi klama", &opts).unwrap();
    let long = compile_with("mi klama", &opts, &longer).unwrap();
    assert!(
        long.total_ms > base.total_ms + 100.0,
        "two /a/ at +140 ms each must lengthen the utterance: {} vs {}",
        long.total_ms,
        base.total_ms
    );
    // A negative duration (hand-edited JSON) must clamp to 0, never walk the
    // schedule backwards into unsorted spans.
    let mut negative = VoiceTable::default();
    negative.vowels[Vowel::A.index()].dur_ms = -500.0;
    let clamped = compile_with("mi klama", &opts, &negative).unwrap();
    assert!(
        clamped.total_ms > 0.0 && clamped.total_ms < base.total_ms,
        "negative durations clamp to zero-length segments"
    );
    let mut sorted = clamped.spans.clone();
    sorted.sort_by(|a, b| a.start_ms.partial_cmp(&b.start_ms).unwrap());
    assert_eq!(clamped.spans, sorted, "spans stay monotone");
}

#[test]
fn absurd_durations_are_capped() {
    // Review finding: durations were clamped below but unbounded above — a
    // loadable config with dur_ms 1e30 aborted the CLI (u32 saturation → a
    // 824 GB sample alloc). Per-segment durations must cap (identity for every
    // pinned value; the ceiling only guards hostile configs).
    let opts = CompileOptions::default();
    let mut huge = VoiceTable::default();
    huge.vowels[Vowel::A.index()].dur_ms = 1e30;
    let s = compile_with("mi klama", &opts, &huge).unwrap();
    assert!(
        s.total_ms.is_finite() && s.total_ms <= 25_000.0,
        "per-segment durations must be capped: total_ms = {}",
        s.total_ms
    );
}
