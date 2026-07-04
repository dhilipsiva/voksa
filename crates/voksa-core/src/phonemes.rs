//! Lojban phoneme inventory and acoustic parameter IR.
//!
//! This is voksa's OWN intermediate representation — deliberately richer than
//! any engine's table type (durations, per-phoneme aspiration, stop timing).
//! The engine adapter crate lowers it; core never depends on an engine.
//! Inventory per docs/phonology.md §1 (CLL chapters 3–4); acoustic seeds per
//! docs/formants.md — tests assert against that table.

use crate::alloc::vec::Vec;

/// The six Lojban vowels. `y` = [ə], never stressed, never counted for stress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vowel {
    A,
    E,
    I,
    O,
    U,
    Y,
}

impl Vowel {
    pub const ALL: [Vowel; 6] = [Vowel::A, Vowel::E, Vowel::I, Vowel::O, Vowel::U, Vowel::Y];

    /// This vowel's slot in [`Self::ALL`] / [`VoiceTable::vowels`].
    pub const fn index(self) -> usize {
        match self {
            Vowel::A => 0,
            Vowel::E => 1,
            Vowel::I => 2,
            Vowel::O => 3,
            Vowel::U => 4,
            Vowel::Y => 5,
        }
    }
}

/// The seventeen Lojban consonants (CLL: b c d f g j k l m n p r s t v x z).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Consonant {
    B,
    C,
    D,
    F,
    G,
    J,
    K,
    L,
    M,
    N,
    P,
    R,
    S,
    T,
    V,
    X,
    Z,
}

impl Consonant {
    pub const ALL: [Consonant; 17] = [
        Consonant::B,
        Consonant::C,
        Consonant::D,
        Consonant::F,
        Consonant::G,
        Consonant::J,
        Consonant::K,
        Consonant::L,
        Consonant::M,
        Consonant::N,
        Consonant::P,
        Consonant::R,
        Consonant::S,
        Consonant::T,
        Consonant::V,
        Consonant::X,
        Consonant::Z,
    ];
}

/// One phoneme-level unit. Diphthongs are single syllable nuclei (16 valid,
/// NO triphthongs); `H` is the apostrophe [h], intervocalic aspiration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phoneme {
    Vowel(Vowel),
    Diphthong(Vowel, Vowel),
    Consonant(Consonant),
    /// ' (apostrophe) = [h]: aspiration noise shaped by the FOLLOWING vowel's
    /// formants (docs/formants.md: no locus of its own).
    H,
}

/// The exact 16 diphthongs (docs/phonology.md §1): falling ai ei oi au;
/// rising ia ie ii io iu, ua ue ui uo uu; names-only iy uy.
pub const DIPHTHONGS: [(Vowel, Vowel); 16] = [
    (Vowel::A, Vowel::I),
    (Vowel::E, Vowel::I),
    (Vowel::O, Vowel::I),
    (Vowel::A, Vowel::U),
    (Vowel::I, Vowel::A),
    (Vowel::I, Vowel::E),
    (Vowel::I, Vowel::I),
    (Vowel::I, Vowel::O),
    (Vowel::I, Vowel::U),
    (Vowel::U, Vowel::A),
    (Vowel::U, Vowel::E),
    (Vowel::U, Vowel::I),
    (Vowel::U, Vowel::O),
    (Vowel::U, Vowel::U),
    (Vowel::I, Vowel::Y),
    (Vowel::U, Vowel::Y),
];

pub fn is_valid_diphthong(a: Vowel, b: Vowel) -> bool {
    DIPHTHONGS.contains(&(a, b))
}

impl Phoneme {
    /// Construct a diphthong, `None` if the pair is not one of the 16.
    pub fn diphthong(a: Vowel, b: Vowel) -> Option<Phoneme> {
        is_valid_diphthong(a, b).then_some(Phoneme::Diphthong(a, b))
    }
}

/// One formant resonance target: center frequency, bandwidth, linear amplitude.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Formant {
    pub freq_hz: f32,
    pub bw_hz: f32,
    pub amp: f32,
}

