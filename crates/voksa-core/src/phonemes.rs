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
    //! Acoustic seed data (docs/formants.md — the single source of truth;
    //! if values here are retuned, update that table and the schedule
    //! snapshots together). Amplitudes fall with formant number so band
    //! peak-picking resolves each formant against glottal-source rolloff;
    //! voiced fricatives/closures carry a low murmur resonator as the
    //! voice bar.

    use super::{Consonant, Formant, SegmentKind, SegmentSpec, Targets, Vowel};

    pub(super) const DIPHTHONG_MS: f32 = 200.0;
    pub(super) const H_MS: f32 = 70.0;
    const FRICATIVE_MS: f32 = 120.0;
    const SONORANT_MS: f32 = 80.0;
    const STOP_CLOSURE_MS: f32 = 60.0;
    const STOP_BURST_MS: f32 = 25.0;

    #[allow(clippy::too_many_arguments)]
    fn t(
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
    fn silence() -> Targets {
        t(
            500.0, 90.0, 0.0, 1500.0, 110.0, 0.0, 2500.0, 150.0, 0.0, 0.0, 0.0,
        )
    }

    pub(super) fn vowel_targets(v: Vowel) -> Targets {
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

    pub(super) fn vowel_duration_ms(v: Vowel) -> f32 {
        match v {
            Vowel::A => 160.0,
            Vowel::Y => 100.0,
            _ => 150.0,
        }
    }

    pub(super) fn consonant_spec(c: Consonant) -> SegmentSpec {
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

    fn steady(targets: Targets, dur_ms: f32) -> SegmentSpec {
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
