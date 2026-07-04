# Handoff: embed the voksa tuning console into dhilipsiva.dev

This is a self-contained brief for a design pass + a Claude Code session working
in the `dhilipsiva/dhilipsiva.dev` repo. It documents everything the embed
needs; the voksa repo is the source of truth (tag `demo-attitudinal`).

## What you are embedding

voksa is a pure-Rust, rule-based Lojban speech synthesizer that runs entirely
in the browser (WASM inside an AudioWorklet — no server, no network calls after
load). The tuning console lets Lojban community members fiddle prosody +
attitudinal (emotion) parameters, listen, and **download a config JSON** to
share back. That JSON replays bit-identically via the native CLI
(`voksa --config file.json`), which is the whole point: crowdsourced tuning.

Reference implementation (working, unstyled): `crates/voksa-web/www/index.html`
in the voksa repo. Port its LOGIC; redesign its LOOK to fit dhilipsiva.dev.

## Files to copy from voksa (build artifacts + worklet)

1. `crates/voksa-web/pkg/voksa_web_bg.wasm` — built with
   `wasm-pack build --release --target web crates/voksa-web` (or take it from a
   checkout at the `demo-attitudinal` tag after running
   `cargo xtask wasm-size`, which builds + verifies it). ~35 KB gzipped.
2. `crates/voksa-web/www/voksa-processor.js` — the AudioWorkletProcessor.
   Copy VERBATIM; do not port it into a bundler module (worklets load via
   `audioWorklet.addModule(url)`).

## Hard technical constraints (violating these breaks audio silently)

- The `.wasm` declares **zero imports**. Instantiate with
  `new WebAssembly.Instance(module, {})` INSIDE the worklet; compile on the
  main thread (`WebAssembly.compileStreaming(fetch(url))`) and pass the Module
  via `processorOptions`. Serve the wasm with the `application/wasm` MIME type
  (or swap to `WebAssembly.compile(await resp.arrayBuffer())`).
- Encode the text with `TextEncoder` on the MAIN thread and pass bytes —
  Firefox's AudioWorkletGlobalScope has no TextEncoder.
- Cache-bust the worklet URL (`voksa-processor.js?v=N`) — `addModule` caches
  aggressively.
- Serve over http(s), not `file://`. No COOP/COEP headers needed; no
  SharedArrayBuffer used.
- Audio starts only after a user gesture (create/resume the AudioContext in a
  click handler).

## The C-ABI (exports of the wasm)

- `voksa_alloc(len) -> ptr` / `voksa_dealloc(ptr, len)` — scratch buffers the
  page writes into (text bytes, f32 params).
- `voksa_render_params(text_ptr, text_len, flags, sample_rate, params_ptr,
  params_len) -> ptr` — renders; returns the f32 PCM base pointer (null on
  error). Read the length with `voksa_out_len()`, copy the samples OUT of wasm
  memory immediately (memory growth detaches views), then `voksa_free_f32(ptr, len)`.
- `flags` bits: `1` flat (no prosody), `2` xu rise, `4` dotside, `8` buffer.

## The f32 param block (63 floats; shorter blocks default the rest)

Indices 0–6 (prosody): `declination_start_hz, declination_end_hz,
stress_duration_factor, stress_f0_excursion_hz, stress_amp_factor, xu_rise_hz,
rate`. Defaults: `120, 95, 1.5, 20, 1.2, 25, 1`.

Indices 7–62 (attitudinals): 7 kinds × 8 fields, kind-major. Kind order:
`ui, uu, oi, ii, o'o, au, o'onai`. Field order: `f0_mean_hz, f0_range_mult,
rate_mult, oq, tilt, di, vibrato_hz, aspiration`. The pinned defaults live in
the reference page's `ATTITUDINALS` descriptor (and in
`crates/voksa-core/src/attitudinal.rs` — `AttitudinalKind::deviation()`).
**Do not reorder either axis** — the layout is the contract with the engine and
the CLI.

## The config JSON (what users share back)

Flat keys for text/flags/prosody (same names as the layout above), plus a
delta-only `attitudinals` object:

```json
{
  "text": "coi munje .ui",
  "xu": false, "dotside": false, "buffer": false, "flat": false,
  "rate": 1.0,
  "attitudinals": { "ui": { "f0_mean_hz": 22, "di": 0.1 } },
  "notes": "joy reads better with a small creak",
  "voksaVersion": "0.0.1"
}
```

Missing keys = defaults, so minimal JSON is valid. Loading must NOT clamp
out-of-range values to slider bounds (widen the slider instead) — the CLI
accepts any finite value and web/CLI replay must stay identical.

## Design asks (from dhilipsiva)

- Lojban labels with English sub-text. Section headers are the cmavo
  (`.ui`, `.uu`, `.oi`, `.ii`, `.o'o`, `.au`, `.o'onai`) with glosses
  (happiness / pity / complaint-pain / fear / patience / desire / anger).
  A "try it" affordance per attitudinal (example phrases: `coi munje .ui`,
  `mi klama .uu`, `coi munje .oi`, `coi munje .ii`, `mi klama .o'o`,
  `mi djica .au`, `mi fengu .o'onai`).
- Two tiers: Basic (7 prosody sliders + 4 flags) and Advanced (7 attitudinal
  panels × 8 sliders). Reset per panel + global.
- Download config JSON / Load config JSON / Download WAV (16-bit mono RIFF —
  the reference page has a 15-line encoder) / a notes field that lands in the
  JSON / a waveform or equivalent visual.
- **Auto-speak on change** (see the reference page): any input change re-speaks
  after a ~400 ms debounce, gated by a "speak on change" toggle (default ON);
  a new utterance REPLACES the playing one (disconnect the previous
  AudioWorkletNode), never stacks. Keep the manual ▶ Speak button too.
- A clear "send me your config" call-to-action (mailto:dhilipsiva@pm.me or a
  GitHub issue link on dhilipsiva/voksa — owner's choice).
- State plainly that the attitudinal mappings are voksa's invention (the CLL
  defines the *meaning* of attitudinals, not their sound) — that's WHY
  community tuning matters.

## Attribution

voksa: MIT OR Apache-2.0, github.com/dhilipsiva/voksa. Synthesis engine:
vendored fork of klattsch-core 0.1.1 (MIT, Tony Gies). No trackers needed; the
page works fully offline after load.
