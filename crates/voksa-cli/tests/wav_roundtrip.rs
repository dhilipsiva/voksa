//! The hand-rolled WAV writer produces files hound reads back with the
//! expected spec and samples. hound is a DEV-dependency only.

use voksa_cli::wav;

#[test]
fn hound_reads_back_our_wav() {
    let path = std::env::temp_dir().join(format!("voksa_roundtrip_{}.wav", std::process::id()));
    let samples = [0.0f32, 0.5, -0.5, 1.0, -1.0];
    wav::write_wav(&path, &samples, 48_000).unwrap();

    let spec = hound::WavReader::open(&path).unwrap().spec();
    assert_eq!(spec.channels, 1);
    assert_eq!(spec.sample_rate, 48_000);
    assert_eq!(spec.bits_per_sample, 16);
    assert_eq!(spec.sample_format, hound::SampleFormat::Int);

    let read: Vec<i16> = hound::WavReader::open(&path)
        .unwrap()
        .into_samples::<i16>()
        .map(|s| s.unwrap())
        .collect();
    let expect: Vec<i16> = samples
        .iter()
        .map(|s| (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16)
        .collect();
    assert_eq!(read, expect);

    let _ = std::fs::remove_file(&path);
}
