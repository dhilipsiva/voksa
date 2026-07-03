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

// ---- nucleus offset (Phase 7.1): the stress stretch anchors at the vowel ----

#[test]
fn nucleus_offset_zero_for_onsetless_syllable() {
    // e'u: syllable 0 "e" has no onset, so its nucleus sits at the span start.
    let s = compiled("e'u");
    assert_eq!(
        s.spans[0].nucleus_off_ms, 0.0,
        "onsetless nucleus is at the span start"
    );
}

#[test]
fn nucleus_offset_includes_aspirate() {
    // e'u: syllable 1 "'u" opens with [h] (70 ms). The breathy transition is
    // onset material — it stays at unit rate, so it counts toward the offset.
    let s = compiled("e'u");
    assert_eq!(s.spans[1].nucleus_off_ms, 70.0);
}

#[test]
fn nucleus_offset_spans_onset_cluster() {
    // klama = kla-ma. kla: k(60+25) + l(80) = 165 ms before the vowel; ma: m(80).
    let s = compiled("klama");
    assert_eq!(s.spans[0].nucleus_off_ms, 165.0);
    assert_eq!(s.spans[1].nucleus_off_ms, 80.0);
}

#[test]
fn nucleus_offset_includes_onset_buffer() {
    // Buffered vrusi = [v ɪ r]u-si: the epenthetic buffer sits in the onset, so
    // the offset is v(120) + buffer(35) + r(80) = 235; the buffer's own span is
    // itself a nucleus (offset 0).
    let s = compiled_with(
        "vrusi",
        CompileOptions {
            dotside: false,
            buffer: true,
        },
    );
    let vru = s
        .spans
        .iter()
        .find(|sp| sp.stressed)
        .expect("vru is stressed");
    assert_eq!(vru.nucleus_off_ms, 235.0);
    let buffer_span = s
        .spans
        .iter()
        .find(|sp| !sp.countable)
        .expect("a buffer span");
    assert_eq!(buffer_span.nucleus_off_ms, 0.0);
}

#[test]
fn nucleus_offset_for_syllabic_consonant() {
    // kat,r,in: the middle syllabic-r syllable has no onset (offset 0) and is
    // never stressable (uncountable).
    let s = compiled("kat,r,in");
    let syllabic = &s.spans[1];
    assert_eq!(syllabic.nucleus_off_ms, 0.0);
    assert!(
        !syllabic.stressed,
        "syllabic-consonant syllables are uncountable"
    );
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
fn tokenize_expands_figures_to_pa_cmavo() {
    // Supersedes the Phase-5 placeholder (tokenize_rejects_digits_for_phase_6).
    use RawToken::Word as W;
    assert_eq!(
        tokenize("mi 42 klama").unwrap(),
        [
            W("mi".into()),
            W("vo".into()),
            W("re".into()),
            W("klama".into()),
        ]
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
