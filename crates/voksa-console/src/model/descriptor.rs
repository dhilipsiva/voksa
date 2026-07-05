//! Static UI metadata (keys, labels, units, ranges) + the runtime-seeded
//! descriptor table. Ranges/labels mirror the design bundle
//! (`docs/design/tuning-console/voksa-engine-data.json`); DEFAULTS always
//! come from `voksa_web::default_params()` — never hand-copied.

use super::path::Path;

/// One field's UI metadata (label, unit, slider range/step).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldSpec {
    /// Stable key — mirrors the CLI config JSON field name.
    pub key: &'static str,
    /// UI label (design-bundle wording).
    pub label: &'static str,
    /// Unit suffix shown next to the label.
    pub unit: &'static str,
    /// Slider minimum (descriptor range; a widened value may exceed it).
    pub min: f32,
    /// Slider maximum.
    pub max: f32,
    /// Slider step (widened controls relax to continuous).
    pub step: f32,
}

const fn fs(
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    min: f32,
    max: f32,
    step: f32,
) -> FieldSpec {
    FieldSpec {
        key,
        label,
        unit,
        min,
        max,
        step,
    }
}

/// The 16 runtime knobs, in `k.`-path order: 7 prosody (flat 0..7) then
/// 9 naturalness (flat 440..449).
pub const KNOBS: [FieldSpec; 16] = [
    fs(
        "declination_start_hz",
        "pitch start",
        "Hz",
        60.0,
        220.0,
        1.0,
    ),
    fs("declination_end_hz", "pitch end", "Hz", 50.0, 200.0, 1.0),
    fs(
        "stress_duration_factor",
        "stress stretch",
        "×",
        1.0,
        3.0,
        0.05,
    ),
    fs(
        "stress_f0_excursion_hz",
        "stress pitch boost",
        "Hz",
        0.0,
        60.0,
        1.0,
    ),
    fs("stress_amp_factor", "stress loudness", "×", 1.0, 2.0, 0.05),
    fs("xu_rise_hz", "xu rise", "Hz", 0.0, 80.0, 1.0),
    fs("rate", "speaking rate", "×", 0.5, 2.5, 0.05),
    fs("flutter", "pitch flutter", "FL", 0.0, 100.0, 1.0),
    fs("breath_aspiration", "breathiness", "+", 0.0, 0.5, 0.01),
    fs(
        "baseline_oq_delta",
        "open quotient Δ",
        "soft",
        -0.5,
        0.8,
        0.01,
    ),
    fs(
        "baseline_tilt_delta",
        "spectral tilt Δ",
        "dark",
        -0.9,
        0.9,
        0.01,
    ),
    fs("micro_f0_hz", "vowel-height pitch", "Hz", 0.0, 15.0, 0.5),
    fs(
        "obstruent_f0_hz",
        "consonant pitch kick",
        "Hz",
        0.0,
        20.0,
        0.5,
    ),
    fs("final_lengthen", "final lengthening", "×", 1.0, 2.0, 0.05),
    fs("cluster_shorten", "cluster shortening", "r", 0.0, 0.4, 0.01),
    fs("undershoot", "vowel undershoot", "u", 0.0, 1.0, 0.05),
];

/// The 8 attitudinal deviation fields, in `Deviation::to_array` order —
/// shared by all 7 emotions.
pub const ATT_FIELDS: [FieldSpec; 8] = [
    fs("f0_mean_hz", "pitch shift", "Hz", -40.0, 40.0, 1.0),
    fs("f0_range_mult", "pitch range", "×", 0.0, 3.0, 0.05),
    fs("rate_mult", "tempo", "× slower", 0.5, 2.0, 0.05),
    fs("oq", "open quotient Δ", "breath", -0.6, 0.8, 0.01),
    fs("tilt", "brightness Δ", "tilt", -0.9, 0.9, 0.01),
    fs("di", "creak", "diplo.", 0.0, 1.0, 0.01),
    fs("vibrato_hz", "flutter", "Hz", 0.0, 15.0, 0.5),
    fs("aspiration", "breathiness Δ", "+", 0.0, 0.6, 0.01),
];

