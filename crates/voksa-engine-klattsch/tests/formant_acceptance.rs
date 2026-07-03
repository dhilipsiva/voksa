//! Phase-1 acceptance gate: the rendered steady /a/ vowel must have
//! F1/F2/F3 within ±10% of the docs/formants.md seeds (730/1090/2440 Hz).

use voksa_engine_klattsch::{SAMPLE_RATE, render_schedule, steady_a_schedule};
use voksa_testkit::{FormantCheck, assert_formants, measure_formants_fft};

#[test]
fn steady_a_formants_within_10_percent() {
    let samples = render_schedule(steady_a_schedule(SAMPLE_RATE), SAMPLE_RATE, 500);
    let got = measure_formants_fft(&samples, SAMPLE_RATE);
    eprintln!(
        "measured /a/: F1={:.1} F2={:.1} F3={:.1} (targets 730/1090/2440)",
        got.f1, got.f2, got.f3
    );
    assert_formants(
        &got,
        &FormantCheck {
            f1: 730.0,
            f2: 1090.0,
            f3: 2440.0,
        },
        0.10,
    );
}