/// Steady-state acoustic targets for one segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Targets {
    pub formants: [Formant; 3],
    /// 0.0 = fully unvoiced .. 1.0 = fully voiced.
    pub voicing: f32,
    /// Aspiration/frication noise drive, 0.0..1.0.
    pub aspiration: f32,
}

impl Targets {
    /// Number of f32 fields — the flat-crossing stride of one Targets.
    pub const FIELDS: usize = 11;

    /// The CANONICAL flat order (== the `data::t()` argument order; the demo's
    /// JS descriptors and the CLI config field names mirror it):
    /// `[f1,b1,a1, f2,b2,a2, f3,b3,a3, voicing, aspiration]`. Do not reorder.
    pub const fn to_array(self) -> [f32; Self::FIELDS] {
        [
            self.formants[0].freq_hz,
            self.formants[0].bw_hz,
            self.formants[0].amp,
            self.formants[1].freq_hz,
            self.formants[1].bw_hz,
            self.formants[1].amp,
            self.formants[2].freq_hz,
            self.formants[2].bw_hz,
            self.formants[2].amp,
            self.voicing,
            self.aspiration,
        ]
    }

    /// Inverse of [`Self::to_array`].
    pub const fn from_array(a: [f32; Self::FIELDS]) -> Self {
        Self {
            formants: [
                Formant {
                    freq_hz: a[0],
                    bw_hz: a[1],
                    amp: a[2],
                },
                Formant {
                    freq_hz: a[3],
                    bw_hz: a[4],
                    amp: a[5],
                },
                Formant {
                    freq_hz: a[6],
                    bw_hz: a[7],
                    amp: a[8],
                },
            ],
            voicing: a[9],
            aspiration: a[10],
        }
    }
}

/// How a segment evolves over its duration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegmentKind {
    /// Vowels, fricatives, nasals, liquids: hold targets.
    Steady(Targets),
    /// Diphthongs: 25% onset at `from`, 50% linear glide, 25% offset at `to`.
    Glide { from: Targets, to: Targets },
    /// Stops: closure (silence or voice bar) then release burst.
    Stop {
        closure: Targets,
        burst: Targets,
        closure_ms: f32,
        burst_ms: f32,
    },
    /// [h]: noise through the FOLLOWING vowel's formants — the adapter
    /// resolves the shape from the next segment at lowering time.
    Aspirate,
}

/// A phoneme's full acoustic specification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentSpec {
    pub kind: SegmentKind,
    pub dur_ms: f32,
}

impl SegmentSpec {
    /// The targets a segment presents at its onset (used by a preceding [h]
    /// and by transition planning). `None` only for [`SegmentKind::Aspirate`].
    pub fn leading_targets(&self) -> Option<Targets> {
        match self.kind {
            SegmentKind::Steady(t) => Some(t),
            SegmentKind::Glide { from, .. } => Some(from),
            SegmentKind::Stop { closure, .. } => Some(closure),
            SegmentKind::Aspirate => None,
        }
    }
}

/// Full acoustic spec for a phoneme, from the PINNED docs/formants.md seeds.
/// Equivalent to `spec_with(p, &VoiceTable::PINNED)`.
pub fn spec(p: Phoneme) -> SegmentSpec {
    spec_with(p, &VoiceTable::PINNED)
}

/// Convenience: specs for a phoneme sequence.
pub fn specs(phonemes: &[Phoneme]) -> Vec<SegmentSpec> {
    phonemes.iter().map(|p| spec(*p)).collect()
}

/// The epenthetic buffer vowel (--buffer flag): [ɪ]-like, acoustically
/// distinct from all six Lojban vowels, as short and weak as possible
/// (CLL §3.8; seeds from docs/formants.md — see `data::BUFFER`). Never
/// written, never stressed, never counted. Equivalent to
/// `buffer_spec_with(&VoiceTable::PINNED)`.
pub fn buffer_spec() -> SegmentSpec {
    buffer_spec_with(&VoiceTable::PINNED)
}

/// One steady phoneme's runtime voice: targets + duration. Stride 12.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SteadyVoice {
    pub targets: Targets,
    pub dur_ms: f32,
}

