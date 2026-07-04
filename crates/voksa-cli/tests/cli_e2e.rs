//! End-to-end: drive the built `voksa` binary via CARGO_BIN_EXE_voksa.

use std::path::PathBuf;
use std::process::Command;

fn voksa() -> Command {
    Command::new(env!("CARGO_BIN_EXE_voksa"))
}

fn tmp_wav(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("voksa_e2e_{}_{}.wav", tag, std::process::id()))
}

fn u32_at(b: &[u8], i: usize) -> usize {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize
}

#[test]
fn out_renders_riff_valid_wav() {
    let path = tmp_wav("out");
    let status = voksa()
        .args(["--out", path.to_str().unwrap(), "coi", "munje"])
        .status()
        .unwrap();
    assert!(status.success());
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.len() > 44, "more than a bare header");
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(u32_at(&bytes, 4), bytes.len() - 8, "RIFF size = file - 8");
    assert_eq!(
        u32_at(&bytes, 40),
        bytes.len() - 44,
        "data size = file - 44"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn bad_lojban_fails_with_stderr_and_no_file() {
    let path = tmp_wav("bad");
    let out = voksa()
        .args(["--out", path.to_str().unwrap(), "hello"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "invalid Lojban must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error"), "stderr names an error: {stderr}");
    assert!(
        stderr.contains("hello"),
        "stderr names the bad word: {stderr}"
    );
    assert!(!path.exists(), "no WAV written on failure");
}

#[test]
fn flat_differs_from_prosodic() {
    let p_pros = tmp_wav("pros");
    let p_flat = tmp_wav("flat");
    assert!(
        voksa()
            .args(["--out", p_pros.to_str().unwrap(), "mi", "tavla", "do"])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        voksa()
            .args([
                "--out",
                p_flat.to_str().unwrap(),
                "--flat",
                "mi",
                "tavla",
                "do"
            ])
            .status()
            .unwrap()
            .success()
    );
    let a = std::fs::read(&p_pros).unwrap();
    let b = std::fs::read(&p_flat).unwrap();
    assert_ne!(a, b, "prosody must change the rendered audio");
    assert_eq!(&a[0..4], b"RIFF");
    assert_eq!(&b[0..4], b"RIFF");
    let _ = std::fs::remove_file(&p_pros);
    let _ = std::fs::remove_file(&p_flat);
}

#[test]
fn flags_accepted_and_xu_flat_rejected() {
    let path = tmp_wav("flags");
    for flag in ["--dotside", "--buffer", "--xu"] {
        let ok = voksa()
            .args(["--out", path.to_str().unwrap(), flag, "coi", "munje"])
            .status()
            .unwrap()
            .success();
        assert!(ok, "{flag} should render");
    }
    let bad = voksa().args(["--xu", "--flat", "coi"]).output().unwrap();
    assert!(!bad.status.success(), "--xu --flat must be rejected");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn config_file_renders_wav() {
    // A JSON tuning config (as the browser demo exports) replays via --config.
    let cfg = std::env::temp_dir().join(format!("voksa_cfg_{}.json", std::process::id()));
    std::fs::write(
        &cfg,
        r#"{"text":"coi munje","rate":1.5,"declination_end_hz":80.0,"notes":"hi"}"#,
    )
    .unwrap();
    let path = tmp_wav("cfg");
    let status = voksa()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "--out",
            path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "--config should render");
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[0..4], b"RIFF");
    assert!(bytes.len() > 44);
    let _ = std::fs::remove_file(&cfg);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn no_args_prints_usage() {
    let out = voksa().output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
    assert!(stderr.contains("usage"), "usage on stderr: {stderr}");
}

#[test]
fn playback_smoke_or_skip() {
    // No --out: attempts real playback. On CI/WSL with no audio device the CLI
    // prints the canonical no-device message and exits nonzero — treat that as
    // a skip. With a device it must succeed.
    let out = voksa().args(["coi"]).output().unwrap();
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("no usable audio output device"),
            "unexpected playback failure: {stderr}"
        );
    }
}
