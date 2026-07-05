//! The console's parameter address space: every tunable value has a [`Path`]
//! (`k.<knob>` / `a.<cmavo>.<i>` / `v.<phoneme>.<i>`) and a total, static
//! mapping onto the frozen 449-float wasm layout.

use core::fmt;
use core::str::FromStr;

use super::descriptor::{ATT_KINDS, KNOBS, VOICE_ITEMS};

/// 7 prosody + 9 naturalness runtime knobs.
pub const KNOB_COUNT: usize = 16;
/// The 7 attitudinal (UI-cmavo) deviation vectors.
pub const ATT_KIND_COUNT: usize = 7;
/// Deviation fields per attitudinal.
pub const ATT_FIELD_COUNT: usize = 8;
/// Voice-table items: 6 vowels + 16 diphthongs + 6 stops + 7 fricatives +
/// 4 sonorants + `[h]` + buffer.
pub const VOICE_ITEM_COUNT: usize = 41;

/// Flat offset of the attitudinal section.
const ATT_OFF: usize = 7;
/// Flat offset of the voice-table section.
const VOICE_OFF: usize = 63;
/// Flat offset of the naturalness knobs.
const NAT_OFF: usize = 440;
/// Prosody knobs (flat 0..7); the remaining knobs live at [`NAT_OFF`].
const PROSODY_KNOBS: usize = 7;

/// Prefix sums of the voice items' spans: `VOICE_OFFSETS[i]` is item `i`'s
/// offset within the 377-slot voice section; the final entry is 377.
pub(super) const VOICE_OFFSETS: [usize; VOICE_ITEM_COUNT + 1] = {
    let mut out = [0usize; VOICE_ITEM_COUNT + 1];
    let mut i = 0;
    while i < VOICE_ITEM_COUNT {
        out[i + 1] = out[i] + VOICE_ITEMS[i].kind.span();
        i += 1;
    }
    out
};

/// A tunable parameter's address. `Copy` and enumerable: `flat_index` /
/// `from_flat` are a total bijection with `0..449`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Path {
    /// Runtime knob `k.<key>`: 0..7 = prosody, 7..16 = naturalness.
    Knob(u8),
    /// Attitudinal field `a.<cmavo>.<i>` (kind 0..7 in `AttitudinalKind::ALL`
    /// order, field 0..8 in `Deviation::to_array` order).
    Att {
        /// Attitudinal kind index (0..7).
        kind: u8,
        /// Deviation field index (0..8).
        field: u8,
    },
    /// Voice-table slot `v.<phoneme>.<i>` (item 0..41 in `VoiceTable::to_array`
    /// order; slot < the item's span: steady 12, dur 1, stop 24).
    Voice {
        /// Voice item index (0..41).
        item: u8,
        /// Slot within the item.
        slot: u8,
    },
}

/// Error parsing a `Path` from its string form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathError(pub String);

impl Path {
    /// The path's index in the frozen 449-float layout.
    pub fn flat_index(self) -> usize {
        match self {
            Path::Knob(k) => {
                let k = k as usize;
                if k < PROSODY_KNOBS {
                    k
                } else {
                    NAT_OFF + (k - PROSODY_KNOBS)
                }
            }
            Path::Att { kind, field } => ATT_OFF + kind as usize * ATT_FIELD_COUNT + field as usize,
            Path::Voice { item, slot } => VOICE_OFF + VOICE_OFFSETS[item as usize] + slot as usize,
        }
    }

    /// The path at a flat index (panics if `idx >= 449`).
    pub fn from_flat(idx: usize) -> Path {
        match idx {
            0..PROSODY_KNOBS => Path::Knob(idx as u8),
            ATT_OFF..VOICE_OFF => {
                let rel = idx - ATT_OFF;
                Path::Att {
                    kind: (rel / ATT_FIELD_COUNT) as u8,
                    field: (rel % ATT_FIELD_COUNT) as u8,
                }
            }
            VOICE_OFF..NAT_OFF => {
                let rel = idx - VOICE_OFF;
                let item = VOICE_OFFSETS.partition_point(|&off| off <= rel) - 1;
                Path::Voice {
                    item: item as u8,
                    slot: (rel - VOICE_OFFSETS[item]) as u8,
                }
            }
            NAT_OFF..449 => Path::Knob((PROSODY_KNOBS + (idx - NAT_OFF)) as u8),
            _ => panic!("flat index {idx} outside the frozen 449-float layout"),
        }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Path::Knob(k) => write!(f, "k.{}", KNOBS[k as usize].key),
            Path::Att { kind, field } => write!(f, "a.{}.{}", ATT_KINDS[kind as usize].key, field),
            Path::Voice { item, slot } => {
                write!(f, "v.{}.{}", VOICE_ITEMS[item as usize].key, slot)
            }
        }
    }
}

impl FromStr for Path {
    type Err = PathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || PathError(format!("not a parameter path: {s}"));
        let (ns, rest) = s.split_once('.').ok_or_else(err)?;
        match ns {
            "k" => {
                let k = KNOBS.iter().position(|f| f.key == rest).ok_or_else(err)?;
                Ok(Path::Knob(k as u8))
            }
            "a" => {
                // The cmavo key may itself contain an apostrophe but never a
                // dot, so the LAST dot separates the field index.
                let (key, field) = rest.rsplit_once('.').ok_or_else(err)?;
                let kind = ATT_KINDS
                    .iter()
                    .position(|a| a.key == key)
                    .ok_or_else(err)?;
                let field: usize = field.parse().map_err(|_| err())?;
                if field >= ATT_FIELD_COUNT {
                    return Err(err());
                }
                Ok(Path::Att {
                    kind: kind as u8,
                    field: field as u8,
                })
            }
            "v" => {
                let (key, slot) = rest.rsplit_once('.').ok_or_else(err)?;
                let item = VOICE_ITEMS
                    .iter()
                    .position(|i| i.key == key)
                    .ok_or_else(err)?;
                let slot: usize = slot.parse().map_err(|_| err())?;
                if slot >= VOICE_ITEMS[item].kind.span() {
                    return Err(err());
                }
                Ok(Path::Voice {
                    item: item as u8,
                    slot: slot as u8,
                })
            }
            _ => Err(err()),
        }
    }
}
