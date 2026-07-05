use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let cmd = args.next();
    // `cargo xtask oracle -- "coi munje"` arrives as ["oracle", "--", "coi munje"].
    let rest: Vec<String> = args.filter(|a| a.as_str() != "--").collect();
    match cmd.as_deref() {
        Some("oracle") => oracle(&rest),
        Some("wasm-size") => wasm_size(),
        Some("listening-battery") => listening_battery(),
        Some("attitudinal-battery") => attitudinal_battery(),
        Some("fuzz") => fuzz(&rest),
        _ => {
            eprintln!(
                "usage: cargo xtask <oracle|wasm-size|listening-battery|attitudinal-battery|fuzz> [args]"
            );
            ExitCode::FAILURE
        }
    }
}

struct BatteryEntry {
    slug: &'static str,
    text: &'static str,
    dotside: bool,
    buffer: bool,
    xu: bool,
}

/// Fixed utterance set — slugs stay stable across phases so batteries diff.
const BATTERY: &[BatteryEntry] = &[
    BatteryEntry {
        slug: "coi-munje",
        text: "coi munje",
        dotside: false,
        buffer: false,
        xu: false,
    },
    BatteryEntry {
        slug: "le-prenu",
        text: "le prenu cu klama",
        dotside: false,
        buffer: false,
        xu: false,
    },
    BatteryEntry {
        slug: "mi-zgana",
        text: "mi zgana le sance",
        dotside: false,
        buffer: false,
        xu: false,
    },
    BatteryEntry {
        slug: "la-djan",
        text: "coi la djan. cu klama",
        dotside: false,
        buffer: false,
        xu: false,
    },
    BatteryEntry {
        slug: "li-pi",
        text: "li 3.14",
        dotside: false,
        buffer: false,
        xu: false,
    },
    BatteryEntry {
        slug: "nelci-buffer",
        text: "mi nelci le zdani",
        dotside: false,
        buffer: true,
        xu: false,
    },
    BatteryEntry {
        slug: "djan-dotside",
        text: "coi la djan. cu klama",
        dotside: true,
        buffer: false,
        xu: false,
    },
    BatteryEntry {
        slug: "xu-rise",
        text: "xu do klama",
        dotside: false,
        buffer: false,
        xu: true,
    },
    BatteryEntry {
        slug: "declarative",
        text: "mi tavla do bau la lojban.",
        dotside: false,
        buffer: false,
        xu: false,
    },
    BatteryEntry {
        slug: "djosefin",
        text: "la DJOsefin. klama",
        dotside: false,
        buffer: false,
        xu: false,
    },
];

/// One CP2 attitudinal item: the utterance WITH a UI cmavo, the SAME words
/// without it (the neutral A/B baseline that isolates the voice-quality
/// coloring), and the human-readable emotion for the scoring sheet.
struct AttitudinalEntry {
    slug: &'static str,
    text: &'static str,
    base: &'static str,
    emotion: &'static str,
}

/// The Phase-10 attitudinal battery — each exercises a distinct new engine
/// capability (F0 mean/range, OQ, spectral tilt, diplophonia, vibrato).
const ATTITUDINAL_BATTERY: &[AttitudinalEntry] = &[
    AttitudinalEntry {
        slug: "joy-ui",
        text: "coi munje .ui",
        base: "coi munje",
        emotion: "joy (.ui)",
    },
    AttitudinalEntry {
        slug: "complaint-oi",
        text: "coi munje .oi",
        base: "coi munje",
        emotion: "complaint / pain (.oi)",
    },
    AttitudinalEntry {
        slug: "fear-ii",
        text: "coi munje .ii",
        base: "coi munje",
        emotion: "fear (.ii)",
    },
    AttitudinalEntry {
        slug: "sadness-uu",
        text: "mi klama .uu",
        base: "mi klama",
        emotion: "sadness / pity (.uu)",
    },
    AttitudinalEntry {
        slug: "patience-oo",
        text: "mi klama .o'o",
        base: "mi klama",
        emotion: "patience / calm (.o'o)",
    },
    AttitudinalEntry {
        slug: "desire-au",
        text: "mi djica .au",
        base: "mi djica",
        emotion: "desire (.au)",
    },
    AttitudinalEntry {
        slug: "anger-oonai",
        text: "mi fengu .o'onai",
        base: "mi fengu",
        emotion: "anger (.o'onai)",
    },
];

