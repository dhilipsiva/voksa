//! THE contract test: whatever the console exports must replay identically
//! through `voksa --config` — enforced by parsing the export with the CLI's
//! REAL `Config` and comparing the resolved option structs/tables.

use voksa_cli::config::Config;
use voksa_console::model::{Descriptors, ExportInputs, Flags, export};
use voksa_core::attitudinal::AttitudinalKind;
use voksa_core::phonemes::{Consonant, STOP_ORDER, VoiceTable, Vowel};
use voksa_core::prosody::ProsodyOptions;

fn export_with(values: &[f32], flags: Flags) -> String {
    let d = Descriptors::from_defaults(&voksa_web::default_params()).unwrap();
    export(
        &d,
        &ExportInputs {
            values,
            text: "coi la djan. cu klama",
            flags,
            notes: "round-trip",
            phonetics: "coi ‖ la DJAN. ‖ cu KLA.ma",
            sample_rate: 48_000,
        },
    )
}

#[test]
fn default_export_parses_to_default_cli_options() {
    let json = export_with(&voksa_web::default_params(), Flags::default());
    let cfg = Config::from_json(&json).expect("CLI parses the console export");
    assert_eq!(cfg.text, "coi la djan. cu klama");
    assert_eq!(cfg.prosody_options(), ProsodyOptions::default());
    assert_eq!(cfg.attitudinal_table(), Default::default());
    assert_eq!(cfg.voice_table(), VoiceTable::default());
    assert!(!cfg.compile_options().dotside && !cfg.compile_options().buffer);
}

#[test]
fn dirty_export_replays_through_the_cli_parser() {
    let mut values = voksa_web::default_params();
    values[6] = 1.15; // rate
    values[440] = 40.0; // flutter
    values[63] = 1400.0; // a.f1 (out of slider range — must survive verbatim)
    values[218] = 1.0; // k burst amp3
    values[221] = 52.0; // k closure_ms
    values[7] = 31.0; // ui f0_mean
    let json = export_with(
        &values,
        Flags {
            xu: true,
            ..Default::default()
        },
    );
    let cfg = Config::from_json(&json).expect("CLI parses the console export");

    let p = cfg.prosody_options();
    assert_eq!(p.rate, 1.15);
    assert_eq!(p.flutter, 40.0);
    assert!(p.xu_rise, "xu flag maps onto the prosody options");
    assert_eq!(
        p.declination_start_hz, 120.0,
        "untouched knobs stay default"
    );

    let t = cfg.voice_table();
    assert_eq!(
        t.vowels[Vowel::A.index()].targets.formants[0].freq_hz,
        1400.0
    );
    let ki = STOP_ORDER.iter().position(|&c| c == Consonant::K).unwrap();
    assert_eq!(t.stops[ki].burst.formants[2].amp, 1.0);
    assert_eq!(t.stops[ki].closure_ms, 52.0);
    assert_eq!(t.stops[ki].burst_ms, 25.0, "untouched timing stays pinned");

    let a = cfg.attitudinal_table();
    assert_eq!(a.get(AttitudinalKind::Joy).f0_mean_hz, 31.0);
    assert_eq!(
        a.get(AttitudinalKind::Joy).f0_range_mult,
        AttitudinalKind::Joy.deviation().f0_range_mult,
        "unnamed fields stay pinned"
    );
}
