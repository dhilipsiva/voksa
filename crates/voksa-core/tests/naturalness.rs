//! Phase-11 levers 3–4 acceptance: microprosody (intrinsic vowel F0 +
//! obstruent perturbations) and duration rules (onset-cluster compression,
//! phrase-final lengthening) as deterministic schedule transforms — plus the
//! metadata (MicroClass, span cluster counts) they read, and the FROZEN
//! naturalness_off contract snapshot (never regenerated: the CP3 "off" arm
//! must stay the Phase-10 voice forever).

use voksa_core::compiler::{CompileOptions, compile};
use voksa_core::prosody::{MICRO_DECAY_MS, OBSTRUENT_DIP_RATIO, ProsodyOptions, apply_prosody};
use voksa_core::schedule::{BASE_F0_HZ, MicroClass, UtteranceSchedule, VowelHeight};

fn compiled(text: &str) -> UtteranceSchedule {
    compile(text, &CompileOptions::default()).unwrap_or_else(|e| panic!("{text}: {e:?}"))
}

fn compiled_buffered(text: &str) -> UtteranceSchedule {
    let opts = CompileOptions {
        buffer: true,
        ..Default::default()
    };
    compile(text, &opts).unwrap_or_else(|e| panic!("{text}: {e:?}"))
}

/// naturalness_off + FLAT legacy prosody (no declination slope, no stress
/// realization, no rate) — isolates a single naturalness knob's arithmetic.
fn isolating() -> ProsodyOptions {
    ProsodyOptions {
        declination_start_hz: BASE_F0_HZ,
        declination_end_hz: BASE_F0_HZ,
        stress_duration_factor: 1.0,
        stress_f0_excursion_hz: 0.0,
        stress_amp_factor: 1.0,
        ..ProsodyOptions::naturalness_off()
    }
}

// ---- metadata --------------------------------------------------------------

#[test]
fn events_carry_micro_classes() {
    // "mi klama": m Sonorant, i Vowel(High), k VoicelessObstruent (closure +
    // burst), l Sonorant, a Vowel(Low), m Sonorant, a Vowel(Low).
    let s = compiled("mi klama");
    let classes: Vec<MicroClass> = s.events.iter().map(|e| e.micro).collect();
    assert_eq!(
        classes,
        [
            MicroClass::Sonorant,                 // m
            MicroClass::Vowel(VowelHeight::High), // i
            MicroClass::VoicelessObstruent,       // k closure
            MicroClass::VoicelessObstruent,       // k burst
            MicroClass::Sonorant,                 // l
            MicroClass::Vowel(VowelHeight::Low),  // a
            MicroClass::Sonorant,                 // m
            MicroClass::Vowel(VowelHeight::Low),  // a
        ]
    );
}

#[test]
fn pause_events_are_silence_and_diphthongs_tagged() {
    // "coi la djan.": c, oi(2 glide events), l, a, dj…, pause after djan.
    let s = compiled("coi la djan.");
    assert!(
        s.events.iter().any(|e| e.micro == MicroClass::Silence),
        "the post-cmevla pause event must be Silence"
    );
    assert!(
        s.events.iter().any(|e| e.micro == MicroClass::Diphthong),
        "coi's oi glide events must be Diphthong"
    );
}

#[test]
fn spans_count_clusters_post_buffering() {
    // klama: KLA onset kl (2), MA onset m (1); no codas.
    let s = compiled("klama");
    assert_eq!(s.spans[0].onset_count, 2);
    assert_eq!(s.spans[0].coda_count, 0);
    assert_eq!(s.spans[1].onset_count, 1);
    // Buffered vrusi: the [ɪ] breaks vr → both remaining onsets count 1.
    let b = compiled_buffered("vrusi");
    assert!(
        b.spans.iter().all(|sp| sp.onset_count <= 1),
        "a buffered cluster must report broken (counts ≤ 1)"
    );
    // djan: coda n.
    let d = compiled("la djan.");
    let djan = d.spans.iter().find(|sp| sp.word_index == 1).unwrap();
    assert_eq!(djan.coda_count, 1);
}

// ---- lever 3: microprosody --------------------------------------------------

#[test]
fn intrinsic_vowel_f0_shifts_high_and_low() {
    let mut opts = isolating();
    opts.micro_f0_hz = 4.0;
    let s = apply_prosody(compiled("mi klama"), &opts);
    let f0_of = |idx: usize| s.events[idx].frame.f0_hz;
    assert_eq!(f0_of(1), BASE_F0_HZ + 4.0, "/i/ is High: +4");
    assert_eq!(f0_of(5), BASE_F0_HZ - 4.0, "/a/ is Low: −4");
    assert_eq!(f0_of(0), BASE_F0_HZ, "sonorants keep the baseline");
}