/// Render the CP2 attitudinal battery to artifacts/listening/phase10/: per item
/// the affect-colored render (prosody + attitudinal overlay), the neutral base
/// render (same words minus the UI cmavo — isolates the coloring), and the
/// eSpeak-NG jbo oracle, plus an index.html scoring page (recognizability +
/// naturalness). Then STOP — the human scores and tags phase10-complete.
fn attitudinal_battery() -> ExitCode {
    use voksa_core::compiler::CompileOptions;
    use voksa_core::prosody::ProsodyOptions;
    use voksa_engine_klattsch::{SAMPLE_RATE, render_utterance_prosodic};

    let dir = workspace_root().join("artifacts/listening/phase10");
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("error: cannot create {}: {e}", dir.display());
        return ExitCode::FAILURE;
    }
    let sr = SAMPLE_RATE;
    let copts = CompileOptions::default();
    let popts = ProsodyOptions::default();
    let mut rows = String::new();
    for (i, e) in ATTITUDINAL_BATTERY.iter().enumerate() {
        let affect = match render_utterance_prosodic(e.text, &copts, &popts, sr) {
            Ok(s) => s,
            Err(err) => {
                eprintln!("error: {}: {err:?}", e.slug);
                return ExitCode::FAILURE;
            }
        };
        let neutral = match render_utterance_prosodic(e.base, &copts, &popts, sr) {
            Ok(s) => s,
            Err(err) => {
                eprintln!("error: {} (base): {err:?}", e.slug);
                return ExitCode::FAILURE;
            }
        };
        for (kind, samples) in [("voksa", &affect), ("neutral", &neutral)] {
            let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            if peak >= 1.0 {
                eprintln!("error: {}_{} clips (peak {peak:.3})", kind, e.slug);
                return ExitCode::FAILURE;
            }
            voksa_testkit::write_wav(dir.join(format!("{kind}_{}.wav", e.slug)), samples, sr);
        }
        let oracle_path = dir.join(format!("oracle_{}.wav", e.slug));
        let status = Command::new("espeak-ng")
            .args(["-v", "jbo", "-w"])
            .arg(&oracle_path)
            .arg(e.text.to_ascii_lowercase())
            .status();
        match status {
            Ok(s) if s.success() => {}
            other => {
                eprintln!("error: espeak-ng oracle for {}: {other:?}", e.slug);
                return ExitCode::FAILURE;
            }
        }
        rows.push_str(&format!(
            r#"<tr><td>{n}</td><td>{emotion}</td><td><code>{text}</code></td>
<td><audio controls src="voksa_{slug}.wav"></audio></td>
<td><audio controls src="neutral_{slug}.wav"></audio></td>
<td><audio controls src="oracle_{slug}.wav"></audio></td>
<td><input type="text" class="heard" data-slug="{slug}" size="12" placeholder="emotion?"></td>
<td><input type="number" min="1" max="5" class="mos-n" data-slug="{slug}"></td>
<td><input type="text" class="notes" data-slug="{slug}" size="20"></td></tr>
"#,
            n = i + 1,
            emotion = e.emotion,
            text = e.text,
            slug = e.slug,
        ));
    }
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>voksa CP2 attitudinal battery (phase 10)</title>
<style>body{{font-family:sans-serif;margin:2em}}table{{border-collapse:collapse}}td,th{{border:1px solid #ccc;padding:6px}}textarea{{width:100%;height:12em}}</style></head><body>
<h1>voksa — Listening Checkpoint 2 (Phase 10, attitudinals)</h1>
<p>For each row: play <b>voksa (affect)</b> and its <b>neutral</b> baseline (same words, no emotion marker).
In <b>heard</b>, write the emotion you actually perceive (blind if you can — is it recognizable?); rate <b>MOS nat</b> 1–5.
The eSpeak oracle is a plain reference (it does not voice attitude). The overlay is INVENTED / non-normative.
When done, press the button and paste the markdown into <code>docs/listening/phase10.md</code>.</p>
<table><tr><th>#</th><th>intended emotion</th><th>text</th><th>voksa (affect)</th><th>neutral (base)</th><th>eSpeak oracle</th><th>heard emotion</th><th>MOS nat.</th><th>notes</th></tr>
{rows}</table>
<p><button onclick="collect()">Build markdown results</button></p>
<textarea id="out" readonly placeholder="results appear here — copy into docs/listening/phase10.md"></textarea>
<script>
function collect() {{
  const slugs = [...new Set([...document.querySelectorAll('.mos-n')].map(e => e.dataset.slug))];
  let md = '| slug | heard emotion | MOS naturalness | notes |\n|---|---|---|---|\n';
  for (const s of slugs) {{
    const v = c => (document.querySelector(`.${{c}}[data-slug="${{s}}"]`) || {{}}).value || '';
    md += `| ${{s}} | ${{v('heard')}} | ${{v('mos-n')}} | ${{v('notes')}} |\n`;
  }}
  document.getElementById('out').value = md;
}}
</script></body></html>
"#
    );
    if let Err(e) = fs::write(dir.join("index.html"), html) {
        eprintln!("error: writing index.html: {e}");
        return ExitCode::FAILURE;
    }
    println!(
        "attitudinal-battery: wrote {} items x3 WAVs + index.html to {}",
        ATTITUDINAL_BATTERY.len(),
        dir.display()
    );
    ExitCode::SUCCESS
}

