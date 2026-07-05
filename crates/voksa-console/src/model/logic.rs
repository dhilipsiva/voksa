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
    if v >= d.min && v <= d.max {
        None
    } else {
        Some(Widen {
            min: v.min(d.min),
            max: v.max(d.max),
        })
    }
}

/// Modified = the value differs from the engine default in f32 space (Rust
/// f32 comparison IS the reference page's `Math.fround` semantics).
pub fn is_dirty(d: &Descriptor, v: f32) -> bool {
    v != d.default
}

/// A batch of `(flat index, value)` writes the UI store applies atomically
/// (skip-if-equal per cell).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WritePlan(pub Vec<(usize, f32)>);

/// Drop plan entries whose target already holds the value — the store's
/// skip-if-equal pass, so applying a plan invalidates only cells that change.
pub fn filter_plan(current: &[f32], plan: &WritePlan) -> WritePlan {
    WritePlan(
        plan.0
            .iter()
            .copied()
            .filter(|&(idx, v)| current.get(idx).copied() != Some(v))
            .collect(),
    )
}

/// A reset plan: every index in `range` back to its engine default (single
/// rows, emotions, phonemes, the whole table — callers pick the range).
pub fn reset_plan(
    desc: &super::descriptor::Descriptors,
    range: core::ops::Range<usize>,
) -> WritePlan {
    WritePlan(range.map(|i| (i, desc.get(i).default)).collect())
}

/// The effective render params for the naturalness A/B latch: identity when
/// `ab_off` is false; when true, a COPY with the nine naturalness knob slots
/// overridden to their identity ("Naturalness off") values. Stored slider
/// values are never touched — this is a render-time override only.
pub fn ab_effective(
    desc: &super::descriptor::Descriptors,
    params: &[f32],
    ab_off: bool,
) -> Vec<f32> {
    use super::descriptor::KNOBS;
    let mut out = params.to_vec();
    if ab_off {
        let off = super::presets::PRESETS
            .iter()
            .find(|p| p.name == "Naturalness off")
            .expect("the 'Naturalness off' preset defines the identity values");
        for &(key, v) in off.over {
            let knob = KNOBS
                .iter()
                .position(|f| f.key == key)
                .expect("preset knob key exists");
            out[desc.knob_index(knob)] = v;
        }
    }
    out
}
