//! Phase-10 schedule-level acceptance: attitudinal (UI-cmavo) detection, scope,
//! intensity, and the voice-quality overlay. INVENTED / non-normative (see
//! `voksa_core::attitudinal`); these assert voksa's own realization, not CLL.

use voksa_core::attitudinal::{
    AttitudinalKind, AttitudinalTable, Deviation, apply_attitudinal, apply_attitudinal_with,
};
use voksa_core::compiler::{CompileOptions, compile};
use voksa_core::prosody::{ProsodyOptions, apply_prosody};
use voksa_core::schedule::UtteranceSchedule;

fn compiled(text: &str) -> UtteranceSchedule {
    compile(text, &CompileOptions::default()).unwrap_or_else(|e| panic!("{text}: {e:?}"))
}

/// compile → prosody → attitudinal, the full P10 schedule transform.
fn colored(text: &str) -> UtteranceSchedule {
    apply_attitudinal(apply_prosody(compiled(text), &ProsodyOptions::default()))
}

fn word_window(s: &UtteranceSchedule, word_index: usize) -> (f32, f32) {
    let mut start = f32::INFINITY;
    let mut end = f32::NEG_INFINITY;
    for sp in s.spans.iter().filter(|sp| sp.word_index == word_index) {
        start = start.min(sp.start_ms);
        end = end.max(sp.start_ms + sp.dur_ms);
    }
    assert!(start.is_finite(), "word {word_index} has no spans");
    (start, end)
}

/// Mean F0 over the voiced events inside a word's time window.
fn mean_word_f0(s: &UtteranceSchedule, word_index: usize) -> f32 {
    let (a, b) = word_window(s, word_index);
    let mut sum = 0.0;
    let mut n = 0u32;
    for e in &s.events {
        if e.at_ms >= a - 1e-3 && e.at_ms < b - 1e-3 && e.frame.targets.voicing > 0.0 {
            sum += e.frame.f0_hz;
            n += 1;
        }
    }
    assert!(n > 0, "word {word_index} has no voiced events");
    sum / n as f32
}

fn max_word_di(s: &UtteranceSchedule, word_index: usize) -> f32 {
    let (a, b) = word_window(s, word_index);
    s.events
        .iter()
        .filter(|e| e.at_ms >= a - 1e-3 && e.at_ms < b - 1e-3)
        .map(|e| e.frame.di)
        .fold(0.0f32, f32::max)
}

fn max_word_aspiration(s: &UtteranceSchedule, word_index: usize) -> f32 {
    let (a, b) = word_window(s, word_index);
    s.events
        .iter()
        .filter(|e| e.at_ms >= a - 1e-3 && e.at_ms < b - 1e-3 && e.frame.targets.voicing > 0.0)
        .map(|e| e.frame.targets.aspiration)
        .fold(0.0f32, f32::max)
}

// ---- detection + scope + intensity ------------------------------------------

#[test]
fn detects_joy_on_preceding_word() {
    // "coi munje .ui" → words coi(0) munje(1) ui(2); .ui colors munje (i-1).
    let s = compiled("coi munje .ui");
    assert_eq!(s.attitudinals.len(), 1, "one attitudinal expected");
    let sc = s.attitudinals[0];
    assert_eq!(sc.kind, AttitudinalKind::Joy);
    assert_eq!(sc.word_index, 1, "scope targets the preceding content word");
    assert_eq!(sc.intensity, 1.0, "bare attitudinal = full intensity");
}

#[test]
fn detects_intensity_nai_flips_polarity() {
    // "mi klama .ui nai" → ui(2) with a following nai(3) → intensity −1.0.
    let s = compiled("mi klama .ui nai");
    assert_eq!(s.attitudinals.len(), 1);
    let sc = s.attitudinals[0];
    assert_eq!(sc.kind, AttitudinalKind::Joy);
    assert_eq!(sc.word_index, 1, "colors klama, the word before .ui");
    assert_eq!(sc.intensity, -1.0);
}

#[test]
fn detects_intensity_rue_scales_down() {
    let s = compiled("mi klama .ui ru'e");
    assert_eq!(s.attitudinals.len(), 1);
    assert!((s.attitudinals[0].intensity - 0.4).abs() < 1e-6);
}

#[test]
fn detects_anger_as_fused_token() {
    // ".o'onai" is a single fused token → Anger.
    let s = compiled("mi fengu .o'onai");
    assert_eq!(s.attitudinals.len(), 1);
    assert_eq!(s.attitudinals[0].kind, AttitudinalKind::Anger);
    assert_eq!(s.attitudinals[0].word_index, 1);
}

#[test]
fn utterance_initial_colors_first_content_word() {
    // ".ui coi munje" → ui(0) has no preceding content → colors coi(1).
    let s = compiled(".ui coi munje");
    assert_eq!(s.attitudinals.len(), 1);
    assert_eq!(s.attitudinals[0].word_index, 1);
}

#[test]
fn modal_utterance_has_no_attitudinals() {
    assert!(compiled("coi munje").attitudinals.is_empty());
}

// ---- overlay realization ----------------------------------------------------

#[test]
fn joy_raises_target_word_f0() {
    // Joy adds +14 Hz (× intensity) to munje; compare against the same schedule
    // before the attitudinal overlay.
    let before = apply_prosody(compiled("coi munje .ui"), &ProsodyOptions::default());
    let after = colored("coi munje .ui");
    let mean_before = mean_word_f0(&before, 1);
    let mean_after = mean_word_f0(&after, 1);
    assert!(
        mean_after > mean_before + 5.0,
        "Joy must raise munje's F0: {mean_before} -> {mean_after}"
    );
}