/// One stop's runtime voice: closure + burst targets and their timing.
/// Stride 24.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StopVoice {
    pub closure: Targets,
    pub burst: Targets,
    pub closure_ms: f32,
    pub burst_ms: f32,
}

/// CANONICAL consonant orderings for [`VoiceTable`] sections and the flat-f32
/// crossing. Do not reorder: the demo's JS descriptors and the CLI config
/// mirror these.
pub const STOP_ORDER: [Consonant; 6] = [
    Consonant::P,
    Consonant::T,
    Consonant::K,
    Consonant::B,
    Consonant::D,
    Consonant::G,
];
pub const FRICATIVE_ORDER: [Consonant; 7] = [
    Consonant::F,
    Consonant::V,
    Consonant::S,
    Consonant::Z,
    Consonant::C,
    Consonant::J,
    Consonant::X,
];
pub const NASAL_ORDER: [Consonant; 2] = [Consonant::M, Consonant::N];
pub const LIQUID_ORDER: [Consonant; 2] = [Consonant::L, Consonant::R];

/// A diphthong's slot in [`DIPHTHONGS`] / [`VoiceTable::diphthong_dur_ms`].
pub fn diphthong_index(a: Vowel, b: Vowel) -> Option<usize> {
    DIPHTHONGS.iter().position(|&d| d == (a, b))
}

/// The RUNTIME per-phoneme acoustic table (demo tuning console D2b): every
/// independent parameter of the segmental layer. Diphthong ENDPOINTS derive
/// from the (tuned) vowel targets and [h] takes its shape from the following
/// vowel (docs/formants.md) — only their durations are free here. `Default`
/// equals the pinned docs/formants.md seeds, so
/// `spec_with(p, &VoiceTable::default())` is byte-identical to `spec(p)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoiceTable {
    /// [`Vowel::ALL`] order: a e i o u y.
    pub vowels: [SteadyVoice; 6],
    /// [`DIPHTHONGS`] order.
    pub diphthong_dur_ms: [f32; 16],
    /// [`STOP_ORDER`]: p t k b d g.
    pub stops: [StopVoice; 6],
    /// [`FRICATIVE_ORDER`]: f v s z c j x.
    pub fricatives: [SteadyVoice; 7],
    /// [`NASAL_ORDER`]: m n.
    pub nasals: [SteadyVoice; 2],
    /// [`LIQUID_ORDER`]: l r.
    pub liquids: [SteadyVoice; 2],
    /// [h] duration (shape always follows the next vowel).
    pub h_dur_ms: f32,
    /// The epenthetic buffer vowel (--buffer flag).
    pub buffer: SteadyVoice,
}

/// Decompose a steady consonant's PINNED `data::` spec (const — feeds
/// [`VoiceTable::PINNED`]; must not go through `spec()`, which reads PINNED).
const fn pinned_steady(c: Consonant) -> SteadyVoice {
    let s = data::consonant_spec(c);
    match s.kind {
        SegmentKind::Steady(targets) => SteadyVoice {
            targets,
            dur_ms: s.dur_ms,
        },
        _ => panic!("not a steady consonant"),
    }
}

/// Decompose a stop consonant's PINNED `data::` spec (const).
const fn pinned_stop(c: Consonant) -> StopVoice {
    match data::consonant_spec(c).kind {
        SegmentKind::Stop {
            closure,
            burst,
            closure_ms,
            burst_ms,
        } => StopVoice {
            closure,
            burst,
            closure_ms,
            burst_ms,
        },
        _ => panic!("not a stop"),
    }
}

const fn pinned_vowel(v: Vowel) -> SteadyVoice {
    SteadyVoice {
        targets: data::vowel_targets(v),
        dur_ms: data::vowel_duration_ms(v),
    }
}

