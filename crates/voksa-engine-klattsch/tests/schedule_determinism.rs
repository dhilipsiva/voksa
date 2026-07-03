//! The schedule (the compiler's future output format) must be deterministic.
//! Snapshot the lowered events; verify same-seed renders are identical within
//! a single platform/process. (Cross-platform WAV bit-comparison stays
//! forbidden — this is same-process engine determinism, a different claim.)

use voksa_engine_klattsch::{SAMPLE_RATE, render_schedule, steady_a_schedule};

#[test]
fn schedule_lowering_is_deterministic() {
    let schedule = steady_a_schedule(SAMPLE_RATE);
    insta::assert_debug_snapshot!(schedule.events());
}

#[test]
fn same_seed_renders_identically_in_process() {
    let a = render_schedule(steady_a_schedule(SAMPLE_RATE), SAMPLE_RATE, 100);
    let b = render_schedule(steady_a_schedule(SAMPLE_RATE), SAMPLE_RATE, 100);
    assert_eq!(a, b);
}