#[test]
fn sadness_lowers_f0_and_adds_breathiness() {
    // "mi klama .uu": klama is word 1; sadness lowers F0 and raises aspiration.
    let before = apply_prosody(compiled("mi klama .uu"), &ProsodyOptions::default());
    let after = colored("mi klama .uu");
    assert!(
        mean_word_f0(&after, 1) < mean_word_f0(&before, 1) - 5.0,
        "Sadness must lower klama's F0"
    );
    assert!(
        max_word_aspiration(&after, 1) > max_word_aspiration(&before, 1) + 0.05,
        "Sadness must add breathiness to voiced frames"
    );
}

#[test]
fn complaint_sets_diplophonia() {
    // ".oi" (complaint) injects creak (diplophonia) on its target word.
    let s = colored("coi munje .oi");
    assert!(
        max_word_di(&s, 1) > 0.05,
        "Complaint must set diplophonia on munje"
    );
}

#[test]
fn nai_inverts_the_f0_shift() {
    // Joy raises; Joy × nai (−1) must lower relative to the pre-overlay schedule.
    let before = apply_prosody(compiled("mi klama .ui nai"), &ProsodyOptions::default());
    let after = colored("mi klama .ui nai");
    assert!(
        mean_word_f0(&after, 1) < mean_word_f0(&before, 1) - 3.0,
        "nai must invert Joy's F0 raise into a drop"
    );
}

#[test]
fn overlay_is_deterministic() {
    let a = colored("coi munje .ui");
    let b = colored("coi munje .ui");
    assert_eq!(a, b, "the attitudinal overlay must be deterministic");
}

#[test]
fn modal_utterance_overlay_is_identity() {
    // No attitudinals → apply_attitudinal is a no-op (byte-identical schedule).
    let prosodic = apply_prosody(compiled("coi munje"), &ProsodyOptions::default());
    let after = apply_attitudinal(prosodic.clone());
    assert_eq!(prosodic, after);
}

// ---- runtime deviation table (demo tuning console D2a) -----------------------

#[test]
fn default_table_matches_pinned_vectors() {
    // Byte-identity guard: the runtime table defaults to the pinned constants,
    // so every existing snapshot stays valid.
    let t = AttitudinalTable::default();
    for k in AttitudinalKind::ALL {
        assert_eq!(
            t.get(k),
            k.deviation(),
            "{k:?} must default to its pinned deviation vector"
        );
    }
}

#[test]
fn deviation_array_round_trip() {
    // The flat-f32 crossing (wasm param block) must be lossless for every kind.
    for k in AttitudinalKind::ALL {
        let d = k.deviation();
        assert_eq!(Deviation::from_array(d.to_array()), d, "{k:?}");
    }
}

#[test]
fn kind_index_matches_all_order() {
    for (i, k) in AttitudinalKind::ALL.iter().enumerate() {
        assert_eq!(k.index(), i, "{k:?} index must equal its ALL slot");
    }
}

#[test]
fn apply_with_default_table_equals_pinned() {
    let s = apply_prosody(compiled("coi munje .ui"), &ProsodyOptions::default());
    assert_eq!(
        apply_attitudinal(s.clone()),
        apply_attitudinal_with(s, &AttitudinalTable::default()),
        "the default table must reproduce the pinned overlay byte-identically"
    );
}

#[test]
fn custom_table_changes_overlay() {
    // A community-tuned Joy vector (bigger mean-F0 shift) must color munje
    // further than the pinned one.
    let s = apply_prosody(compiled("coi munje .ui"), &ProsodyOptions::default());
    let mut table = AttitudinalTable::default();
    table.deviations[AttitudinalKind::Joy.index()].f0_mean_hz = 40.0;
    let pinned = apply_attitudinal_with(s.clone(), &AttitudinalTable::default());
    let custom = apply_attitudinal_with(s, &table);
    let (p, c) = (mean_word_f0(&pinned, 1), mean_word_f0(&custom, 1));
    assert!(
        c > p + 10.0,
        "a 40 Hz Joy shift must out-raise the pinned 14 Hz: pinned={p:.1}, custom={c:.1}"
    );
}

#[test]
fn custom_table_is_deterministic() {
    let mut table = AttitudinalTable::default();
    table.deviations[AttitudinalKind::Complaint.index()].d_di = 0.5;
    let mk = || {
        apply_attitudinal_with(
            apply_prosody(compiled("coi munje .oi"), &ProsodyOptions::default()),
            &table,
        )
    };
    assert_eq!(mk(), mk());
}

// ---- per-attitudinal schedule snapshots (compile → prosody → attitudinal) ----

#[test]
fn snapshot_joy_coi_munje_ui() {
    insta::assert_debug_snapshot!(colored("coi munje .ui"));
}

#[test]
fn snapshot_sadness_mi_klama_uu() {
    insta::assert_debug_snapshot!(colored("mi klama .uu"));
}

#[test]
fn snapshot_complaint_coi_munje_oi() {
    insta::assert_debug_snapshot!(colored("coi munje .oi"));
}

#[test]
fn snapshot_fear_coi_munje_ii() {
    insta::assert_debug_snapshot!(colored("coi munje .ii"));
}

#[test]
fn snapshot_anger_mi_fengu_oonai() {
    insta::assert_debug_snapshot!(colored("mi fengu .o'onai"));
}