#[test]
fn rise_after_voiceless_onset_decays_via_settle_event() {
    // "mi ta": t (voiceless) then a — the vowel onset rises +6 and settles
    // back to its intrinsic value over MICRO_DECAY_MS via an inserted event.
    let mut opts = isolating();
    opts.micro_f0_hz = 4.0;
    opts.obstruent_f0_hz = 6.0;
    let s = apply_prosody(compiled("mi ta"), &opts);
    // Find the /a/ vowel event (last Low vowel).
    let (ai, a) = s
        .events
        .iter()
        .enumerate()
        .find(|(_, e)| e.micro == MicroClass::Vowel(VowelHeight::Low))
        .expect("ta has an /a/");
    assert_eq!(
        a.frame.f0_hz,
        BASE_F0_HZ - 4.0 + 6.0,
        "onset: intrinsic + rise"
    );
    let settle = &s.events[ai + 1];
    assert_eq!(settle.at_ms, a.at_ms + MICRO_DECAY_MS, "settle at +50 ms");
    assert_eq!(settle.transition_ms, MICRO_DECAY_MS);
    assert_eq!(
        settle.frame.f0_hz,
        BASE_F0_HZ - 4.0,
        "settles to intrinsic-only"
    );
    assert_eq!(settle.micro, a.micro);
}

#[test]
fn dip_after_voiced_obstruent() {
    // "mi ba": b (voiced) then a — dip = −OBSTRUENT_DIP_RATIO·Δ.
    let mut opts = isolating();
    opts.obstruent_f0_hz = 6.0;
    let s = apply_prosody(compiled("mi ba"), &opts);
    let a = s
        .events
        .iter()
        .find(|e| e.micro == MicroClass::Vowel(VowelHeight::Low))
        .expect("ba has an /a/");
    assert_eq!(a.frame.f0_hz, BASE_F0_HZ - OBSTRUENT_DIP_RATIO * 6.0);
}

#[test]
fn perturbation_blocked_across_pauses() {
    // "mi .a": the pause (Silence) sits between /i/ and /a/ — no perturbation
    // even though a consonant precedes further back.
    let mut opts = isolating();
    opts.micro_f0_hz = 4.0;
    opts.obstruent_f0_hz = 6.0;
    let s = apply_prosody(compiled("mi .a"), &opts);
    let a = s
        .events
        .iter()
        .find(|e| e.micro == MicroClass::Vowel(VowelHeight::Low))
        .expect("has an /a/");
    assert_eq!(
        a.frame.f0_hz,
        BASE_F0_HZ - 4.0,
        "intrinsic only after a pause"
    );
}

// ---- lever 4: duration rules -------------------------------------------------

#[test]
fn cluster_shortening_compresses_kl_onset() {
    let mut opts = isolating();
    opts.cluster_shorten = 0.15;
    let base = apply_prosody(compiled("klama"), &isolating());
    let short = apply_prosody(compiled("klama"), &opts);
    // KLA's onset window (k=2) compresses ×0.85 exactly; later material shifts.
    let (b0, s0) = (&base.spans[0], &short.spans[0]);
    assert!(
        (s0.nucleus_off_ms - b0.nucleus_off_ms * 0.85).abs() < 1e-3,
        "kl onset ×0.85: {} vs {}",
        s0.nucleus_off_ms,
        b0.nucleus_off_ms
    );
    let saved = b0.nucleus_off_ms * 0.15;
    assert!(
        (base.total_ms - short.total_ms - saved).abs() < 1e-2,
        "total shrinks by the saved onset time"
    );
    // Singleton onsets are identity.
    assert!(
        (base.spans[1].dur_ms - short.spans[1].dur_ms).abs() < 1e-3,
        "MA (k=1) must not compress"
    );
}

#[test]
fn cluster_shortening_identity_when_buffered() {
    let mut opts = isolating();
    opts.cluster_shorten = 0.15;
    let base = apply_prosody(compiled_buffered("vrusi"), &isolating());
    let short = apply_prosody(compiled_buffered("vrusi"), &opts);
    assert_eq!(base, short, "buffered clusters (counts ≤ 1) are identity");
}

#[test]
fn final_lengthening_stretches_last_rhyme_only() {
    let mut opts = isolating();
    opts.final_lengthen = 1.3;
    let base = apply_prosody(compiled("coi munje"), &isolating());
    let long = apply_prosody(compiled("coi munje"), &opts);
    let (bl, ll) = (base.spans.last().unwrap(), long.spans.last().unwrap());
    let rhyme = bl.dur_ms - bl.nucleus_off_ms;
    assert!(
        (ll.dur_ms - (bl.dur_ms + rhyme * 0.3)).abs() < 1e-2,
        "last rhyme ×1.3: {} vs {}",
        ll.dur_ms,
        bl.dur_ms
    );
    assert!(
        (long.total_ms - (base.total_ms + rhyme * 0.3)).abs() < 1e-2,
        "total grows by the added rhyme time"
    );
    // Earlier spans untouched.
    assert_eq!(base.spans[0], long.spans[0]);
}

