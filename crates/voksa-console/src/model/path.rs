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
/// 4 sonorants + [h] + buffer.
pub const VOICE_ITEM_COUNT: usize = 41;

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
        let _ = self;
        0 // stub — C1 green
    }

    /// The path at a flat index (panics if `idx >= 449`).
    pub fn from_flat(idx: usize) -> Path {
        let _ = idx;
        Path::Knob(0) // stub — C1 green
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // stub — C1 green (uses KNOBS/ATT_KINDS/VOICE_ITEMS key tables)
        let _ = (&KNOBS, &ATT_KINDS, &VOICE_ITEMS);
        write!(f, "?")
    }
}

impl FromStr for Path {
    type Err = PathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Err(PathError(format!("stub — C1 green: {s}")))
    }
}