/// The 11 steady-target fields, in `Targets::to_array` order — shared by
/// every phoneme's steady/closure/burst blocks.
pub const VT_FIELDS: [FieldSpec; 11] = [
    fs("f1_hz", "F1", "Hz", 100.0, 1200.0, 5.0),
    fs("bw1_hz", "F1 bw", "Hz", 20.0, 500.0, 5.0),
    fs("amp1", "F1 amp", "", 0.0, 1.2, 0.01),
    fs("f2_hz", "F2", "Hz", 400.0, 3000.0, 10.0),
    fs("bw2_hz", "F2 bw", "Hz", 20.0, 1000.0, 5.0),
    fs("amp2", "F2 amp", "", 0.0, 1.2, 0.01),
    fs("f3_hz", "F3", "Hz", 1000.0, 8000.0, 10.0),
    fs("bw3_hz", "F3 bw", "Hz", 20.0, 4000.0, 10.0),
    fs("amp3", "F3 amp", "", 0.0, 1.2, 0.01),
    fs("voicing", "voicing", "", 0.0, 1.0, 0.01),
    fs("aspiration", "aspiration", "", 0.0, 1.0, 0.01),
];

/// Segment duration (steady phonemes, diphthongs, `[h]`).
pub const VT_DUR: FieldSpec = fs("dur_ms", "duration", "ms", 0.0, 400.0, 5.0);
/// Stop closure-phase duration.
pub const VT_CLOSURE_MS: FieldSpec = fs("closure_ms", "closure", "ms", 0.0, 150.0, 1.0);
/// Stop release-burst duration.
pub const VT_BURST_MS: FieldSpec = fs("burst_ms", "burst", "ms", 0.0, 80.0, 1.0);

/// One attitudinal's identity (cmavo key, display label, gloss, try-example).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttKind {
    /// Config-JSON key (the cmavo without the leading dot).
    pub key: &'static str,
    /// Display label (`.ui` …).
    pub label: &'static str,
    /// English gloss.
    pub sub: &'static str,
    /// The try-it example utterance.
    pub example: &'static str,
}

/// The 7 attitudinals, in `AttitudinalKind::ALL` order.
pub const ATT_KINDS: [AttKind; 7] = [
    AttKind {
        key: "ui",
        label: ".ui",
        sub: "happiness / joy",
        example: "coi munje .ui",
    },
    AttKind {
        key: "uu",
        label: ".uu",
        sub: "pity / sadness",
        example: "mi klama .uu",
    },
    AttKind {
        key: "oi",
        label: ".oi",
        sub: "complaint / pain",
        example: "coi munje .oi",
    },
    AttKind {
        key: "ii",
        label: ".ii",
        sub: "fear",
        example: "coi munje .ii",
    },
    AttKind {
        key: "o'o",
        label: ".o'o",
        sub: "patience / calm",
        example: "mi klama .o'o",
    },
    AttKind {
        key: "au",
        label: ".au",
        sub: "desire",
        example: "mi djica .au",
    },
    AttKind {
        key: "o'onai",
        label: ".o'onai",
        sub: "anger",
        example: "mi fengu .o'onai",
    },
];

/// A voice item's parameter shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    /// 11 steady targets + duration (12 slots).
    Steady,
    /// Duration only (1 slot): diphthongs and `[h]`.
    Dur,
    /// Closure targets (11) + burst targets (11) + closure/burst ms (24 slots).
    Stop,
}

impl ItemKind {
    /// Slots this item occupies in the voice-table section.
    pub const fn span(self) -> usize {
        match self {
            ItemKind::Steady => 12,
            ItemKind::Dur => 1,
            ItemKind::Stop => 24,
        }
    }
}

/// One voice-table item (phoneme / diphthong / buffer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceItem {
    /// Config-JSON key (the letter, digraph, `'`, or `buffer`).
    pub key: &'static str,
    /// IPA hint shown on the keycap/editor.
    pub ipa: &'static str,
    /// Parameter shape.
    pub kind: ItemKind,
    /// Index into [`SECTIONS`].
    pub section: u8,
}

const fn vi(key: &'static str, ipa: &'static str, kind: ItemKind, section: u8) -> VoiceItem {
    VoiceItem {
        key,
        ipa,
        kind,
        section,
    }
}

/// A manner-class grouping of voice items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section {
    /// Stable id.
    pub id: &'static str,
    /// English label.
    pub label: &'static str,
    /// Lojban sub-label (design: only real Lojban from the repo).
    pub lojban: &'static str,
}

