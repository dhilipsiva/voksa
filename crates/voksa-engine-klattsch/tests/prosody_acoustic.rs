//! Phase-7 acoustic acceptance (playbook §e criteria): measure the rendered
//! prosody with the F0 harness — declination slope, stress prominence, xu
//! rise. Tolerances per the playbook (±5 Hz synthetic; ±8 at a padded start).

use voksa_core::compiler::CompileOptions;
use voksa_core::prosody::ProsodyOptions;
use voksa_engine_klattsch::{SAMPLE_RATE, render_utterance_prosodic};
use voksa_testkit::{f0_near, fit_line, measure_f0_track};

fn render(text: &str, xu: bool) -> Vec<f32> {
    render_utterance_prosodic(
        text,
        &CompileOptions::default(),
        &ProsodyOptions {
            xu_rise: xu,
            ..Default::default()
        },
        SAMPLE_RATE,
    )
    .unwrap_or_else(|e| panic!("{text}: {e:?}"))
}

#[test]
fn declination_slope_negative_with_correct_endpoints() {
    let samples = render("mi tavla do bau la lojban.", false);
    let track = measure_f0_track(&samples, SAMPLE_RATE);
    assert!(track.len() >= 15, "need voiced frames, got {}", track.len());
    let (slope, _) = fit_line(&track);
    assert!(
        slope < 0.0,
        "declination slope must be negative, got {slope}"
    );
    let start = track.first().unwrap().1;
    let end = track.last().unwrap().1;
    assert!(
        (start - 120.0).abs() <= 8.0,
        "start F0 {start:.1} (expect ~120; stressed excursions may touch early frames)"
    );
    assert!((end - 95.0).abs() <= 8.0, "end F0 {end:.1} (expect ~95)");
}

#[test]
fn stressed_syllable_has_higher_f0_and_amplitude() {
    // klama: kla stressed. Use the prosodic schedule's span times to locate
    // the stressed window acoustically.
    let text = "le prenu cu klama";
    let schedule = voksa_core::prosody::apply_prosody(
        voksa_core::compiler::compile(text, &CompileOptions::default()).unwrap(),
        &ProsodyOptions::default(),
    );
    let samples = render(text, false);
    let track = measure_f0_track(&samples, SAMPLE_RATE);
    let stressed: Vec<_> = schedule.spans.iter().filter(|sp| sp.stressed).collect();
    let unstressed: Vec<_> = schedule
        .spans
        .iter()
        .filter(|sp| !sp.stressed && sp.countable)
        .collect();
    let mean_f0_over = |spans: &[&voksa_core::schedule::SyllableSpan]| {
        let vals: Vec<f32> = spans
            .iter()
            .filter_map(|sp| f0_near(&track, sp.start_ms + sp.dur_ms / 2.0, sp.dur_ms / 2.0))
            .collect();
        vals.iter().sum::<f32>() / vals.len() as f32
    };
    // Peak 30 ms-window RMS per span (the vowel core): whole-span RMS would
    // unfairly average in the stop closures stressed syllables contain.
    let rms_over = |spans: &[&voksa_core::schedule::SyllableSpan]| {
        let win = (0.030 * SAMPLE_RATE as f32) as usize;
        let mut acc = 0.0f32;
        let mut n = 0usize;
        for sp in spans {
            let a = (sp.start_ms / 1000.0 * SAMPLE_RATE as f32) as usize;
            let b = (((sp.start_ms + sp.dur_ms) / 1000.0) * SAMPLE_RATE as f32) as usize;
            let slice = &samples[a.min(samples.len())..b.min(samples.len())];
            let mut peak = 0.0f32;
            let mut start = 0;
            while start + win <= slice.len() {
                let rms = (slice[start..start + win].iter().map(|s| s * s).sum::<f32>()
                    / win as f32)
                    .sqrt();
                peak = peak.max(rms);
                start += win / 2;
            }
            acc += peak;
            n += 1;
        }
        acc / n.max(1) as f32
    };
    let f0_stressed = mean_f0_over(&stressed);
    let f0_unstressed = mean_f0_over(&unstressed);
    assert!(
        f0_stressed > f0_unstressed + 5.0,
        "stressed F0 {f0_stressed:.1} vs unstressed {f0_unstressed:.1}"
    );
    let rms_stressed = rms_over(&stressed);
    let rms_unstressed = rms_over(&unstressed);
    assert!(
        rms_stressed > rms_unstressed,
        "stressed rms {rms_stressed:.4} vs unstressed {rms_unstressed:.4}"
    );
}

#[test]
fn xu_terminal_rise_is_audible_in_f0() {
    let samples = render("xu do klama", true);
    let track = measure_f0_track(&samples, SAMPLE_RATE);
    assert!(track.len() >= 8);
    let end = track.last().unwrap();
    // The declination floor is ~95 Hz; a risen ending must sit well above it.
    // ("xu do klama" is too short for a clean pre-final reference window —
    // any fixed offset lands in the stressed syllable — so the rigorous
    // comparison is the flat-vs-risen delta below.)
    assert!(
        end.1 > 110.0,
        "final F0 {:.1} must sit well above the declination floor",
        end.1
    );
    // And the same text without the flag falls instead.
    let flat = render("xu do klama", false);
    let flat_track = measure_f0_track(&flat, SAMPLE_RATE);
    let flat_end = flat_track.last().unwrap();
    assert!(
        end.1 > flat_end.1 + 5.0,
        "risen end {:.1} vs flat end {:.1}",
        end.1,
        flat_end.1
    );
}
