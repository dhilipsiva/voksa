//! Lojban phoneme inventory and acoustic parameter IR.
//!
//! This is voksa's OWN intermediate representation — deliberately richer than
//! any engine's table type (durations, per-phoneme aspiration, stop timing).
//! The engine adapter (voksa-engine-klattsch) lowers it; core never depends
//! on an engine. Inventory per docs/phonology.md §1 (CLL chapters 3–4);
//! acoustic seeds per docs/formants.md — tests assert against that table.

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

/// Full acoustic spec for a phoneme. Seeds from docs/formants.md.
pub fn spec(p: Phoneme) -> SegmentSpec {
    match p {
        Phoneme::Vowel(v) => SegmentSpec {
            kind: SegmentKind::Steady(data::vowel_targets(v)),
            dur_ms: data::vowel_duration_ms(v),
        },
        Phoneme::Diphthong(a, b) => SegmentSpec {
            kind: SegmentKind::Glide {
                from: data::vowel_targets(a),
                to: data::vowel_targets(b),
            },
            dur_ms: data::DIPHTHONG_MS,
        },
        Phoneme::Consonant(c) => data::consonant_spec(c),
        Phoneme::H => SegmentSpec {
            kind: SegmentKind::Aspirate,
            dur_ms: data::H_MS,
        },
    }
}

/// Convenience: specs for a phoneme sequence.
pub fn specs(phonemes: &[Phoneme]) -> Vec<SegmentSpec> {
    phonemes.iter().map(|p| spec(*p)).collect()
}

mod data {
    //! Acoustic seed data (docs/formants.md).
    //!
    //! RED-CHECKPOINT SKELETON: durations and timing are real, but every
    //! amplitude/voicing/aspiration is zeroed — the table renders silence.
    //! Real values land after the failing acceptance tests are committed.

    use super::{Consonant, Formant, SegmentKind, SegmentSpec, Targets, Vowel};

    pub(super) const DIPHTHONG_MS: f32 = 200.0;
    pub(super) const H_MS: f32 = 70.0;
    const FRICATIVE_MS: f32 = 120.0;
    const SONORANT_MS: f32 = 80.0;
    const STOP_CLOSURE_MS: f32 = 60.0;
    const STOP_BURST_MS: f32 = 25.0;

    const SILENT: Targets = Targets {
        formants: [
            Formant {
                freq_hz: 500.0,
                bw_hz: 90.0,
                amp: 0.0,
            },
            Formant {
                freq_hz: 1500.0,
                bw_hz: 110.0,
                amp: 0.0,
            },
            Formant {
                freq_hz: 2500.0,
                bw_hz: 150.0,
                amp: 0.0,
            },
        ],
        voicing: 0.0,
        aspiration: 0.0,
    };

    pub(super) fn vowel_targets(_v: Vowel) -> Targets {
        SILENT
    }

    pub(super) fn vowel_duration_ms(v: Vowel) -> f32 {
        match v {
            Vowel::A => 160.0,
            Vowel::Y => 100.0,
            _ => 150.0,
        }
    }

    pub(super) fn consonant_spec(c: Consonant) -> SegmentSpec {
        match c {
            Consonant::B
            | Consonant::D
            | Consonant::G
            | Consonant::P
            | Consonant::T
            | Consonant::K => SegmentSpec {
                kind: SegmentKind::Stop {
                    closure: SILENT,
                    burst: SILENT,
                    closure_ms: STOP_CLOSURE_MS,
                    burst_ms: STOP_BURST_MS,
                },
                dur_ms: STOP_CLOSURE_MS + STOP_BURST_MS,
            },
            Consonant::C
            | Consonant::F
            | Consonant::J
            | Consonant::S
            | Consonant::V
            | Consonant::X
            | Consonant::Z => SegmentSpec {
                kind: SegmentKind::Steady(SILENT),
                dur_ms: FRICATIVE_MS,
            },
            Consonant::L | Consonant::M | Consonant::N | Consonant::R => SegmentSpec {
                kind: SegmentKind::Steady(SILENT),
                dur_ms: SONORANT_MS,
            },
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