#[test]
fn final_lengthening_composes_with_stressed_final() {
    // "la dyGOL.": explicit final stress → rhyme gets 1.5 (stress) × 1.3
    // (final) ≈ 1.95× the unstretched rhyme.
    let mut both = isolating();
    both.stress_duration_factor = 1.5;
    both.final_lengthen = 1.3;
    let base = apply_prosody(compiled("la dyGOL."), &isolating());
    let comp = apply_prosody(compiled("la dyGOL."), &both);
    let bl = base.spans.last().unwrap();
    let cl = comp.spans.last().unwrap();
    let rhyme = bl.dur_ms - bl.nucleus_off_ms;
    assert!(
        (cl.dur_ms - (bl.nucleus_off_ms + rhyme * 1.5 * 1.3)).abs() < 1e-2,
        "stress × final compose multiplicatively: {} vs {}",
        cl.dur_ms,
        bl.nucleus_off_ms + rhyme * 1.95
    );
}

// ---- lever 5: undershoot -------------------------------------------------------

#[test]
fn undershoot_centralizes_short_vowels_exactly() {
    // The buffered [ɪ] (35 ms) reduces strongly: u = 0.35·(1 − 35/200) =
    // 0.28875 → F2 = 1900 + (1500 − 1900)·u = 1784.5.
    let mut opts = isolating();
    opts.undershoot = 0.35;
    let s = apply_prosody(compiled_buffered("le zdani"), &opts);
    let buffer_ev = s
        .events
        .iter()
        .find(|e| {
            e.micro == MicroClass::Vowel(VowelHeight::Mid)
                && (e.frame.targets.formants[0].freq_hz - 400.0).abs() < 60.0
        })
        .expect("buffered zdani has the [ɪ] event");
    assert!(
        (buffer_ev.frame.targets.formants[1].freq_hz - 1784.5).abs() < 1e-2,
        "buffer F2 must centralize to 1784.5, got {}",
        buffer_ev.frame.targets.formants[1].freq_hz
    );
}

#[test]
fn undershoot_spares_long_stressed_vowels_and_diphthongs() {
    let mut opts = isolating();
    opts.stress_duration_factor = 1.5;
    opts.undershoot = 0.35;
    let s = apply_prosody(compiled("klama"), &opts);
    // KLA's /a/ rhyme post-stretch is 240 ms ≥ 200 → full quality.
    let a = s
        .events
        .iter()
        .find(|e| e.micro == MicroClass::Vowel(VowelHeight::Low))
        .expect("klama has /a/");
    assert_eq!(
        a.frame.targets.formants[0].freq_hz, 730.0,
        "stressed /a/ keeps F1"
    );
    // Diphthongs untouched (already dynamic).
    let d = apply_prosody(compiled("coi"), &opts);
    for e in d.events.iter().filter(|e| e.micro == MicroClass::Diphthong) {
        assert!(
            [570.0, 270.0].contains(&e.frame.targets.formants[0].freq_hz),
            "diphthong endpoints stay pinned"
        );
    }
}

// ---- levers 1–2: baseline voice quality ----------------------------------------

#[test]
fn naturalness_stage_sets_vq_lanes_and_breath() {
    let mut opts = isolating();
    opts.flutter = 25.0;
    opts.breath_aspiration = 0.06;
    opts.baseline_oq_delta = 0.10;
    opts.baseline_tilt_delta = -0.10;
    let s = apply_prosody(compiled("coi munje"), &opts);
    for e in &s.events {
        assert_eq!(e.frame.flutter, 25.0, "flutter set on every frame");
        assert!((e.frame.oq - 1.10).abs() < 1e-6, "oq delta on every frame");
        assert!(
            (e.frame.tilt - (-0.10)).abs() < 1e-6,
            "tilt delta on every frame"
        );
        if e.frame.targets.voicing > 0.0 {
            assert!(
                (e.frame.targets.aspiration - 0.06).abs() < 1e-6
                    || e.frame.targets.aspiration > 0.06,
                "breath lands on voiced frames"
            );
        } else if e.micro == MicroClass::Silence {
            assert_eq!(e.frame.targets.aspiration, 0.0, "silence stays breath-free");
        }
    }
}

// ---- the frozen contract ------------------------------------------------------

#[test]
fn snapshot_naturalness_off_coi_munje() {
    // FROZEN: this snapshot is the Phase-10 voice and must NEVER regenerate —
    // naturalness_off() stays byte-identical across all Phase-11 default flips.
    insta::assert_debug_snapshot!(apply_prosody(
        compiled("coi munje"),
        &ProsodyOptions::naturalness_off()
    ));
}

#[test]
fn naturalness_off_equals_default_while_identity_pinned() {
    // Until N-D flips the pinned constants, default == naturalness_off. This
    // test is REPLACED in N-D by default_options_equal_pinned_constants — it
    // documents the N-B/N-C state.
    assert_eq!(
        apply_prosody(compiled("coi munje"), &ProsodyOptions::default()),
        apply_prosody(compiled("coi munje"), &ProsodyOptions::naturalness_off()),
    );
}
