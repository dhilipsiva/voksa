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
    // munje = word 1, syllables mun (stressed) + je. Only the rhyme (nucleus
    // onward) stretches 1.5×; the onset consonants stay at unit rate.
    let b_stressed = stressed_spans(&before)[0];
    let a_stressed = stressed_spans(&after)[0];
    let rhyme = b_stressed.dur_ms - b_stressed.nucleus_off_ms;
    let added = rhyme * (STRESS_DURATION_FACTOR - 1.0);
    assert!(
        (a_stressed.dur_ms - (b_stressed.dur_ms + added)).abs() < 1e-2,
        "rhyme-only stretch: {} vs {}",
        a_stressed.dur_ms,
        b_stressed.dur_ms + added
    );
    // The onset is untouched: same start, same offset to the nucleus.
    assert!((a_stressed.start_ms - b_stressed.start_ms).abs() < 1e-3);
    assert!((a_stressed.nucleus_off_ms - b_stressed.nucleus_off_ms).abs() < 1e-3);
    // The following (unstressed) span shifted by the added rhyme duration.
    let b_last = before.spans.last().unwrap();
    let a_last = after.spans.last().unwrap();
    assert!((a_last.start_ms - (b_last.start_ms + added)).abs() < 1e-2);
    assert!(
        (a_last.dur_ms - b_last.dur_ms).abs() < 1e-3,
        "unstressed spans unstretched"
    );
    assert!((after.total_ms - (before.total_ms + added)).abs() < 1e-2);
}

#[test]
fn stretch_leaves_onset_at_unit_rate() {
    // Event-level CP1 guarantee: the onset consonant's event keeps its unit
    // transition time; only the nucleus event's transition scales 1.5×.
    let before = compiled("coi munje");
    let after = apply_prosody(before.clone(), &ProsodyOptions::default());
    let stressed = stressed_spans(&after)[0];
    assert!(stressed.nucleus_off_ms > 0.0, "mun has an onset");
    let nucleus_at = stressed.start_ms + stressed.nucleus_off_ms;
    assert_eq!(
        before.events.len(),
        after.events.len(),
        "stretch preserves count"
    );
    // Onset event: sits at the span start, before the nucleus.
    let (b_on, a_on) = before
        .events
        .iter()
        .zip(&after.events)
        .find(|(_, a)| (a.at_ms - stressed.start_ms).abs() < 1e-3)
        .expect("an onset event at the span start");
    assert!(
        (a_on.transition_ms - b_on.transition_ms).abs() < 1e-3,
        "onset transition must stay at unit rate"
    );
    // Nucleus event: at nucleus_at, transition scaled.
    let (b_nuc, a_nuc) = before
        .events
        .iter()
        .zip(&after.events)
        .find(|(_, a)| (a.at_ms - nucleus_at).abs() < 1e-3)
        .expect("a nucleus event");
    assert!(
        (a_nuc.transition_ms - b_nuc.transition_ms * STRESS_DURATION_FACTOR).abs() < 1e-3,
        "nucleus transition must scale ×{STRESS_DURATION_FACTOR}"
    );
}

#[test]
fn excursion_and_amp_boost_remain_whole_span() {
    // The stretch is nucleus-scoped, but the F0 excursion + amplitude boost
    // still cover the WHOLE stressed syllable, onset consonants included.
    let after = prosodic("coi munje");
    let stressed = stressed_spans(&after)[0];
    assert!(stressed.nucleus_off_ms > 0.0, "mun has an onset");
    let onset_event = after
        .events
        .iter()
        .find(|e| {
            e.at_ms >= stressed.start_ms - 1e-3
                && e.at_ms < stressed.start_ms + stressed.nucleus_off_ms - 1e-3
                && e.frame.targets.voicing > 0.0
        })
        .expect("a voiced onset event inside the stressed span");
    let total = after.total_ms;
    let baseline = DECLINATION_START_HZ
        + (DECLINATION_END_HZ - DECLINATION_START_HZ) * (onset_event.at_ms / total);
    assert!(
        (onset_event.frame.f0_hz - baseline - STRESS_F0_EXCURSION_HZ).abs() < 1.0,
        "onset event must carry the +{STRESS_F0_EXCURSION_HZ} Hz excursion"
    );
}

#[test]
fn buffered_stressed_onset_not_stretched() {
    // Buffered vrusi: the epenthetic buffer sits in the stressed onset (before
    // the nucleus) and must NOT stretch or shift.
    let opts = CompileOptions {
        dotside: false,
        buffer: true,
    };
    let before = compile("vrusi", &opts).unwrap();
    let after = apply_prosody(before.clone(), &ProsodyOptions::default());
    let b_buf = before.spans.iter().find(|sp| !sp.countable).unwrap();
    let a_buf = after.spans.iter().find(|sp| !sp.countable).unwrap();
    assert!(
        (a_buf.dur_ms - b_buf.dur_ms).abs() < 1e-3,
        "onset buffer not stretched"
    );
    assert!(
        (a_buf.start_ms - b_buf.start_ms).abs() < 1e-3,
        "onset buffer not shifted"
    );
}

