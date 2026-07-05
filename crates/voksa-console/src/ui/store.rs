//! The reactive projection of the model: one `Signal<Cell>` per parameter
//! (dragging one slider invalidates one row), a `generation` counter for
//! whole-state subscribers (auto-speak), and plan application with the
//! model's skip-if-equal filter.

use dioxus::prelude::*;

use crate::model::{self, Descriptors, Widen, WritePlan};

/// One parameter's live state: the value plus its widen entry (present only
/// while the value sits outside the descriptor range).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    /// Current value.
    pub value: f32,
    /// Widened bounds, if any (widen-never-clamp).
    pub widened: Option<Widen>,
}

/// The console's parameter store: 449 cell signals + a generation counter.
/// `Copy` — passed through context; cells are created once at the root.
#[derive(Clone, Copy)]
pub struct ParamStore {
    desc: CopyValue<Descriptors>,
    cells: CopyValue<Vec<Signal<Cell>>>,
    /// Bumped on every parameter mutation — the auto-speak subscription
    /// point (subscribing to cells directly would re-run per row).
    pub generation: Signal<u64>,
}

impl ParamStore {
    /// Build the store from the descriptor table (root `use_hook` only).
    pub fn new(desc: Descriptors) -> Self {
        let cells = (0..desc.len())
            .map(|i| {
                Signal::new(Cell {
                    value: desc.get(i).default,
                    widened: None,
                })
            })
            .collect();
        ParamStore {
            desc: CopyValue::new(desc),
            cells: CopyValue::new(cells),
            generation: Signal::new(0),
        }
    }

    /// The descriptor table.
    pub fn desc(&self) -> ReadableRef<'_, CopyValue<Descriptors>> {
        self.desc.read()
    }

    /// The cell signal at a flat index (rows subscribe to exactly this).
    pub fn cell(&self, idx: usize) -> Signal<Cell> {
        self.cells.read()[idx]
    }

    /// Set one parameter (recomputing its widen entry); bumps `generation`
    /// only when something actually changed.
    pub fn set(&mut self, idx: usize, value: f32) {
        if !value.is_finite() {
            return; // typed input can produce NaN/inf — never enters state
        }
        let widened = model::widen_for(self.desc.read().get(idx), value);
        let mut sig = self.cell(idx);
        let next = Cell { value, widened };
        if next != *sig.peek() {
            sig.set(next);
            *self.generation.write() += 1;
        }
    }

    /// Reset one parameter to its engine default (dblclick / row ↺).
    pub fn reset(&mut self, idx: usize) {
        let default = self.desc.read().get(idx).default;
        self.set(idx, default);
    }

    /// Non-reactive full snapshot — event handlers only (export, speak).
    pub fn snapshot(&self) -> Vec<f32> {
        self.cells.read().iter().map(|c| c.peek().value).collect()
    }

    /// Apply a write plan (skip-if-equal), bumping `generation` once if any
    /// cell changed.
    pub fn apply(&mut self, plan: &WritePlan) {
        let current = self.snapshot();
        let filtered = model::filter_plan(&current, plan);
        if filtered.0.is_empty() {
            return;
        }
        for &(idx, value) in &filtered.0 {
            let widened = model::widen_for(self.desc.read().get(idx), value);
            self.cell(idx).set(Cell { value, widened });
        }
        *self.generation.write() += 1;
    }

    /// Dirty count over a flat range (per-rack Δ badges — call inside a memo).
    pub fn dirty_count(&self, range: core::ops::Range<usize>) -> usize {
        let desc = self.desc.read();
        range
            .filter(|&i| model::is_dirty(desc.get(i), self.cell(i).read().value))
            .count()
    }
}
