# ADR 0002 — wasm32-wasip2 WIT component (`voksa-component`)

Date: 2026-07-05 · Status: ACCEPTED (owner approved with the Phase-11 plan)

## Context

The browser adapter (`voksa-web`) ships a raw C-ABI wasm module purpose-built
for the AudioWorklet (zero imports, hand-written JS glue). That surface is
deliberately NOT a general embedding API: consumers outside the demo (wasmtime
hosts, other languages via the component model, server-side tooling) would
have to re-implement pointer/length marshalling by hand.

The WebAssembly Component Model gives a typed, language-neutral surface for
exactly this. Rust ≥ 1.82 emits a component DIRECTLY for the
`wasm32-wasip2` target — no adapter module, no cargo-component wrapper.

## Decision

Add `crates/voksa-component` (cdylib, `publish = false`):

- `wit/voksa.wit` — `package voksa:synth@0.1.0`, world `voksa` exporting:
  - `synthesize(text: string, flags: u32, sample-rate: u32) -> result<list<f32>, string>`
  - `transcribe(text: string, flags: u32) -> result<string, string>`
  - `version() -> string`
- `wit_bindgen::generate!` guest implementation delegating to `voksa_web`'s
  `synth` / `transcription` (the rlib) — browser parity by construction: both
  surfaces compile the same functions.
- `flags` uses the same bit layout as the C-ABI (`flat 0x1, xu 0x2,
  dotside 0x4, buffer 0x8`).

Gate: `cargo xtask component` — build for wasm32-wasip2 → `wasm-tools
validate` (must be a valid component) → WIT drift check (`wasm-tools
component wit` output must match the checked-in `.wit`) → gzip size canary
(200 KB budget; a separate artifact from the 43 KB voksa-web gate, which is
untouched). New `component` CI job runs the gate.

## License sign-off

`wit-bindgen` (and its `wit-bindgen-rt` runtime) are licensed
`Apache-2.0 WITH LLVM-exception` — a different SPDX ID than the repo's
"MIT/Apache only" dependency rule. The LLVM exception strictly RELAXES
Apache-2.0 (it lifts the §4 attribution burden for binary redistribution), so
the dependency is at least as permissive as plain Apache-2.0. The owner
approved this dependency as part of the Phase-11 plan approval (2026-07-05);
this ADR records it. The dependency is confined to `voksa-component` — it
never enters voksa-core, the browser wasm, or the CLI.

## Consequences

- flake.nix devshell gains the `wasm32-wasip2` rust-std and `wasm-tools`.
- The component is a build artifact, not a published crate (crates.io
  publishing is post-v0.1 backlog; `publish = false`).
- The C-ABI worklet surface remains the shipping browser path — the component
  is additive, and the voksa-web size gate is unaffected.