impl VoiceTable {
    /// The pinned docs/formants.md seeds, built FROM the `data::` helpers —
    /// there is exactly one copy of the numbers, so `Default`/`PINNED`
    /// byte-identity with `spec()` holds by construction.
    pub const PINNED: Self = Self {
        vowels: [
            pinned_vowel(Vowel::A),
            pinned_vowel(Vowel::E),
            pinned_vowel(Vowel::I),
            pinned_vowel(Vowel::O),
            pinned_vowel(Vowel::U),
            pinned_vowel(Vowel::Y),
        ],
        diphthong_dur_ms: [data::DIPHTHONG_MS; 16],
        stops: [
            pinned_stop(Consonant::P),
            pinned_stop(Consonant::T),
            pinned_stop(Consonant::K),
            pinned_stop(Consonant::B),
            pinned_stop(Consonant::D),
            pinned_stop(Consonant::G),
        ],
        fricatives: [
            pinned_steady(Consonant::F),
            pinned_steady(Consonant::V),
            pinned_steady(Consonant::S),
            pinned_steady(Consonant::Z),
            pinned_steady(Consonant::C),
            pinned_steady(Consonant::J),
            pinned_steady(Consonant::X),
        ],
        nasals: [pinned_steady(Consonant::M), pinned_steady(Consonant::N)],
        liquids: [pinned_steady(Consonant::L), pinned_steady(Consonant::R)],
        h_dur_ms: data::H_MS,
        buffer: data::BUFFER,
    };
}

impl Default for VoiceTable {
    fn default() -> Self {
        Self::PINNED
    }
}

impl SteadyVoice {
    pub const FIELDS: usize = Targets::FIELDS + 1;

    /// `[targets(11), dur_ms]`.
    fn write(&self, dst: &mut [f32], at: &mut usize) {
        dst[*at..*at + Targets::FIELDS].copy_from_slice(&self.targets.to_array());
        dst[*at + Targets::FIELDS] = self.dur_ms;
        *at += Self::FIELDS;
    }

    fn read(src: &[f32], at: &mut usize) -> Self {
        let mut t = [0.0f32; Targets::FIELDS];
        t.copy_from_slice(&src[*at..*at + Targets::FIELDS]);
        let out = Self {
            targets: Targets::from_array(t),
            dur_ms: src[*at + Targets::FIELDS],
        };
        *at += Self::FIELDS;
        out
    }
}

impl StopVoice {
    pub const FIELDS: usize = 2 * Targets::FIELDS + 2;

    /// `[closure(11), burst(11), closure_ms, burst_ms]`.
    fn write(&self, dst: &mut [f32], at: &mut usize) {
        dst[*at..*at + Targets::FIELDS].copy_from_slice(&self.closure.to_array());
        dst[*at + Targets::FIELDS..*at + 2 * Targets::FIELDS]
            .copy_from_slice(&self.burst.to_array());
        dst[*at + 2 * Targets::FIELDS] = self.closure_ms;
        dst[*at + 2 * Targets::FIELDS + 1] = self.burst_ms;
        *at += Self::FIELDS;
    }

    fn read(src: &[f32], at: &mut usize) -> Self {
        let mut c = [0.0f32; Targets::FIELDS];
        let mut b = [0.0f32; Targets::FIELDS];
        c.copy_from_slice(&src[*at..*at + Targets::FIELDS]);
        b.copy_from_slice(&src[*at + Targets::FIELDS..*at + 2 * Targets::FIELDS]);
        let out = Self {
            closure: Targets::from_array(c),
            burst: Targets::from_array(b),
            closure_ms: src[*at + 2 * Targets::FIELDS],
            burst_ms: src[*at + 2 * Targets::FIELDS + 1],
        };
        *at += Self::FIELDS;
        out
    }
}

impl VoiceTable {
    /// Flat-f32 field count.
    pub const FIELDS: usize = 377;

    /// LAYOUT — the ONE normative flat ordering (JS descriptors + CLI config
    /// mirror it; do not reorder):
    /// vowels @0 (6×12=72) → diphthong durations @72 (16×1) → stops @88
    /// (6×24=144) → fricatives @232 (7×12=84) → nasals @316 (2×12=24) →
    /// liquids @340 (2×12=24) → h duration @364 (1) → buffer @365 (12) = 377.
    pub fn to_array(&self) -> [f32; Self::FIELDS] {
        let mut out = [0.0f32; Self::FIELDS];
        let mut at = 0usize;
        for v in &self.vowels {
            v.write(&mut out, &mut at);
        }
        out[at..at + 16].copy_from_slice(&self.diphthong_dur_ms);
        at += 16;
        for s in &self.stops {
            s.write(&mut out, &mut at);
        }
        for f in &self.fricatives {
            f.write(&mut out, &mut at);
        }
        for n in &self.nasals {
            n.write(&mut out, &mut at);
        }
        for l in &self.liquids {
            l.write(&mut out, &mut at);
        }
        out[at] = self.h_dur_ms;
        at += 1;
        self.buffer.write(&mut out, &mut at);
        debug_assert_eq!(at, Self::FIELDS);
        out
    }

