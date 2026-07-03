//! `voksa` — native CLI: speak Lojban aloud, or render a WAV with `--out`.

use std::process::ExitCode;

use voksa_cli::{args, playback, wav};
use voksa_core::compiler::{CompileError, CompileOptions};
use voksa_core::prosody::ProsodyOptions;
use voksa_engine_klattsch::{SAMPLE_RATE, render_utterance, render_utterance_prosodic};

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return ExitCode::SUCCESS;
    }
    if argv.iter().any(|a| a == "--version" || a == "-V") {
        println!("voksa {}", voksa_core::VERSION);
        return ExitCode::SUCCESS;
    }

    let parsed = match args::parse(argv.into_iter()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\n");
            print_usage();
            return ExitCode::FAILURE;
        }
    };

    let copts = CompileOptions {
        dotside: parsed.dotside,
        buffer: parsed.buffer,
    };
    let popts = ProsodyOptions { xu_rise: parsed.xu };
    let render_at = |sr: u32| -> Result<Vec<f32>, CompileError> {
        if parsed.flat {
            render_utterance(&parsed.text, &copts, sr)
        } else {
            render_utterance_prosodic(&parsed.text, &copts, &popts, sr)
        }
    };

    match &parsed.out {
        // Offline render: fixed 48 kHz, no audio device touched (CI-safe).
        Some(path) => match render_at(SAMPLE_RATE) {
            Ok(samples) => match wav::write_wav(path, &samples, SAMPLE_RATE) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("error: writing {}: {e}", path.display());
                    ExitCode::FAILURE
                }
            },
            Err(e) => {
                eprintln!("error: {}", format_compile_error(&e));
                ExitCode::FAILURE
            }
        },
        // Live playback: render at the negotiated device rate.
        None => match playback::play(|sr| render_at(sr).map_err(|e| format_compile_error(&e))) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
    }
}

fn format_compile_error(e: &CompileError) -> String {
    match e {
        CompileError::Word { word, error } => format!("cannot speak {word:?}: {error:?}"),
        CompileError::MalformedNumber { figure, error } => {
            format!("bad number {figure:?}: {error:?}")
        }
        CompileError::Empty => "no speakable words in the input".to_string(),
    }
}

fn print_usage() {
    eprintln!("voksa - rule-based Lojban speech synthesizer");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    voksa [FLAGS] <text>...              speak <text> aloud");
    eprintln!("    voksa --out <FILE> [FLAGS] <text>... render a WAV instead");
    eprintln!();
    eprintln!("FLAGS:");
    eprintln!("    --out <FILE>   render 48 kHz mono WAV to FILE (no audio device needed)");
    eprintln!("    --flat         disable prosody (Phase-5 baseline)");
    eprintln!("    --xu           terminal question rise (not with --flat)");
    eprintln!("    --dotside      leading pause before every cmevla");
    eprintln!("    --buffer       epenthetic buffer vowels for clarity in noise");
    eprintln!("    --help, --version");
}
