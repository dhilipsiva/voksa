# ADR 0003 — voksa-console: engine-in-app Dioxus component (player-only worklet)

Date: 2026-07-05 · Status: ACCEPTED (owner chose Option A after a written
pros/cons comparison)

## Context

The tuning console is being redesigned (QUINE design system; the handoff
bundle is vendored at `docs/design/tuning-console/`) and ported from the
plain-HTML page (`crates/voksa-web/www/index.html`) to a **Dioxus 0.7
component-library crate**, `crates/voksa-console`, consumed by dhilipsiva.dev
as a git dependency. The plain page's audio design (ADR-less, Phase 9): the
zero-import `voksa_web_bg.wasm` is compiled on the main thread, passed into an
AudioWorklet via `processorOptions`, and the worklet both RENDERS and plays.

For a Rust component library that architecture inverts the costs: every
consumer must serve two extra static assets (`.wasm` + processor JS) with
correct MIME and cache-busting, and the Dioxus app would still need a second
engine access path (transcription, slider defaults) on the main thread —
either hand-written bindings into a second wasm instance or a duplicate
linked engine that can version-skew against the served asset.

## Decision

`voksa-console` links `voksa-web` as a plain **rlib**: `synth()`,
`transcription()`, and `default_params()` are direct, typed Rust calls inside
the app's own wasm. Audio playback uses a ~40-line **player-only**
AudioWorklet (JS, `include_str!`-embedded, loaded via a Blob URL) that
receives one transferable Float32Array per utterance and posts `done`; the
app disconnects the old node before posting a new render (replace, never
stack). An optional prop accepts an asset URL for the worklet as a
Safari/CSP escape hatch.

Consequences:

- **Zero embed assets**: a host adds the git dependency and mounts
  `TuningConsole` — no `.wasm` copying, no MIME configuration, no
  cache-busting, no `TextEncoder` workaround (the entire "gotchas" section of
  the old handoff doc becomes moot for Rust hosts).
- **One engine**: UI, engine, and the config `voksaVersion` stamp compile
  from the same crate graph; version skew is impossible by construction.
  Errors are typed `Result`s end-to-end.
- **Rendering moves to the main thread**. Typical utterances render in
  <100 ms; the worst case is bounded by the adapter's 600 s offline ceiling.
  If community configs ever jank in practice, rendering can move to a Web
  Worker without changing the component API (recorded backlog).

## What this does NOT change

- `crates/voksa-web` keeps its raw C-ABI cdylib, the zero-imports invariant,
  and the 43 KB `cargo xtask wasm-size` gate: that artifact remains the
  documented embedding surface for **non-Rust** hosts (plus the wasip2
  component, ADR 0002). The Phase-9 wasm-in-worklet design note in CLAUDE.md
  continues to describe THAT surface; this ADR governs only the console.
- The engine pipeline, param-block layout (449 floats, frozen append-only),
  and config JSON schema are untouched; the console's exports must keep
  replaying bit-identically through `voksa --config` (enforced by a native
  round-trip test against voksa-cli's parser).

## Alternatives rejected

- **Verbatim port (wasm-in-worklet)**: keeps rendering off the main thread
  but permanently taxes every consumer with asset wiring, and still needs a
  second engine path for transcription/defaults — the duplicated-engine +
  version-skew hazard outweighs the render-thread benefit for this use case.
- **`document::eval` audio glue**: no transferable-buffer path, stringly
  lifecycle management. web-sys is used instead.

## New dependencies (license rule check)

`dioxus` 0.7, `wasm-bindgen`, `web-sys`, `js-sys`, `wasm-bindgen-futures`,
`gloo-timers`, `console_error_panic_hook` — all MIT OR Apache-2.0, within the
repo's dependency rule. They enter only `voksa-console`/`voksa-console-demo`;
the shipping `voksa-web` cdylib still has NO wasm-bindgen runtime dependency.