/// Render the listening battery: per utterance a prosodic render, a flat
/// (Phase-5) baseline for within-phase ABX, and the eSpeak-NG jbo oracle,
/// plus an index.html A/B page with MOS note-taking.
fn listening_battery() -> ExitCode {
    use voksa_core::compiler::CompileOptions;
    use voksa_core::prosody::ProsodyOptions;

    let dir = workspace_root().join("artifacts/listening/phase7");
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("error: cannot create {}: {e}", dir.display());
        return ExitCode::FAILURE;
    }
    let sr = voksa_engine_klattsch::SAMPLE_RATE;
    let mut rows = String::new();
    for entry in BATTERY {
        let copts = CompileOptions {
            dotside: entry.dotside,
            buffer: entry.buffer,
        };
        let popts = ProsodyOptions {
            xu_rise: entry.xu,
            ..Default::default()
        };
        let prosodic = match voksa_engine_klattsch::render_utterance_prosodic(
            entry.text, &copts, &popts, sr,
        ) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {}: {e:?}", entry.slug);
                return ExitCode::FAILURE;
            }
        };
        let flat = match voksa_engine_klattsch::render_utterance(entry.text, &copts, sr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {}: {e:?}", entry.slug);
                return ExitCode::FAILURE;
            }
        };
        for (kind, samples) in [("voksa", &prosodic), ("flat", &flat)] {
            let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            if peak >= 1.0 {
                eprintln!("error: {}_{} clips (peak {peak:.3})", kind, entry.slug);
                return ExitCode::FAILURE;
            }
            voksa_testkit::write_wav(dir.join(format!("{kind}_{}.wav", entry.slug)), samples, sr);
        }
        // eSpeak NG oracle (out-of-process; GPLv3 tool, never linked).
        let oracle_path = dir.join(format!("oracle_{}.wav", entry.slug));
        let status = Command::new("espeak-ng")
            .args(["-v", "jbo", "-w"])
            .arg(&oracle_path)
            .arg(entry.text.to_ascii_lowercase())
            .status();
        match status {
            Ok(s) if s.success() => {}
            other => {
                eprintln!("error: espeak-ng oracle for {}: {other:?}", entry.slug);
                return ExitCode::FAILURE;
            }
        }
        rows.push_str(&format!(
            r#"<tr><td>{n}</td><td><code>{text}</code>{flags}</td>
<td><audio controls src="voksa_{slug}.wav"></audio></td>
<td><audio controls src="flat_{slug}.wav"></audio></td>
<td><audio controls src="oracle_{slug}.wav"></audio></td>
<td><input type="number" min="1" max="5" class="mos-i" data-slug="{slug}"></td>
<td><input type="number" min="1" max="5" class="mos-n" data-slug="{slug}"></td>
<td><select class="abx" data-slug="{slug}"><option value=""></option><option>prosodic</option><option>flat</option><option>tie</option></select></td>
<td><input type="text" class="notes" data-slug="{slug}" size="24"></td></tr>
"#,
            n = rows.matches("<tr>").count() + 1,
            text = entry.text,
            flags = {
                let mut f = String::new();
                if entry.dotside { f.push_str(" <b>[dotside]</b>"); }
                if entry.buffer { f.push_str(" <b>[buffer]</b>"); }
                if entry.xu { f.push_str(" <b>[xu rise]</b>"); }
                f
            },
            slug = entry.slug,
        ));
    }
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>voksa CP1 listening battery (phase 7)</title>
<style>body{{font-family:sans-serif;margin:2em}}table{{border-collapse:collapse}}td,th{{border:1px solid #ccc;padding:6px}}textarea{{width:100%;height:12em}}</style></head><body>
<h1>voksa — Listening Checkpoint 1 (Phase 7)</h1>
<p>Rate each utterance: MOS 1–5 for intelligibility and naturalness; ABX = which of prosodic/flat sounds better.
Compare with the eSpeak-NG jbo oracle for reference. When done, press the button and paste the markdown into <code>docs/listening/phase7.md</code>.</p>
<table><tr><th>#</th><th>text</th><th>voksa (prosodic)</th><th>flat (no prosody)</th><th>eSpeak oracle</th><th>MOS int.</th><th>MOS nat.</th><th>ABX</th><th>notes</th></tr>
{rows}</table>
<p><button onclick="collect()">Build markdown results</button></p>
<textarea id="out" readonly placeholder="results appear here — copy into docs/listening/phase7.md"></textarea>
<script>
function collect() {{
  const slugs = [...new Set([...document.querySelectorAll('.mos-i')].map(e => e.dataset.slug))];
  let md = '| slug | MOS intelligibility | MOS naturalness | ABX | notes |\n|---|---|---|---|---|\n';
  for (const s of slugs) {{
    const v = c => (document.querySelector(`.${{c}}[data-slug="${{s}}"]`) || {{}}).value || '';
    md += `| ${{s}} | ${{v('mos-i')}} | ${{v('mos-n')}} | ${{v('abx')}} | ${{v('notes')}} |\n`;
  }}
  document.getElementById('out').value = md;
}}
</script></body></html>
"#
    );
    if let Err(e) = fs::write(dir.join("index.html"), html) {
        eprintln!("error: writing index.html: {e}");
        return ExitCode::FAILURE;
    }
    println!(
        "listening-battery: wrote {} utterances x3 WAVs + index.html to {}",
        BATTERY.len(),
        dir.display()
    );
    ExitCode::SUCCESS
}

