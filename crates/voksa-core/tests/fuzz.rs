//! Phase-11 W1 stable fuzzing (proptest, no nightly): totality + structural
//! sanity of the full text → schedule pipeline under hostile input. Run deep
//! with `cargo xtask fuzz` (PROPTEST_CASES=65536); the normal CI `test` job
//! runs this suite at the proptest default (256 cases).

use proptest::prelude::*;
use voksa_core::compiler::{CompileOptions, compile, compile_with};
use voksa_core::phonemes::VoiceTable;
use voksa_core::prosody::{ProsodyOptions, apply_prosody};
use voksa_core::schedule::UtteranceSchedule;
use voksa_core::transcribe::transcribe;

/// Structural sanity: every number finite, times non-negative, events
/// time-sorted, spans inside the utterance, nuclei inside their spans.
///
/// Two tolerances: EPS for order relations that survive scaling exactly
/// (multiplication by a positive constant is monotone), and a RELATIVE
/// epsilon for the span-window sum — sub-EPS slack between `start + dur` and
/// `total` scales linearly under extreme (legitimate) rate values.
fn assert_sane(s: &UtteranceSchedule) -> Result<(), TestCaseError> {
    const EPS: f32 = 1e-3;
    let span_eps = EPS.max(s.total_ms.abs() * 1e-5);
    prop_assert!(
        s.total_ms.is_finite() && s.total_ms >= 0.0,
        "total_ms {}",
        s.total_ms
    );
    let mut prev = 0.0f32;
    for e in &s.events {
        prop_assert!(
            e.at_ms.is_finite() && e.at_ms >= -EPS,
            "event at_ms {}",
            e.at_ms
        );
        prop_assert!(
            e.transition_ms.is_finite() && e.transition_ms >= 0.0,
            "transition_ms {}",
            e.transition_ms
        );
        prop_assert!(
            e.at_ms >= prev - EPS,
            "events sorted: {} < {}",
            e.at_ms,
            prev
        );
        prev = e.at_ms;
        prop_assert!(e.frame.f0_hz.is_finite(), "f0 {}", e.frame.f0_hz);
        for f in &e.frame.targets.formants {
            prop_assert!(
                f.freq_hz.is_finite() && f.bw_hz.is_finite() && f.amp.is_finite(),
                "formant {f:?}"
            );
        }
        prop_assert!(
            e.frame.oq.is_finite()
                && e.frame.tilt.is_finite()
                && e.frame.di.is_finite()
                && e.frame.vibrato_hz.is_finite()
                && e.frame.flutter.is_finite(),
            "VQ lanes finite"
        );
    }
    for sp in &s.spans {
        prop_assert!(sp.start_ms.is_finite() && sp.start_ms >= -EPS);
        prop_assert!(sp.dur_ms.is_finite() && sp.dur_ms >= 0.0);
        prop_assert!(sp.nucleus_off_ms.is_finite() && sp.nucleus_off_ms >= 0.0);
        prop_assert!(
            sp.nucleus_off_ms <= sp.dur_ms + EPS,
            "nucleus_off {} > dur {}",
            sp.nucleus_off_ms,
            sp.dur_ms
        );
        prop_assert!(
            sp.start_ms + sp.dur_ms <= s.total_ms + span_eps,
            "span [{}, +{}] beyond total {}",
            sp.start_ms,
            sp.dur_ms,
            s.total_ms
        );
    }
    Ok(())
}

/// Lojban-flavored soup: the alphabet plus the punctuation/digit surface the
/// tokenizer + normalizer accept (or reject with typed errors).
static SOUP: [char; 42] = [
    'a', 'e', 'i', 'o', 'u', 'y', 'b', 'c', 'd', 'f', 'g', 'j', 'k', 'l', 'm', 'n', 'p', 'r', 's',
    't', 'v', 'x', 'z', '\'', ',', '.', ' ', 'A', 'E', 'K', 'M', 'S', '0', '1', '2', '3', '4', '9',
    ':', 'q', 'h', 'w',
];

fn lojban_soup() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(&SOUP[..]), 0..40)
        .prop_map(|chars| chars.into_iter().collect())
}

static NUMBER_CHARS: [char; 15] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',', ':', ' ', 'p',
];

fn number_soup() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(&NUMBER_CHARS[..]), 0..24)
        .prop_map(|chars| chars.into_iter().collect())
}

