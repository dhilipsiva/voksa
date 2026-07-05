//! voksa tuning console — a Dioxus component library over the voksa engine.
//!
//! ADR 0003: the engine is linked as a plain rlib (`voksa_web::synth` /
//! `transcription` / `default_params` are direct calls); audio playback is a
//! player-only AudioWorklet. The look and structure follow the QUINE design
//! handoff vendored at `docs/design/tuning-console/` — the Workbench
//! prototype there is the reference implementation for behavior.
//!
//! `model` is pure Rust (no dioxus, no web) and carries every contract the
//! console promises: the frozen 449-float path space, widen-never-clamp,
//! f32-space dirty diffing, delta-only export, REPLACE load, presets. The
//! UI layers (C2+) are thin projections over it.

#![warn(missing_docs)]

pub mod assets;
pub mod audio;
pub mod engine;
pub mod model;
pub mod ui;

pub use ui::{TuningConsole, TuningConsoleProps};