/// Deep fuzz run (Phase-11 W1): the proptest suites (`tests/fuzz.rs` in
/// voksa-core + voksa-web) at PROPTEST_CASES=65536 (override: `--cases N`).
/// The normal CI `test` job already runs them at the proptest default (256);
/// this is the weekly `fuzz-deep` CI job + local soak. Note: the render-bound
/// voksa-web suite self-caps at 1024 cases (documented in its source).
fn fuzz(args: &[String]) -> ExitCode {
    let mut cases = 65_536u32;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--cases" {
            match it.next().and_then(|v| v.parse().ok()) {
                Some(n) => cases = n,
                None => {
                    eprintln!("usage: cargo xtask fuzz [--cases N]");
                    return ExitCode::FAILURE;
                }
            }
        }
    }
    println!("fuzz: PROPTEST_CASES={cases} (voksa-web suite self-caps at 1024)");
    let status = Command::new("cargo")
        .args([
            "nextest",
            "run",
            "--workspace",
            "-E",
            "binary(fuzz)",
            "--no-fail-fast",
        ])
        .env("PROPTEST_CASES", cases.to_string())
        .current_dir(workspace_root())
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("fuzz: all suites green at {cases} cases");
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("error: fuzz run failed: {other:?} (commit any new proptest-regressions)");
            ExitCode::FAILURE
        }
    }
}