    /// Inverse of [`Self::to_array`].
    pub fn from_array(a: [f32; Self::FIELDS]) -> Self {
        let mut at = 0usize;
        let vowels = core::array::from_fn(|_| SteadyVoice::read(&a, &mut at));
        let mut diphthong_dur_ms = [0.0f32; 16];
        diphthong_dur_ms.copy_from_slice(&a[at..at + 16]);
        at += 16;
        let stops = core::array::from_fn(|_| StopVoice::read(&a, &mut at));
        let fricatives = core::array::from_fn(|_| SteadyVoice::read(&a, &mut at));
        let nasals = core::array::from_fn(|_| SteadyVoice::read(&a, &mut at));
        let liquids = core::array::from_fn(|_| SteadyVoice::read(&a, &mut at));
        let h_dur_ms = a[at];
        at += 1;
        let buffer = SteadyVoice::read(&a, &mut at);
        debug_assert_eq!(at, Self::FIELDS);
        Self {
            vowels,
            diphthong_dur_ms,
            stops,
            fricatives,
            nasals,
            liquids,
            h_dur_ms,
            buffer,
        }
    }
}

/// This consonant's slot in the [`VoiceTable`] section for its manner class.
const fn stop_slot(c: Consonant) -> Option<usize> {
    match c {
        Consonant::P => Some(0),
        Consonant::T => Some(1),
        Consonant::K => Some(2),
        Consonant::B => Some(3),
        Consonant::D => Some(4),
        Consonant::G => Some(5),
        _ => None,
    }
}

const fn fricative_slot(c: Consonant) -> Option<usize> {
    match c {
        Consonant::F => Some(0),
        Consonant::V => Some(1),
        Consonant::S => Some(2),
        Consonant::Z => Some(3),
        Consonant::C => Some(4),
        Consonant::J => Some(5),
        Consonant::X => Some(6),
        _ => None,
    }
}

const fn nasal_slot(c: Consonant) -> Option<usize> {
    match c {
        Consonant::M => Some(0),
        Consonant::N => Some(1),
        _ => None,
    }
}

const fn liquid_slot(c: Consonant) -> Option<usize> {
    match c {
        Consonant::L => Some(0),
        Consonant::R => Some(1),
        _ => None,
    }
}

/// A steady [`SegmentSpec`] from a runtime voice entry. Durations clamp to
/// ≥ 0 (a hand-edited config's negative duration would walk the schedule
/// backwards; NaN would panic the span sort — `.max(0.0)` neutralizes both
/// and is exact identity for every pinned value).
fn steady_from(sv: SteadyVoice) -> SegmentSpec {
    SegmentSpec {
        kind: SegmentKind::Steady(sv.targets),
        dur_ms: sv.dur_ms.max(0.0),
    }
}