/// The 6 voice-table sections, in layout order.
pub const SECTIONS: [Section; 6] = [
    Section {
        id: "vowels",
        label: "vowels",
        lojban: "karsna",
    },
    Section {
        id: "diphthongs",
        label: "diphthongs",
        lojban: "relkarsna",
    },
    Section {
        id: "stops",
        label: "stops",
        lojban: "zunsna",
    },
    Section {
        id: "fricatives",
        label: "fricatives",
        lojban: "zunsna",
    },
    Section {
        id: "sonorants",
        label: "nasals + liquids",
        lojban: "zunsna",
    },
    Section {
        id: "other",
        label: "other",
        lojban: "",
    },
];

/// The 41 voice items in EXACT `VoiceTable::to_array` order (the frozen wasm
/// layout: vowels ×12, diphthong durations ×1, stops ×24, fricatives ×12,
/// nasals+liquids ×12, `[h]` ×1, buffer ×12 — 377 slots total).
pub const VOICE_ITEMS: [VoiceItem; 41] = [
    vi("a", "a", ItemKind::Steady, 0),
    vi("e", "ɛ", ItemKind::Steady, 0),
    vi("i", "i", ItemKind::Steady, 0),
    vi("o", "o", ItemKind::Steady, 0),
    vi("u", "u", ItemKind::Steady, 0),
    vi("y", "ə", ItemKind::Steady, 0),
    vi("ai", "ai̯", ItemKind::Dur, 1),
    vi("ei", "ei̯", ItemKind::Dur, 1),
    vi("oi", "oi̯", ItemKind::Dur, 1),
    vi("au", "au̯", ItemKind::Dur, 1),
    vi("ia", "i̯a", ItemKind::Dur, 1),
    vi("ie", "i̯ɛ", ItemKind::Dur, 1),
    vi("ii", "i̯i", ItemKind::Dur, 1),
    vi("io", "i̯o", ItemKind::Dur, 1),
    vi("iu", "i̯u", ItemKind::Dur, 1),
    vi("ua", "u̯a", ItemKind::Dur, 1),
    vi("ue", "u̯ɛ", ItemKind::Dur, 1),
    vi("ui", "u̯i", ItemKind::Dur, 1),
    vi("uo", "u̯o", ItemKind::Dur, 1),
    vi("uu", "u̯u", ItemKind::Dur, 1),
    vi("iy", "i̯ə", ItemKind::Dur, 1),
    vi("uy", "u̯ə", ItemKind::Dur, 1),
    vi("p", "p", ItemKind::Stop, 2),
    vi("t", "t", ItemKind::Stop, 2),
    vi("k", "k", ItemKind::Stop, 2),
    vi("b", "b", ItemKind::Stop, 2),
    vi("d", "d", ItemKind::Stop, 2),
    vi("g", "ɡ", ItemKind::Stop, 2),
    vi("f", "f", ItemKind::Steady, 3),
    vi("v", "v", ItemKind::Steady, 3),
    vi("s", "s", ItemKind::Steady, 3),
    vi("z", "z", ItemKind::Steady, 3),
    vi("c", "ʃ", ItemKind::Steady, 3),
    vi("j", "ʒ", ItemKind::Steady, 3),
    vi("x", "x", ItemKind::Steady, 3),
    vi("m", "m", ItemKind::Steady, 4),
    vi("n", "n", ItemKind::Steady, 4),
    vi("l", "l", ItemKind::Steady, 4),
    vi("r", "r", ItemKind::Steady, 4),
    vi("'", "h", ItemKind::Dur, 5),
    vi("buffer", "ɪ", ItemKind::Steady, 5),
];

/// One flat-index parameter's full descriptor: path + UI metadata + the
/// engine-seeded default.
#[derive(Debug, Clone, PartialEq)]
pub struct Descriptor {
    /// Index in the frozen 449-float layout.
    pub idx: usize,
    /// The parameter's path.
    pub path: Path,
    /// UI label (contextual: stop slots gain `closure `/`burst ` prefixes).
    pub label: String,
    /// Unit suffix.
    pub unit: &'static str,
    /// Descriptor slider minimum.
    pub min: f32,
    /// Descriptor slider maximum.
    pub max: f32,
    /// Slider step.
    pub step: f32,
    /// The engine default (seeded from `voksa_web::default_params()`).
    pub default: f32,
    /// Help-registry key resolved by the `?` popover.
    pub help_key: String,
}

