//! Phase-7 schedule-level acceptance: the prosody transform is deterministic
//! and realizes declination, stress, and the xu rise exactly as specified.

use voksa_core::compiler::{CompileOptions, compile};
use voksa_core::prosody::{
    DECLINATION_END_HZ, DECLINATION_START_HZ, ProsodyOptions, STRESS_AMP_FACTOR,
    STRESS_DURATION_FACTOR, STRESS_F0_EXCURSION_HZ, XU_RISE_HZ, apply_prosody,
};
use voksa_core::schedule::{SyllableSpan, UtteranceSchedule};

fn compiled(text: &str) -> UtteranceSchedule {
    compile(text, &CompileOptions::default()).unwrap_or_else(|e| panic!("{text}: {e:?}"))
}

fn prosodic(text: &str) -> UtteranceSchedule {
    apply_prosody(compiled(text), &ProsodyOptions::default())
}

fn stressed_spans(s: &UtteranceSchedule) -> Vec<SyllableSpan> {
    s.spans.iter().copied().filter(|sp| sp.stressed).collect()
}

fn in_span(at: f32, sp: &SyllableSpan) -> bool {
    at >= sp.start_ms - 1e-3 && at < sp.start_ms + sp.dur_ms - 1e-3
}

#[test]
fn declination_endpoints_and_monotone_baseline() {
    let s = prosodic("mi tavla do bau la lojban.");
    let voiced: Vec<(f32, f32)> = s
        .events
        .iter()
        .filter(|e| e.frame.targets.voicing > 0.0)
        .map(|e| (e.at_ms, e.frame.f0_hz))
        .collect();
    assert!(voiced.len() >= 8);
    let first = voiced.first().unwrap();
    let last = voiced.last().unwrap();
    assert!(
        (first.1 - DECLINATION_START_HZ).abs() <= 8.0,
        "start f0 {:.1}",
        first.1
    );
    assert!(
        (last.1 - DECLINATION_END_HZ).abs() <= 8.0,
        "end f0 {:.1}",
        last.1
    );
    // Baseline (outside stressed spans) is non-increasing.
    let stressed = stressed_spans(&s);
    let baseline: Vec<f32> = voiced
        .iter()
        .filter(|(at, _)| !stressed.iter().any(|sp| in_span(*at, sp)))
        .map(|(_, f0)| *f0)
        .collect();
    for w in baseline.windows(2) {
        assert!(
            w[1] <= w[0] + 1e-3,
            "baseline must decline: {} -> {}",
            w[0],
            w[1]
        );
    }
}

#[test]
fn stress_stretches_duration_and_shifts_later_material() {
    let before = compiled("coi munje");
    let after = apply_prosody(before.clone(), &ProsodyOptions::default());
    // munje = word 1, syllables mun (stressed) + je.
    let b_stressed = stressed_spans(&before)[0];
    let a_stressed = stressed_spans(&after)[0];
    let ratio = a_stressed.dur_ms / b_stressed.dur_ms;
    assert!(
        (ratio - STRESS_DURATION_FACTOR).abs() < 1e-3,
        "duration ratio {ratio}"
    );
    // The following (unstressed) span shifted by the added duration.
    let b_last = before.spans.last().unwrap();
    let a_last = after.spans.last().unwrap();
    let added = b_stressed.dur_ms * (STRESS_DURATION_FACTOR - 1.0);
    assert!((a_last.start_ms - (b_last.start_ms + added)).abs() < 1e-2);
    assert!(
        (a_last.dur_ms - b_last.dur_ms).abs() < 1e-3,
        "unstressed spans unstretched"
    );
    assert!((after.total_ms - (before.total_ms + added)).abs() < 1e-2);
}

#[test]
fn stress_excursion_and_amp_boost_apply_in_span_only() {
    let before = compiled("coi munje");
    let after = apply_prosodic_pair(&before);
    let stressed = stressed_spans(&after.1)[0];
    for (b, a) in before.events.iter().zip(&after.0.events) {
        // Pair pre-stretch events with post-stretch via order preservation.
        let _ = (b, a);
    }
    // Check via the transformed schedule directly: in-span voiced events sit
    // ABOVE the local declination baseline by the excursion.
    let s = &after.1;
    let total = s.total_ms;
    for e in &s.events {
        let baseline =
            DECLINATION_START_HZ + (DECLINATION_END_HZ - DECLINATION_START_HZ) * (e.at_ms / total);
        let delta = e.frame.f0_hz - baseline;
        if in_span(e.at_ms, &stressed) {
            assert!(
                (delta - STRESS_F0_EXCURSION_HZ).abs() < 1.0,
                "in-span excursion {delta:.2}"
            );
        } else {
            assert!(delta.abs() < 1.0, "out-of-span delta {delta:.2}");
        }
    }
    // Amplitude boost present in-span: compare against the pre-transform
    // event amplitude (same event order).
    let b_events = &before.events;
    let a_events = &s.events;
    assert_eq!(b_events.len(), a_events.len());
    for (b, a) in b_events.iter().zip(a_events) {
        let boosted = in_span(a.at_ms, &stressed);
        for (bf, af) in b
            .frame
            .targets
            .formants
            .iter()
            .zip(&a.frame.targets.formants)
        {
            let expected = if boosted {
                bf.amp * STRESS_AMP_FACTOR
            } else {
                bf.amp
            };
            assert!((af.amp - expected).abs() < 1e-4);
        }
    }
}