/// Like [`spec`] but reading the acoustic numbers from a RUNTIME table (demo
/// tuning console D2b). Diphthong glide ENDPOINTS come from the (tuned) vowel
/// entries; [h] keeps taking its shape from the following vowel at lowering
/// time. `spec_with(p, &VoiceTable::default())` is byte-identical to
/// `spec(p)`.
pub fn spec_with(p: Phoneme, voice: &VoiceTable) -> SegmentSpec {
    match p {
        Phoneme::Vowel(v) => steady_from(voice.vowels[v.index()]),
        Phoneme::Diphthong(a, b) => SegmentSpec {
            kind: SegmentKind::Glide {
                from: voice.vowels[a.index()].targets,
                to: voice.vowels[b.index()].targets,
            },
            dur_ms: diphthong_index(a, b)
                .map_or(data::DIPHTHONG_MS, |i| voice.diphthong_dur_ms[i])
                .max(0.0),
        },
        Phoneme::Consonant(c) => {
            if let Some(i) = stop_slot(c) {
                let s = voice.stops[i];
                let (closure_ms, burst_ms) = (s.closure_ms.max(0.0), s.burst_ms.max(0.0));
                SegmentSpec {
                    kind: SegmentKind::Stop {
                        closure: s.closure,
                        burst: s.burst,
                        closure_ms,
                        burst_ms,
                    },
                    dur_ms: closure_ms + burst_ms,
                }
            } else if let Some(i) = fricative_slot(c) {
                steady_from(voice.fricatives[i])
            } else if let Some(i) = nasal_slot(c) {
                steady_from(voice.nasals[i])
            } else if let Some(i) = liquid_slot(c) {
                steady_from(voice.liquids[i])
            } else {
                unreachable!("every consonant has a manner class")
            }
        }
        Phoneme::H => SegmentSpec {
            kind: SegmentKind::Aspirate,
            dur_ms: voice.h_dur_ms.max(0.0),
        },
    }
}

/// Like [`buffer_spec`] but reading from a RUNTIME table (demo tuning console
/// D2b). `buffer_spec_with(&VoiceTable::default())` == `buffer_spec()`.
pub fn buffer_spec_with(voice: &VoiceTable) -> SegmentSpec {
    steady_from(voice.buffer)
}

mod data {
    //! Acoustic seed data (docs/formants.md — the single source of truth;
    //! if values here are retuned, update that table and the schedule
    //! snapshots together). Amplitudes fall with formant number so band
    //! peak-picking resolves each formant against glottal-source rolloff;
    //! voiced fricatives/closures carry a low murmur resonator as the
    //! voice bar.

    use super::{Consonant, Formant, SegmentKind, SegmentSpec, SteadyVoice, Targets, Vowel};

    pub(super) const DIPHTHONG_MS: f32 = 200.0;
    pub(super) const H_MS: f32 = 70.0;
    const FRICATIVE_MS: f32 = 120.0;
    const SONORANT_MS: f32 = 80.0;
    const STOP_CLOSURE_MS: f32 = 60.0;
    const STOP_BURST_MS: f32 = 25.0;

    /// The epenthetic buffer vowel seed ([ɪ]-like; docs/formants.md).
    pub(super) const BUFFER: SteadyVoice = SteadyVoice {
        targets: t(
            400.0, 90.0, 0.5, 1900.0, 110.0, 0.3, 2600.0, 150.0, 0.15, 1.0, 0.0,
        ),
        dur_ms: 35.0,
    };

    #[allow(clippy::too_many_arguments)]
    const fn t(
        f1: f32,
        b1: f32,
        a1: f32,
        f2: f32,
        b2: f32,
        a2: f32,
        f3: f32,
        b3: f32,
        a3: f32,
        voicing: f32,
        aspiration: f32,
    ) -> Targets {
        Targets {
            formants: [
                Formant {
                    freq_hz: f1,
                    bw_hz: b1,
                    amp: a1,
                },
                Formant {
                    freq_hz: f2,
                    bw_hz: b2,
                    amp: a2,
                },
                Formant {
                    freq_hz: f3,
                    bw_hz: b3,
                    amp: a3,
                },
            ],
            voicing,
            aspiration,
        }
    }

    /// Total silence (voiceless stop closures).
    const fn silence() -> Targets {
        t(
            500.0, 90.0, 0.0, 1500.0, 110.0, 0.0, 2500.0, 150.0, 0.0, 0.0, 0.0,
        )
    }

