//! Render the Phase-1 steady /a/ spike to a WAV for human ears.
//! Usage: cargo run -p voksa-engine-klattsch --example render_a

use voksa_engine_klattsch::{SAMPLE_RATE, render_schedule, steady_a_schedule};

fn main() {
    let samples = render_schedule(steady_a_schedule(SAMPLE_RATE), SAMPLE_RATE, 1000);
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    let path = "artifacts/listening/phase1/a.wav";
    voksa_testkit::write_wav(path, &samples, SAMPLE_RATE);
    println!("wrote {path} ({} samples, rms {rms:.4})", samples.len());
}