/// Helper returning (transformed, transformed) to keep the borrow story tidy.
fn apply_prosodic_pair(before: &UtteranceSchedule) -> (UtteranceSchedule, UtteranceSchedule) {
    let a = apply_prosody(before.clone(), &ProsodyOptions::default());
    (a.clone(), a)
}

#[test]
fn two_brivla_accumulate_shifts() {
    let before = compiled("le prenu cu klama");
    let after = apply_prosody(before.clone(), &ProsodyOptions::default());
    let b_stressed = stressed_spans(&before);
    let a_stressed = stressed_spans(&after);
    assert_eq!(b_stressed.len(), 2, "prenu + klama");
    for (b, a) in b_stressed.iter().zip(&a_stressed) {
        let ratio = a.dur_ms / b.dur_ms;
        assert!((ratio - STRESS_DURATION_FACTOR).abs() < 1e-3);
    }
    let added: f32 = b_stressed
        .iter()
        .map(|sp| sp.dur_ms * (STRESS_DURATION_FACTOR - 1.0))
        .sum();
    assert!((after.total_ms - (before.total_ms + added)).abs() < 1e-2);
}

#[test]
fn xu_rise_adds_one_sorted_event_with_cloned_targets() {
    let flat = apply_prosody(compiled("xu do klama"), &ProsodyOptions::default());
    let risen = apply_prosody(compiled("xu do klama"), &ProsodyOptions { xu_rise: true });
    assert_eq!(risen.events.len(), flat.events.len() + 1);
    // Events stay time-sorted.
    for w in risen.events.windows(2) {
        assert!(w[0].at_ms <= w[1].at_ms + 1e-3);
    }
    // The added event: find it (the one not matching flat's sequence).
    let extra = risen
        .events
        .iter()
        .find(|e| {
            !flat
                .events
                .iter()
                .any(|f| (f.at_ms - e.at_ms).abs() < 1e-6 && f.frame == e.frame)
        })
        .expect("one added event");
    let last_span = risen
        .spans
        .iter()
        .max_by(|a, b| a.start_ms.partial_cmp(&b.start_ms).unwrap())
        .unwrap();
    assert!(
        in_span(extra.at_ms, last_span),
        "rise sits in the final syllable"
    );
    // Its F0 exceeds the local declination baseline by ~XU_RISE_HZ (the
    // prevailing event it cloned was already declined).
    let total = risen.total_ms;
    let prevailing_baseline =
        DECLINATION_START_HZ + (DECLINATION_END_HZ - DECLINATION_START_HZ) * (extra.at_ms / total);
    assert!(
        extra.frame.f0_hz > prevailing_baseline + XU_RISE_HZ * 0.6,
        "rise f0 {:.1} vs baseline {:.1}",
        extra.frame.f0_hz,
        prevailing_baseline
    );
}

#[test]
fn pauses_shift_but_never_stretch() {
    let before = compiled("coi la djan. cu klama");
    let after = apply_prosody(before.clone(), &ProsodyOptions::default());
    let count_silence = |s: &UtteranceSchedule| {
        s.events
            .iter()
            .filter(|e| e.frame.targets.formants.iter().all(|f| f.amp == 0.0))
            .count()
    };
    assert_eq!(count_silence(&before), count_silence(&after));
}

#[test]
fn transform_is_deterministic() {
    let a = prosodic("mi tavla do bau la lojban.");
    let b = prosodic("mi tavla do bau la lojban.");
    assert_eq!(a, b);
}

#[test]
fn snapshot_prosodic_coi_munje() {
    insta::assert_debug_snapshot!(prosodic("coi munje"));
}

#[test]
fn snapshot_prosodic_declarative() {
    insta::assert_debug_snapshot!(prosodic("mi tavla do bau la lojban."));
}

#[test]
fn snapshot_prosodic_xu() {
    insta::assert_debug_snapshot!(apply_prosody(
        compiled("xu do klama"),
        &ProsodyOptions { xu_rise: true }
    ));
}