/// The full 449-descriptor table, seeded from the engine's default block.
#[derive(Debug, Clone, PartialEq)]
pub struct Descriptors {
    list: Vec<Descriptor>,
}

impl Descriptors {
    /// Build the table from the engine's canonical default block
    /// (`voksa_web::default_params()`). Errors if the block is not exactly
    /// the frozen 449-float layout.
    pub fn from_defaults(defaults: &[f32]) -> Result<Descriptors, String> {
        if defaults.len() != voksa_web::FULL_PARAM_COUNT {
            return Err(format!(
                "engine default block must be {} floats, got {}",
                voksa_web::FULL_PARAM_COUNT,
                defaults.len()
            ));
        }
        let mut list = Vec::with_capacity(defaults.len());
        for (idx, &default) in defaults.iter().enumerate() {
            let path = Path::from_flat(idx);
            let (spec, label, help_key) = match path {
                Path::Knob(k) => {
                    let spec = &KNOBS[k as usize];
                    let section = if (k as usize) < 7 {
                        "prosody"
                    } else {
                        "naturalness"
                    };
                    (
                        spec,
                        spec.label.to_string(),
                        format!("{section}.{}", spec.key),
                    )
                }
                Path::Att { field, .. } => {
                    let spec = &ATT_FIELDS[field as usize];
                    (
                        spec,
                        spec.label.to_string(),
                        format!("att.fields.{}", spec.key),
                    )
                }
                Path::Voice { item, slot } => {
                    let slot = slot as usize;
                    let (spec, label) = match VOICE_ITEMS[item as usize].kind {
                        ItemKind::Steady if slot < 11 => {
                            (&VT_FIELDS[slot], VT_FIELDS[slot].label.to_string())
                        }
                        ItemKind::Steady => (&VT_DUR, VT_DUR.label.to_string()),
                        ItemKind::Dur => (&VT_DUR, VT_DUR.label.to_string()),
                        ItemKind::Stop if slot < 11 => (
                            &VT_FIELDS[slot],
                            format!("closure {}", VT_FIELDS[slot].label),
                        ),
                        ItemKind::Stop if slot < 22 => (
                            &VT_FIELDS[slot - 11],
                            format!("burst {}", VT_FIELDS[slot - 11].label),
                        ),
                        ItemKind::Stop if slot == 22 => {
                            (&VT_CLOSURE_MS, VT_CLOSURE_MS.label.to_string())
                        }
                        ItemKind::Stop => (&VT_BURST_MS, VT_BURST_MS.label.to_string()),
                    };
                    (spec, label, format!("vt.fields.{}", spec.key))
                }
            };
            list.push(Descriptor {
                idx,
                path,
                label,
                unit: spec.unit,
                min: spec.min,
                max: spec.max,
                step: spec.step,
                default,
                help_key,
            });
        }
        Ok(Descriptors { list })
    }

    /// The descriptor at a flat index.
    pub fn get(&self, idx: usize) -> &Descriptor {
        &self.list[idx]
    }

    /// Total descriptor count (always 449 once built).
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Whether the table is empty (never true once built).
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// The flat-index range a voice item occupies.
    pub fn voice_item_range(&self, item: usize) -> core::ops::Range<usize> {
        let start = Path::Voice {
            item: item as u8,
            slot: 0,
        }
        .flat_index();
        start..start + VOICE_ITEMS[item].kind.span()
    }

    /// The flat-index range an attitudinal's 8 fields occupy.
    pub fn att_range(&self, kind: usize) -> core::ops::Range<usize> {
        let start = Path::Att {
            kind: kind as u8,
            field: 0,
        }
        .flat_index();
        start..start + ATT_FIELDS.len()
    }

    /// Flat index of a knob (0..16 in `KNOBS` order).
    pub fn knob_index(&self, knob: usize) -> usize {
        Path::Knob(knob as u8).flat_index()
    }

    /// The flat-index range of the whole voice-table section (63..440) — the
    /// scope of the table-level reset.
    pub fn voice_range(&self) -> core::ops::Range<usize> {
        let start = self.voice_item_range(0).start;
        let end = self.voice_item_range(VOICE_ITEMS.len() - 1).end;
        start..end
    }
}
