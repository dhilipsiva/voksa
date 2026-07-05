//! The console's pure state model — zero dioxus/web imports, natively tested.
//!
//! Address space: the frozen 449-float wasm layout (prosody 0..7,
//! attitudinals 7..63, voice table 63..440, naturalness 440..449), addressed
//! by [`Path`] (`k.<knob>` / `a.<cmavo>.<i>` / `v.<phoneme>.<i>`). Defaults
//! are seeded from `voksa_web::default_params()` at runtime — never
//! hand-copied. The UI store is a thin projection applying [`WritePlan`]s.

mod config;
mod descriptor;
mod logic;
mod path;
mod presets;

pub use config::{ExportInputs, Flags, LoadPlan, export, load};
pub use descriptor::{
    ATT_FIELDS, ATT_KINDS, AttKind, Descriptor, Descriptors, FieldSpec, ItemKind, KNOBS, SECTIONS,
    Section, VOICE_ITEMS, VT_BURST_MS, VT_CLOSURE_MS, VT_DUR, VT_FIELDS, VoiceItem,
};
pub use logic::{Widen, WritePlan, is_dirty, widen_for};
pub use path::{ATT_FIELD_COUNT, ATT_KIND_COUNT, KNOB_COUNT, Path, PathError, VOICE_ITEM_COUNT};
pub use presets::{PRESETS, Preset, apply_preset};

/// The full parameter count — re-exported from the engine crate so the model
/// can never drift from the frozen layout.
pub const PARAM_TOTAL: usize = voksa_web::FULL_PARAM_COUNT;