/// Build the web crate for the browser (wasm-pack runs wasm-opt -Oz per
/// crates/voksa-web/Cargo.toml), then assert the gzipped `.wasm` is under
/// budget and declares ZERO imports — the AudioWorklet instantiates it with an
/// empty import object (`new WebAssembly.Instance(module, {})`), so any import
/// (e.g. a stray wasm-bindgen `String` on the surface) would break it.
fn wasm_size() -> ExitCode {
    // Measured 2026-07-03 (engine + prosody + simd128): 33_073 B gzip.
    // Budget = ~1.3× that, leaving headroom for engine/prosody growth.
    const WASM_GZIP_BUDGET: u64 = 43_000;
    let root = workspace_root();

    let built = Command::new("wasm-pack")
        .args(["build", "--release", "--target", "web"])
        .arg(root.join("crates/voksa-web"))
        .status();
    match built {
        Ok(s) if s.success() => {}
        other => {
            eprintln!("error: `wasm-pack build` failed: {other:?}");
            return ExitCode::FAILURE;
        }
    }

    let wasm = root.join("crates/voksa-web/pkg/voksa_web_bg.wasm");
    if !wasm.exists() {
        eprintln!("error: {} not found", wasm.display());
        return ExitCode::FAILURE;
    }

    match wasm_import_count(&wasm) {
        Ok(0) => {}
        Ok(n) => {
            eprintln!(
                "error: wasm declares {n} import(s); the AudioWorklet needs zero \
                 (did a wasm-bindgen String/js_sys type reach the public surface?)"
            );
            return ExitCode::FAILURE;
        }
        // Best-effort: if wasm-dis is unavailable (e.g. binaryen not installed
        // in a minimal CI), warn but still enforce the size gate below.
        Err(e) => {
            eprintln!(
                "warning: could not inspect wasm imports ({e}); skipping the zero-imports check"
            );
        }
    }

    let gzip = match gzip_size(&wasm) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("error: gzipping {}: {e}", wasm.display());
            return ExitCode::FAILURE;
        }
    };
    println!("wasm size (gzip): {gzip} bytes (budget: {WASM_GZIP_BUDGET} bytes)");
    if gzip > WASM_GZIP_BUDGET {
        eprintln!("error: over budget by {} bytes", gzip - WASM_GZIP_BUDGET);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Gzipped size in bytes, via the `gzip` in the dev shell (keeps xtask dep-free;
/// `gzip -9 -c` writes the compressed stream to stdout, which we just measure).
fn gzip_size(path: &Path) -> Result<u64, String> {
    let out = Command::new("gzip")
        .args(["-9", "-c"])
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    Ok(out.stdout.len() as u64)
}

/// Number of `(import ...)` entries the module declares, via `wasm-dis`
/// (binaryen, in the dev shell).
fn wasm_import_count(path: &Path) -> Result<usize, String> {
    let out = Command::new("wasm-dis")
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.trim_start().starts_with("(import "))
        .count())
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