/// A small compilable corpus for the transform-level properties (they need a
/// valid schedule to transform; the soup properties cover the parser).
fn corpus_text() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "mi klama",
            "coi munje",
            "xu do klama",
            "le prenu cu klama",
            "coi la djan.",
            "li ci pi pa vo",
            "coi munje .ui",
        ][..],
    )
}

/// The pre-registered W1 finding, red-first: `scale_rate` passed NaN through
/// both of its guards (`rate == 1.0` and `rate <= 0.0` are both false for
/// NaN), poisoning every time in the schedule; ±inf and subnormal rates
/// overflow the times to inf/NaN via `k = 1/rate`. Non-finite or
/// non-realizable rates must behave as identity.
#[test]
fn non_finite_rate_is_identity() {
    let compiled = compile("mi klama", &CompileOptions::default()).unwrap();
    let unit = apply_prosody(
        compiled.clone(),
        &ProsodyOptions {
            rate: 1.0,
            ..Default::default()
        },
    );
    for hostile in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1e-39] {
        let out = apply_prosody(
            compiled.clone(),
            &ProsodyOptions {
                rate: hostile,
                ..Default::default()
            },
        );
        assert_eq!(
            out, unit,
            "rate {hostile} must fall back to identity, not poison the schedule"
        );
    }
}

proptest! {
    /// Totality on arbitrary unicode: compile and transcribe never panic, and
    /// they AGREE on what parses (the demo shows transcriptions for exactly
    /// the utterances it can speak).
    #[test]
    fn pipeline_is_total_on_arbitrary_unicode(text in any::<String>()) {
        let opts = CompileOptions::default();
        let compiled = compile(&text, &opts);
        let transcribed = transcribe(&text, &opts);
        prop_assert_eq!(compiled.is_ok(), transcribed.is_ok(), "compile/transcribe disagree on {:?}", text);
    }

    /// Lojban-flavored soup: whatever parses produces a structurally sane
    /// schedule, stays sane through default prosody, and is deterministic.
    #[test]
    fn lojban_soup_schedules_are_sane(
        text in lojban_soup(),
        dotside in any::<bool>(),
        buffer in any::<bool>(),
    ) {
        let opts = CompileOptions { dotside, buffer };
        if let Ok(s) = compile(&text, &opts) {
            assert_sane(&s)?;
            let again = compile(&text, &opts).unwrap();
            prop_assert_eq!(&again, &s, "compile must be deterministic");
            assert_sane(&apply_prosody(s, &ProsodyOptions::default()))?;
        }
        prop_assert_eq!(
            compile(&text, &opts).is_ok(),
            transcribe(&text, &opts).is_ok(),
            "compile/transcribe disagree on {:?}", text
        );
    }

    /// Digit/number soup exercises the normalizer (PA cmavo, pi, ki'o, pi'e):
    /// total, and in agreement with the transcriber.
    #[test]
    fn number_soup_is_total(text in number_soup()) {
        let opts = CompileOptions::default();
        prop_assert_eq!(
            compile(&text, &opts).is_ok(),
            transcribe(&text, &opts).is_ok(),
            "compile/transcribe disagree on {:?}", text
        );
    }

    /// ANY f32 rate — hostile included — leaves the schedule structurally
    /// sane (non-realizable rates fall back to identity).
    #[test]
    fn any_rate_preserves_sanity(text in corpus_text(), rate in any::<f32>()) {
        let s = compile(text, &CompileOptions::default()).unwrap();
        assert_sane(&apply_prosody(s, &ProsodyOptions { rate, ..Default::default() }))?;
    }

    /// Hostile-but-finite voice tables (the CLI config surface: serde_json
    /// admits any finite f32) compile to sane schedules, deterministically,
    /// and stay sane through default prosody.
    #[test]
    fn hostile_finite_voice_tables_compile_sane(
        text in corpus_text(),
        raw in prop::collection::vec(-1e30f32..1e30f32, VoiceTable::FIELDS),
    ) {
        let arr: [f32; VoiceTable::FIELDS] = raw.try_into().unwrap();
        let table = VoiceTable::from_array(arr);
        let opts = CompileOptions::default();
        let s = compile_with(text, &opts, &table).unwrap();
        assert_sane(&s)?;
        prop_assert_eq!(&compile_with(text, &opts, &table).unwrap(), &s);
        assert_sane(&apply_prosody(s, &ProsodyOptions::default()))?;
    }
}
