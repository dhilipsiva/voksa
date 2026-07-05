//! The behavioral contracts: widen-never-clamp and f32-space dirty diffing.

use super::descriptor::Descriptor;

/// A widened slider range. Exists ONLY while the current value sits outside
/// the descriptor range; while widened, the step relaxes to continuous.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Widen {
    /// Widened minimum (≤ descriptor min).
    pub min: f32,
    /// Widened maximum (≥ descriptor max).
    pub max: f32,
}

/// The widen state after setting `v`: `Some` (bounds unioned with the
/// descriptor range) while `v` is outside `[d.min, d.max]`, `None` once back
/// inside. Values are NEVER clamped.
pub fn widen_for(d: &Descriptor, v: f32) -> Option<Widen> {
    let _ = (d, v);
    None // stub — C1 green
}

/// Modified = the value differs from the engine default in f32 space (Rust
/// f32 comparison IS the reference page's `Math.fround` semantics).
pub fn is_dirty(d: &Descriptor, v: f32) -> bool {
    let _ = (d, v);
    false // stub — C1 green
}

/// A batch of `(flat index, value)` writes the UI store applies atomically
/// (skip-if-equal per cell).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WritePlan(pub Vec<(usize, f32)>);
