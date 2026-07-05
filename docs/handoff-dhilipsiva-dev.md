# Handoff: embed the voksa tuning console into dhilipsiva.dev

This is a self-contained brief for a design pass + a Claude Code session working
in the `dhilipsiva/dhilipsiva.dev` repo. It documents everything the embed
needs; the voksa repo is the source of truth (current `main` — the v0.1.0
release; the `v0.1.0` tag lands once CP3 is signed off).

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
   checkout of current `main` (v0.1.0) after running
   `cargo xtask wasm-size`, which builds + verifies it). ~42 KB gzipped
   (the xtask gate asserts < 43 KB).
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
- `voksa_transcribe(text_ptr, text_len, flags) -> ptr` — the phonetic
  transcription (UTF-8; length via `voksa_out_len()`, free with
  `voksa_dealloc(ptr, len)`; null on error). Notation: syllable dots, CAPITALS
  on the stressed syllable (the CLL convention), ` ‖ ` pauses, `(ɪ)` buffer
  vowels — e.g. `coi MUN.je`, `la DJAN ‖ cu KLA.ma`, `V(ɪ)RU.si`. **Display
  this line prominently** — it is how the community reports wrong phonetics —
  and embed it in the exported config JSON as `phonetics` (the reference page
  does both; refresh it on EVERY text/flag mutation path, including
  programmatic ones — `.value =` fires no `input` event).
- `flags` bits: `1` flat (no prosody), `2` xu rise, `4` dotside, `8` buffer.

## The f32 param block (449 floats; shorter blocks default the rest)

Indices 0–6 (prosody): `declination_start_hz, declination_end_hz,
stress_duration_factor, stress_f0_excursion_hz, stress_amp_factor, xu_rise_hz,
rate`.

Indices 7–62 (attitudinals): 7 kinds × 8 fields, kind-major. Kind order:
`ui, uu, oi, ii, o'o, au, o'onai`. Field order: `f0_mean_hz, f0_range_mult,
rate_mult, oq, tilt, di, vibrato_hz, aspiration`.

Indices 63–439 (per-phoneme voice table, 377 floats): the normative ordering is
the `VoiceTable::to_array` doc comment in `crates/voksa-core/src/phonemes.rs` —
vowels a e i o u y (12 each: f1,bw1,amp1, f2,bw2,amp2, f3,bw3,amp3, voicing,
aspiration, dur_ms) → 16 diphthong durations → stops p t k b d g (24 each:
closure 11 + burst 11 + closure_ms + burst_ms) → fricatives f v s z c j x →
nasals m n → liquids l r → [h] duration → buffer vowel.

Indices 440–448 (naturalness, 9 floats — Phase 11, DEFAULT ON): `flutter,
breath_aspiration, baseline_oq_delta, baseline_tilt_delta, micro_f0_hz,
obstruent_f0_hz, final_lengthen, cluster_shorten, undershoot` (pinned defaults
25 / 0.06 / +0.10 / −0.10 / 4 / 6 / 1.3 / 0.15 / 0.35; semantics in
docs/phonology.md §9.2). These are Basic-tab sliders next to the prosody
knobs, plus a "Naturalness off" preset that sets all nine to their identity
values for A/B listening.

The layout is frozen append-only, so shorter blocks stay valid forever: a
7-float (demo-basic), 63-float (demo-attitudinal), or 440-float
(demo-advanced) block defaults everything past its end.

**Do not hand-copy defaults and do not reorder** — call the
`voksa_default_params()` export (length via `voksa_out_len()`, free with
`voksa_free_f32`) from a main-thread instance and seed every slider from it,
exactly as the reference page does. UI defaults then equal the engine's tables
by construction.

## The config JSON (what users share back)

Flat keys for text/flags/prosody/naturalness (same names as the layout above),
plus delta-only `attitudinals` and `phonemes` objects:

```json
{
  "text": "coi munje .ui",
  "xu": false, "dotside": false, "buffer": false, "flat": false,
  "rate": 1.0,
  "flutter": 25.0,
  "attitudinals": { "ui": { "f0_mean_hz": 22, "di": 0.1 } },
  "notes": "joy reads better with a small creak",
  "voksaVersion": "0.1.0"
}
```

`phonemes` holds per-letter voice-table overrides (stops nest `closure`/`burst`
objects) — copy the export shape from the reference page rather than inventing
key names. The page also auto-stamps `phonetics` (the transcription line the
tuner was looking at), `sampleRate`, and `voksaVersion` (must equal the version
of the wasm you embed — 0.1.0 on current `main`; the voksa repo guards its own
demo's stamp with a native test, `crates/voksa-web/tests/version.rs`).

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
- Two tiers: Basic (7 prosody + 9 naturalness sliders + 4 flags, with a
  "Naturalness off" preset) and Advanced (7 attitudinal
  panels × 8 sliders, plus the per-phoneme voice table: six sections of
  collapsible panels, 377 sliders). Reset per panel + global.
- A **phonetic sentence picker**: copy `www/sentences.json` (18 curated
  coverage sentences, each with an English what-it-exercises gloss + optional
  flags) and render it as a dropdown + a "next" cycle button that sets text +
  flags and auto-speaks. Every entry is gated by a native test in the voksa
  repo, so the list is guaranteed synthesizable.
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
