use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let cmd = args.next();
    // `cargo xtask oracle -- "coi munje"` arrives as ["oracle", "--", "coi munje"].
    let rest: Vec<String> = args.filter(|a| a.as_str() != "--").collect();
    match cmd.as_deref() {
        Some("oracle") => oracle(&rest),
        Some("wasm-size") => todo_stub("wasm-size", "Phase 9"),
        Some("listening-battery") => todo_stub("listening-battery", "Phase 7"),
        _ => {
            eprintln!("usage: cargo xtask <oracle|wasm-size|listening-battery> [args]");
            ExitCode::FAILURE
        }
    }
}

fn todo_stub(name: &str, phase: &str) -> ExitCode {
    eprintln!("error: `cargo xtask {name}` is not implemented until {phase}");
    ExitCode::FAILURE
}

/// Render `text` with the eSpeak NG Lojban voice into fixtures/oracle/<slug>.wav
/// and verify the output is a non-empty RIFF/WAVE file.
///
/// eSpeak NG is GPLv3 and is used strictly as an OUT-OF-PROCESS oracle; its
/// code, phoneme tables, and data files must never be copied into this repo.
fn oracle(args: &[String]) -> ExitCode {
    let text = args.join(" ");
    if text.trim().is_empty() {
        eprintln!("usage: cargo xtask oracle -- \"<lojban text>\"");
        return ExitCode::FAILURE;
    }
    let dir = workspace_root().join("fixtures/oracle");
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("error: cannot create {}: {e}", dir.display());
        return ExitCode::FAILURE;
    }
    let out = dir.join(format!("{}.wav", slugify(&text)));

    let status = Command::new("espeak-ng")
        .args(["-v", "jbo", "-w"])
        .arg(&out)
        .arg(&text)
        .status();
    match status {
        Err(e) => {
            eprintln!("error: failed to run espeak-ng (is it on PATH? use `nix develop`): {e}");
            return ExitCode::FAILURE;
        }
        Ok(s) if !s.success() => {
            eprintln!("error: espeak-ng exited with {s}");
            return ExitCode::FAILURE;
        }
        Ok(_) => {}
    }

    let bytes = match fs::read(&out) {
        Ok(b) => b,
        Err(e) => {
            eprintln!(
                "error: espeak-ng produced no readable file at {}: {e}",
                out.display()
            );
            return ExitCode::FAILURE;
        }
    };
    match validate_riff(&bytes) {
        Ok(()) => {
            println!("oracle: wrote {} ({} bytes)", out.display(), bytes.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {} is not a valid WAV: {e}", out.display());
            ExitCode::FAILURE
        }
    }
}

/// xtask lives at <workspace>/xtask, so the root is one level up.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent directory")
        .to_path_buf()
}

/// "coi munje" -> "coi-munje"; ".i mi'e la voksa." -> "i-mi-e-la-voksa".
fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(c.to_ascii_lowercase());
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }
    if slug.is_empty() {
        slug.push_str("utterance");
    }
    slug
}

/// Minimal WAV sanity check: >= 12 bytes, "RIFF" magic, "WAVE" form type.
fn validate_riff(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 12 {
        return Err(format!(
            "only {} bytes (need >= 12 for a RIFF header)",
            bytes.len()
        ));
    }
    if &bytes[0..4] != b"RIFF" {
        return Err("missing RIFF magic at offset 0".into());
    }
    if &bytes[8..12] != b"WAVE" {
        return Err("missing WAVE form type at offset 8".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{slugify, validate_riff};

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("coi munje"), "coi-munje");
    }

    #[test]
    fn slugify_lojban_punctuation() {
        assert_eq!(slugify(".i mi'e la voksa."), "i-mi-e-la-voksa");
        assert_eq!(slugify("  coi   MUNJE  "), "coi-munje");
        assert_eq!(slugify("...."), "utterance");
    }

    #[test]
    fn riff_accepts_valid_header() {
        let mut wav = Vec::from(*b"RIFF");
        wav.extend_from_slice(&36u32.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        assert!(validate_riff(&wav).is_ok());
    }

    #[test]
    fn riff_rejects_short_and_bogus() {
        assert!(validate_riff(b"RIFF").is_err());
        assert!(validate_riff(b"OggS\x00\x00\x00\x00vorb").is_err());
    }
}