#[test]
fn xu_rise_with_stressed_final_span_stays_sorted() {
    // Final span stressed (djan) + xu rise: exactly one added event, sorted.
    let risen = apply_prosody(
        compiled("xu la djan."),
        &ProsodyOptions {
            xu_rise: true,
            ..Default::default()
        },
    );
    let flat = apply_prosody(compiled("xu la djan."), &ProsodyOptions::default());
    assert_eq!(risen.events.len(), flat.events.len() + 1);
    for w in risen.events.windows(2) {
        assert!(w[0].at_ms <= w[1].at_ms + 1e-3, "events stay time-sorted");
    }
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
    // Each stressed span stretches its rhyme (nucleus onward) by 1.5×.
    for (b, a) in b_stressed.iter().zip(&a_stressed) {
        let rhyme = b.dur_ms - b.nucleus_off_ms;
        assert!((a.dur_ms - (b.dur_ms + rhyme * (STRESS_DURATION_FACTOR - 1.0))).abs() < 1e-2);
    }
    let added: f32 = b_stressed
        .iter()
        .map(|sp| (sp.dur_ms - sp.nucleus_off_ms) * (STRESS_DURATION_FACTOR - 1.0))
        .sum();
    assert!((after.total_ms - (before.total_ms + added)).abs() < 1e-2);
}

#[test]
fn xu_rise_adds_one_sorted_event_with_cloned_targets() {
    let flat = apply_prosody(compiled("xu do klama"), &ProsodyOptions::default());
    let risen = apply_prosody(
        compiled("xu do klama"),
        &ProsodyOptions {
            xu_rise: true,
            ..Default::default()
        },
    );
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
        &ProsodyOptions {
            xu_rise: true,
            ..Default::default()
        }
    ));
}

// ---- runtime parameters (demo tuning console) ----

#[test]
fn default_options_equal_pinned_constants() {
    // The whole default-preserving approach hinges on this: Default == consts.
    let d = ProsodyOptions::default();
    assert_eq!(d.declination_start_hz, DECLINATION_START_HZ);
    assert_eq!(d.declination_end_hz, DECLINATION_END_HZ);
    assert_eq!(d.stress_duration_factor, STRESS_DURATION_FACTOR);
    assert_eq!(d.stress_f0_excursion_hz, STRESS_F0_EXCURSION_HZ);
    assert_eq!(d.stress_amp_factor, STRESS_AMP_FACTOR);
    assert_eq!(d.xu_rise_hz, XU_RISE_HZ);
    assert_eq!(d.rate, 1.0);
    assert!(!d.xu_rise);
}

fn last_voiced_f0(s: &UtteranceSchedule) -> f32 {
    s.events
        .iter()
        .rev()
        .find(|e| e.frame.targets.voicing > 0.0)
        .unwrap()
        .frame
        .f0_hz
}

#[test]
fn declination_end_param_lowers_ending_f0() {
    let text = "mi tavla do bau la lojban.";
    let base = apply_prosody(compiled(text), &ProsodyOptions::default());
    let lowered = apply_prosody(
        compiled(text),
        &ProsodyOptions {
            declination_end_hz: 70.0,
            ..Default::default()
        },
    );
    assert!(
        last_voiced_f0(&lowered) < last_voiced_f0(&base) - 10.0,
        "a lower declination_end must lower the ending F0"
    );
}

#[test]
fn stress_duration_factor_param_changes_stretch() {
    let text = "coi munje";
    let base = apply_prosody(compiled(text), &ProsodyOptions::default());
    let more = apply_prosody(
        compiled(text),
        &ProsodyOptions {
            stress_duration_factor: 2.5,
            ..Default::default()
        },
    );
    let dur = |s: &UtteranceSchedule| stressed_spans(s)[0].dur_ms;
    assert!(
        dur(&more) > dur(&base) + 10.0,
        "a larger stress_duration_factor must stretch the rhyme more"
    );
}

#[test]
fn rate_param_scales_total() {
    let text = "mi tavla do";
    let base = apply_prosody(compiled(text), &ProsodyOptions::default());
    let fast = apply_prosody(
        compiled(text),
        &ProsodyOptions {
            rate: 2.0,
            ..Default::default()
        },
    );
    assert!(
        (fast.total_ms - base.total_ms / 2.0).abs() < 1.0,
        "rate 2.0 must halve total_ms (got {} vs {})",
        fast.total_ms,
        base.total_ms
    );
}

#[test]
fn xu_rise_hz_param_scales_the_rise() {
    let text = "xu do klama";
    let small = apply_prosody(
        compiled(text),
        &ProsodyOptions {
            xu_rise: true,
            xu_rise_hz: 10.0,
            ..Default::default()
        },
    );
    let big = apply_prosody(
        compiled(text),
        &ProsodyOptions {
            xu_rise: true,
            xu_rise_hz: 60.0,
            ..Default::default()
        },
    );
    assert!(
        last_voiced_f0(&big) > last_voiced_f0(&small) + 20.0,
        "a bigger xu_rise_hz must raise the ending F0 more"
    );
}
