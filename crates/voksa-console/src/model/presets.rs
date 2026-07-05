//! Presets: reset-to-defaults plus knob overrides. Flags are PRESERVED — a
//! deliberate deviation from the reference page, specified by the design.

use super::descriptor::Descriptors;
use super::logic::WritePlan;

/// A named preset: knob-key overrides applied over a full reset.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    /// Display name.
    pub name: &'static str,
    /// `(knob key, value)` overrides.
    pub over: &'static [(&'static str, f32)],
}

/// The 6 presets (design bundle order).
pub const PRESETS: [Preset; 6] = [
    Preset {
        name: "Default",
        over: &[],
    },
    Preset {
        name: "Sing-song",
        over: &[
            ("declination_start_hz", 150.0),
            ("declination_end_hz", 80.0),
            ("stress_f0_excursion_hz", 35.0),
            ("stress_duration_factor", 1.8),
        ],
    },
    Preset {
        name: "Monotone",
        over: &[
            ("declination_start_hz", 110.0),
            ("declination_end_hz", 110.0),
            ("stress_f0_excursion_hz", 3.0),
            ("stress_amp_factor", 1.05),
            ("stress_duration_factor", 1.1),
        ],
    },
    Preset {
        name: "Fast",
        over: &[("rate", 1.7)],
    },
    Preset {
        name: "Slow & calm",
        over: &[
            ("rate", 0.8),
            ("stress_f0_excursion_hz", 12.0),
            ("declination_start_hz", 110.0),
            ("declination_end_hz", 90.0),
        ],
    },
    Preset {
        name: "Naturalness off",
        over: &[
            ("flutter", 0.0),
            ("breath_aspiration", 0.0),
            ("baseline_oq_delta", 0.0),
            ("baseline_tilt_delta", 0.0),
            ("micro_f0_hz", 0.0),
            ("obstruent_f0_hz", 0.0),
            ("final_lengthen", 1.0),
            ("cluster_shorten", 0.0),
            ("undershoot", 0.0),
        ],
    },
];

/// The write plan for a preset: EVERY parameter back to its engine default,
/// then the preset's knob overrides. Text and flags are untouched.
pub fn apply_preset(desc: &Descriptors, name: &str) -> Option<WritePlan> {
    let _ = (desc, name);
    None // stub — C1 green
}