    pub(super) const fn vowel_targets(v: Vowel) -> Targets {
        match v {
            Vowel::A => t(
                730.0, 90.0, 1.0, 1090.0, 110.0, 0.8, 2440.0, 150.0, 0.3, 1.0, 0.0,
            ),
            Vowel::E => t(
                530.0, 90.0, 1.0, 1840.0, 110.0, 0.6, 2480.0, 150.0, 0.3, 1.0, 0.0,
            ),
            Vowel::I => t(
                270.0, 90.0, 1.0, 2290.0, 110.0, 0.7, 3010.0, 150.0, 0.45, 1.0, 0.0,
            ),
            Vowel::O => t(
                570.0, 90.0, 1.0, 840.0, 110.0, 0.8, 2410.0, 150.0, 0.25, 1.0, 0.0,
            ),
            Vowel::U => t(
                300.0, 90.0, 1.0, 870.0, 110.0, 0.75, 2240.0, 150.0, 0.25, 1.0, 0.0,
            ),
            Vowel::Y => t(
                500.0, 90.0, 1.0, 1500.0, 110.0, 0.6, 2500.0, 150.0, 0.3, 1.0, 0.0,
            ),
        }
    }

    pub(super) const fn vowel_duration_ms(v: Vowel) -> f32 {
        match v {
            Vowel::A => 160.0,
            Vowel::Y => 100.0,
            _ => 150.0,
        }
    }

    pub(super) const fn consonant_spec(c: Consonant) -> SegmentSpec {
        match c {
            // Stops: closure (silence / voice bar) + release burst.
            Consonant::P
            | Consonant::T
            | Consonant::K
            | Consonant::B
            | Consonant::D
            | Consonant::G => {
                let voiced = matches!(c, Consonant::B | Consonant::D | Consonant::G);
                let closure = if voiced {
                    // Voice bar: low-amplitude murmur during voiced closure.
                    t(
                        200.0, 100.0, 0.3, 1500.0, 110.0, 0.0, 2500.0, 150.0, 0.0, 1.0, 0.0,
                    )
                } else {
                    silence()
                };
                let burst = match c {
                    // Bilabial: diffuse low-frequency burst.
                    Consonant::P | Consonant::B => t(
                        500.0, 90.0, 0.0, 700.0, 700.0, 0.6, 2500.0, 150.0, 0.0, 0.0, 1.0,
                    ),
                    // Alveolar: 3000-4000 Hz emphasis.
                    Consonant::T | Consonant::D => t(
                        500.0, 90.0, 0.0, 1500.0, 110.0, 0.0, 3500.0, 1000.0, 0.85, 0.0, 1.0,
                    ),
                    // Velar: compact mid burst near the velar locus.
                    _ => t(
                        500.0, 90.0, 0.0, 1500.0, 110.0, 0.0, 2150.0, 500.0, 0.85, 0.0, 1.0,
                    ),
                };
                SegmentSpec {
                    kind: SegmentKind::Stop {
                        closure,
                        burst,
                        closure_ms: STOP_CLOSURE_MS,
                        burst_ms: STOP_BURST_MS,
                    },
                    dur_ms: STOP_CLOSURE_MS + STOP_BURST_MS,
                }
            }
            // Fricatives: noise-band resonator; voiced ones add a murmur bar.
            Consonant::S => steady(
                t(
                    500.0, 90.0, 0.0, 1500.0, 110.0, 0.0, 6000.0, 1700.0, 0.9, 0.0, 1.0,
                ),
                FRICATIVE_MS,
            ),
            Consonant::Z => steady(
                t(
                    250.0, 100.0, 0.6, 1500.0, 110.0, 0.0, 6000.0, 1700.0, 0.8, 0.6, 0.6,
                ),
                FRICATIVE_MS,
            ),
            Consonant::C => steady(
                t(
                    500.0, 90.0, 0.0, 1500.0, 110.0, 0.0, 3000.0, 800.0, 0.9, 0.0, 1.0,
                ),
                FRICATIVE_MS,
            ),
            Consonant::J => steady(
                t(
                    250.0, 100.0, 0.6, 1500.0, 110.0, 0.0, 2800.0, 700.0, 0.8, 0.6, 0.6,
                ),
                FRICATIVE_MS,
            ),
            Consonant::X => steady(
                t(
                    500.0, 90.0, 0.0, 1500.0, 110.0, 0.0, 2000.0, 600.0, 0.85, 0.0, 1.0,
                ),
                FRICATIVE_MS,
            ),
            Consonant::F => steady(
                t(
                    500.0, 90.0, 0.0, 1500.0, 110.0, 0.0, 4500.0, 3500.0, 0.3, 0.0, 0.8,
                ),
                FRICATIVE_MS,
            ),
            Consonant::V => steady(
                t(
                    250.0, 100.0, 0.6, 1500.0, 110.0, 0.0, 4500.0, 3500.0, 0.3, 0.6, 0.5,
                ),
                FRICATIVE_MS,
            ),
            // Nasals: low murmur, attenuated mid/high (anti-resonance approx).
            Consonant::M => steady(
                t(
                    300.0, 100.0, 1.0, 1000.0, 150.0, 0.1, 2200.0, 200.0, 0.05, 1.0, 0.0,
                ),
                SONORANT_MS,
            ),
            Consonant::N => steady(
                t(
                    310.0, 100.0, 1.0, 1700.0, 150.0, 0.1, 2600.0, 200.0, 0.05, 1.0, 0.0,
                ),
                SONORANT_MS,
            ),
            // Liquids: l lateral; r rhotic = lowered F3.
            Consonant::L => steady(
                t(
                    360.0, 90.0, 1.0, 1300.0, 110.0, 0.5, 2700.0, 150.0, 0.2, 1.0, 0.0,
                ),
                SONORANT_MS,
            ),
            Consonant::R => steady(
                t(
                    490.0, 90.0, 1.0, 1350.0, 110.0, 0.4, 1750.0, 130.0, 0.45, 1.0, 0.0,
                ),
                SONORANT_MS,
            ),
        }
    }

