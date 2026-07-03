//! Phase-5 acceptance: the text→schedule compiler is deterministic, honors
//! all pause rules and writer periods, realizes --buffer per CLL §3.8, and
//! its output is pinned by insta snapshots.

use voksa_core::compiler::{CompileError, CompileOptions, RawToken, compile, tokenize};
use voksa_core::letters::WordError;
use voksa_core::schedule::UtteranceSchedule;

fn plain() -> CompileOptions {
    CompileOptions::default()
}

fn compiled(text: &str) -> UtteranceSchedule {
    compile(text, &plain()).unwrap_or_else(|e| panic!("{text}: {e:?}"))
}

fn compiled_with(text: &str, opts: CompileOptions) -> UtteranceSchedule {
    compile(text, &opts).unwrap_or_else(|e| panic!("{text}: {e:?}"))
}

/// Count pause events: silent frames held for the pause length. (Voiceless
/// stop closures are also silent frames, but they hold only ~60 ms; pauses
/// hold PAUSE_MS = 100 ms.)
fn pause_events(s: &UtteranceSchedule) -> usize {
    s.events
        .iter()
        .enumerate()
        .filter(|(i, e)| {
            let silent = e.frame.targets.voicing == 0.0
                && e.frame.targets.aspiration == 0.0
                && e.frame.targets.formants.iter().all(|f| f.amp == 0.0);
            let next_at = s.events.get(i + 1).map_or(s.total_ms, |n| n.at_ms);
            silent && (next_at - e.at_ms) >= 99.0
        })
        .count()
}

// ---- snapshots ----

#[test]
fn snapshot_coi_munje() {
    insta::assert_debug_snapshot!(compiled("coi munje"));
}

#[test]
fn snapshot_mi_klama() {
    insta::assert_debug_snapshot!(compiled("mi klama"));
}

#[test]
fn snapshot_la_djan_cu_klama() {
    insta::assert_debug_snapshot!(compiled("la djan. cu klama"));
}

#[test]
fn snapshot_stress_collision() {
    insta::assert_debug_snapshot!(compiled("e'U bridi"));
}

#[test]
fn snapshot_hesitation() {
    insta::assert_debug_snapshot!(compiled("mi .y. klama"));
}

#[test]
fn snapshot_dotside() {
    insta::assert_debug_snapshot!(compiled_with(
        "la djan.",
        CompileOptions {
            dotside: true,
            buffer: false,
        }
    ));
}

#[test]
fn snapshot_buffer_klama() {
    insta::assert_debug_snapshot!(compiled_with(
        "klama",
        CompileOptions {
            dotside: false,
            buffer: true,
        }
    ));
}

#[test]
fn snapshot_buffer_vrusi() {
    // CLL §3.8 ex 8.1: vrusi -> [vɪ ru si] when buffered.
    insta::assert_debug_snapshot!(compiled_with(
        "vrusi",
        CompileOptions {
            dotside: false,
            buffer: true,
        }
    ));
}

// ---- tokenizer ----

#[test]
fn tokenize_periods_delimit_and_pause() {
    use RawToken::{ExplicitPause as P, Word as W};
    assert_eq!(
        tokenize(".i.ai.o").unwrap(),
        [P, W("i".into()), P, W("ai".into()), P, W("o".into()),]
    );
    assert_eq!(
        tokenize("cy. .ibu").unwrap(),
        [W("cy".into()), P, W("ibu".into())]
    );
}

#[test]
fn tokenize_preserves_capitals_and_word_internals() {
    use RawToken::Word as W;
    assert_eq!(
        tokenize("e'U kat,r,in").unwrap(),
        [W("e'U".into()), W("kat,r,in".into())]
    );
}

#[test]
fn tokenize_rejects_digits_for_phase_6() {
    assert_eq!(
        tokenize("mi 42 klama"),
        Err(CompileError::DigitsUnsupported("42".into()))
    );
}

// ---- compile errors ----

#[test]
fn compile_error_vectors() {
    assert_eq!(compile("", &plain()), Err(CompileError::Empty));
    assert_eq!(compile("   ", &plain()), Err(CompileError::Empty));
    assert_eq!(compile("...", &plain()), Err(CompileError::Empty));
    assert_eq!(
        compile("hello", &plain()),
        Err(CompileError::Word {
            word: "hello".into(),
            error: WordError::InvalidCharacter('h'),
        })
    );
}

// ---- semantics ----

#[test]
fn compile_is_deterministic() {
    let opts = CompileOptions {
        dotside: true,
        buffer: true,
    };
    let a = compile("coi la djan. cu klama", &opts).unwrap();
    let b = compile("coi la djan. cu klama", &opts).unwrap();
    assert_eq!(a, b);
}

#[test]
fn writer_period_merges_with_mandatory_pause() {
    // Rule 4 already pauses between coi and djan; the writer period adds
    // nothing: identical schedules.
    assert_eq!(compiled("coi. djan"), compiled("coi djan"));
}

#[test]
fn writer_period_is_honored_where_optional() {
    // No rule mandates a pause between mi and klama; the period forces one.
    let with = compiled("mi. klama");
    let without = compiled("mi klama");
    assert_eq!(pause_events(&with), pause_events(&without) + 1);
    assert!(with.total_ms > without.total_ms);
}

#[test]
fn buffer_preserves_stress_and_adds_uncountable_spans() {
    let plain_arm = compiled("armstrong");
    let buffered = compiled_with(
        "armstrong",
        CompileOptions {
            dotside: false,
            buffer: true,
        },
    );
    let stressed = |s: &UtteranceSchedule| {
        s.spans
            .iter()
            .filter(|sp| sp.stressed)
            .map(|sp| sp.word_index)
            .collect::<Vec<_>>()
    };
    // Exactly one stressed span, same word, buffered or not (CLL §3.9:
    // "the stress remains in the same place").
    assert_eq!(stressed(&plain_arm), [0]);
    assert_eq!(stressed(&buffered), [0]);
    assert!(buffered.spans.len() > plain_arm.spans.len());
    assert!(buffered.total_ms > plain_arm.total_ms);
    for span in &buffered.spans {
        if span.stressed {
            assert!(span.countable, "stressed span must be countable");
        }
    }
    assert!(
        buffered.spans.iter().any(|sp| !sp.countable),
        "buffer spans are uncountable"
    );
}

#[test]
fn spans_cover_words_in_order() {
    let s = compiled("coi djan");
    // coi = 1 syllable (word 0), djan = 1 syllable (word 1).
    assert_eq!(s.spans.len(), 2);
    assert_eq!(s.spans[0].word_index, 0);
    assert_eq!(s.spans[1].word_index, 1);
    assert!(
        !s.spans[0].stressed,
        "coi is a cmavo: unstressed by default"
    );
    assert!(
        s.spans[1].stressed,
        "djan: single countable syllable of a cmevla"
    );
    assert!(s.spans[0].start_ms < s.spans[1].start_ms);
}

#[test]
fn pause_silence_events_match_rules() {
    // coi djan: pause between (r4) + trailing (r2) = 2 silence events.
    assert_eq!(pause_events(&compiled("coi djan")), 2);
    // mi klama: no mandatory pauses at all.
    assert_eq!(pause_events(&compiled("mi klama")), 0);
    // .y. alone: leading + trailing.
    assert_eq!(pause_events(&compiled("y")), 2);
}