    const fn steady(targets: Targets, dur_ms: f32) -> SegmentSpec {
        SegmentSpec {
            kind: SegmentKind::Steady(targets),
            dur_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_is_complete() {
        assert_eq!(Consonant::ALL.len(), 17);
        assert_eq!(Vowel::ALL.len(), 6);
        assert_eq!(DIPHTHONGS.len(), 16);
    }

    #[test]
    fn diphthong_validation_matches_cll() {
        assert!(is_valid_diphthong(Vowel::A, Vowel::I));
        assert!(is_valid_diphthong(Vowel::I, Vowel::Y)); // names-only, still valid
        assert!(is_valid_diphthong(Vowel::U, Vowel::U));
        assert!(!is_valid_diphthong(Vowel::A, Vowel::E)); // not a diphthong
        assert!(!is_valid_diphthong(Vowel::Y, Vowel::I)); // y never starts one
        assert!(Phoneme::diphthong(Vowel::A, Vowel::E).is_none());
        assert!(Phoneme::diphthong(Vowel::A, Vowel::U).is_some());
    }

    #[test]
    fn every_phoneme_has_positive_duration() {
        for v in Vowel::ALL {
            assert!(spec(Phoneme::Vowel(v)).dur_ms > 0.0);
        }
        for c in Consonant::ALL {
            assert!(spec(Phoneme::Consonant(c)).dur_ms > 0.0);
        }
        for (a, b) in DIPHTHONGS {
            assert!(spec(Phoneme::Diphthong(a, b)).dur_ms > 0.0);
        }
        assert!(spec(Phoneme::H).dur_ms > 0.0);
    }

    #[test]
    fn sibilant_voicing_matches_phonology() {
        // s c x unvoiced; z j voiced counterparts (docs/phonology.md §1).
        let voicing = |c: Consonant| {
            spec(Phoneme::Consonant(c))
                .leading_targets()
                .expect("fricatives are steady")
                .voicing
        };
        assert_eq!(voicing(Consonant::S), 0.0);
        assert_eq!(voicing(Consonant::C), 0.0);
        assert_eq!(voicing(Consonant::X), 0.0);
        assert!(voicing(Consonant::Z) > 0.0, "z is voiced");
        assert!(voicing(Consonant::J) > 0.0, "j is voiced");
    }

    #[test]
    fn vowels_are_voiced_and_audible() {
        for v in Vowel::ALL {
            let t = spec(Phoneme::Vowel(v))
                .leading_targets()
                .expect("vowels are steady");
            assert!(t.voicing > 0.9, "{v:?} must be fully voiced");
            assert!(
                t.formants[0].amp > 0.0,
                "{v:?} F1 must have nonzero amplitude"
            );
        }
    }
}
