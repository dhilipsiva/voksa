# Handoff: embed the voksa tuning console into dhilipsiva.dev

A self-contained brief for a Claude Code session working in the
`dhilipsiva/dhilipsiva.dev` repo. The voksa repo is the source of truth
(current `main`, v0.1.0). There are two embedding paths:

1. **Primary — a Dioxus component (git dependency).** If dhilipsiva.dev is (or
   can host) a Dioxus app, add one dependency and mount one component. The
   engine is compiled into your wasm; no second wasm file, no C-ABI, no
   worklet glue to copy. **Use this.**
2. **Secondary — the raw C-ABI wasm.** For a non-Rust host (a plain JS/React
   page), instantiate `voksa_web_bg.wasm` yourself. Heavier to wire; documented
   at the end.

## What you are embedding

voksa is a pure-Rust, rule-based Lojban speech synthesizer that runs entirely
in the browser (no server, no network after load). The tuning console lets
Lojban community members fiddle prosody + naturalness + attitudinal (emotion)
parameters and the full per-phoneme voice table, listen, and **export a config
JSON** to share back. That JSON replays bit-identically via the native CLI
(`voksa --config file.json`) — crowdsourced voice tuning is the whole point.

The console is `crates/voksa-console` in the voksa repo (ADR 0003): a Dioxus
0.7 component library. It links `voksa-web` as a plain rlib — `synth` /
`transcription` / `default_params` are direct Rust calls — and plays audio
through a ~40-line player-only AudioWorklet it owns. The standalone runner
`crates/voksa-console-demo` is the reference integration.

---

## Primary path: mount as a Dioxus component

### 1. Add the dependency

```toml
# Cargo.toml
[dependencies]
voksa-console = { git = "https://github.com/dhilipsiva/voksa", tag = "console-v1" }
dioxus = { version = "0.7", features = ["web"] }
```

(Pin `tag`/`rev` — the crate is `publish = false`, so it is only consumed as a
git dependency. `voksa-console` pulls its own wasm-only deps —
`web-sys`/`js-sys`/`wasm-bindgen`/`gloo-timers` — behind
`cfg(target_arch = "wasm32")`, so a native build of your app still compiles.)

### 2. Mount the component

```rust
use dioxus::prelude::*;
use voksa_console::TuningConsole;

#[component]
fn VoicePage() -> Element {
    rsx! {
        // Your QUINE token sheet must already be on the page (see §4).
        TuningConsole { initial_theme: "dark" }
    }
}
```

`TuningConsole` mounts once and owns all of its state (the 449-parameter store,
the audio graph, the help/about UI). It renders a single `div.vx-root`.

### 3. Props (`TuningConsoleProps`)

| Prop | Type | Default | Meaning |
|------|------|---------|---------|
| `initial_theme` | `Option<String>` | `None` | Sets `data-theme` on the root (`"dark"`/`"light"`). `None` inherits the host page's theme. |
| `class` | `Option<String>` | `None` | Extra classes appended to `vx-root` (host layout hooks). |
| `inline_styles` | `bool` | `false` | Inline the component stylesheet with `document::Style` instead of linking the manganis asset — for consumers **not** building with `dx` (see §4). |

### 4. CSS + QUINE tokens (the one integration requirement)

The crate ships **component classes only** (`console.css`, all `vx-*`),
styled entirely against QUINE **semantic tokens** (`var(--ember-500)`,
`var(--surface-card)`, `var(--font-mono)`, …). Two things must be true on the
page:

- **The component stylesheet is loaded.** With a `dx build`, manganis
  (`asset!("/assets/console.css")`) bundles it across the git dependency
  automatically — nothing to do. If you are **not** building with `dx`, pass
  `inline_styles: true` (the crate embeds the CSS via `include_str!`, exposed
  as `voksa_console::assets::CONSOLE_CSS_SOURCE`), or link that source yourself.
  The CSS contains no `url()` references, so inlining is exact.
- **The host provides the QUINE token sheet.** The console does not ship fonts
  or token values — the host page must define the QUINE `--*` custom properties
  (colors, type, spacing, motion) on an ancestor of the mount point, or the
  console renders unstyled. The reference token sheet the standalone demo
  vendors is `crates/voksa-console-demo/assets/quine-tokens.css`; on
  dhilipsiva.dev, use the site's own QUINE integration.

`crates/voksa-console-demo/src/main.rs` is the minimal working example:

```rust
use dioxus::prelude::*;
use voksa_console::TuningConsole;

const TOKENS: Asset = asset!("/assets/quine-tokens.css");

fn main() { dioxus::launch(App); }

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TOKENS }   // host-provided QUINE tokens
        div { class: "vx-demo-page", TuningConsole {} }
    }
}
```

Run it with `cd crates/voksa-console-demo && dx serve`.

### 5. Embed contract (already honored by the component)

No site chrome; the title row is non-sticky (the host owns the navbar); the
source column's sticky offset is page-relative; theme follows the host
`data-theme`. Audio starts only after a user gesture — the first ▶ speak click
unlocks the AudioContext (browser autoplay policy), after which "speak on
change" can drive playback. Render is on the main thread (< 100 ms for a
typical sentence).

### What the component already implements

You get all of this out of the box — no design work required beyond providing
QUINE tokens:

- Lojban-labeled controls with English glosses; a `?` help popover on every
  control resolving ~74 authored entries; an about panel (`λ`).
- **Prosody + naturalness** (7 + 9 knobs, with an A/B latch that hears the
  voice with the naturalness layer off), **attitudinals** (7 emotion panels ×
  8 deviation fields, with per-emotion "try it" examples), and the **full
  per-phoneme voice table** (41 phonemes, 377 parameters, a changed-only view).
- A **phonetic sentence picker** (18 curated coverage sentences, each gated by
  a native synthesize test), a **live phonetic-analysis line** (colored
  syllable/stress/pause/buffer tokens), a waveform, WAV download, auto-speak,
  and the share loop: **export config JSON / load config (file or drag-drop,
  REPLACE semantics, widen-never-clamp) / a notes field** that travels in the
  JSON, plus a "send it back" CTA (GitHub issue + mailto).

---

## The config JSON (what users share back)

The console's export/load already produces and consumes this; the schema is the
CLI's (`voksa --config`), so a shared config replays bit-identically. Flat keys
for text/flags/prosody/naturalness (same names as the layout below), plus
**delta-only** `attitudinals` and `phonemes` objects (only changed values),
plus `phonetics`/`notes`/`sampleRate`/`voksaVersion` stamps:

```json
{
  "text": "coi munje .ui",
  "xu": false, "dotside": false, "buffer": false, "flat": false,
  "rate": 1.0,
  "flutter": 25.0,
  "attitudinals": { "ui": { "f0_mean_hz": 22, "di": 0.1 } },
  "phonemes": { "s": { "dur_ms": 96 }, "k": { "burst": { "amp3": 1.0 }, "closure_ms": 52 } },
  "phonetics": "coi MUN.je",
  "notes": "joy reads better with a small creak",
  "sampleRate": 48000,
  "voksaVersion": "0.1.0"
}
```

`phonemes` holds per-letter voice-table overrides (stops nest `closure`/`burst`
objects). Missing keys = defaults, so minimal JSON is valid. `voksaVersion` is
stamped from `env!("CARGO_PKG_VERSION")` at build time — it always matches the
engine you embed. Loading REPLACES all state and must NOT clamp out-of-range
values to slider bounds (the console widens the slider instead) — the CLI
accepts any finite value and web/CLI replay must stay identical.

## Phonetic transcription notation

The live analysis line (and the exported `phonetics`) uses CLL-flavored
notation: syllable dots, CAPITALS on the stressed syllable, ` ‖ ` pauses, `(ɪ)`
buffer vowels — e.g. `coi MUN.je`, `la DJAN ‖ cu KLA.ma`, `V(ɪ)RU.si`. Numbers
show their normalized cmavo (`li 3.14` → `li ci pi pa vo`). Display it
prominently — it is how the community reports wrong phonetics.

## The f32 parameter layout (frozen, append-only)

The console addresses parameters by this 449-float layout; you only need it for
the C-ABI path or to reason about the config schema.

- **0–6 prosody**: `declination_start_hz, declination_end_hz,
  stress_duration_factor, stress_f0_excursion_hz, stress_amp_factor, xu_rise_hz,
  rate`.
- **7–62 attitudinals**: 7 kinds × 8 fields, kind-major. Kinds:
  `ui, uu, oi, ii, o'o, au, o'onai`. Fields: `f0_mean_hz, f0_range_mult,
  rate_mult, oq, tilt, di, vibrato_hz, aspiration`.
- **63–439 voice table (377)**: the `VoiceTable::to_array` order in
  `crates/voksa-core/src/phonemes.rs` — vowels a e i o u y (12 each) → 16
  diphthong durations → stops p t k b d g (24 each: closure 11 + burst 11 +
  closure_ms + burst_ms) → fricatives f v s z c j x → nasals m n → liquids l r
  → `[h]` duration → buffer vowel.
- **440–448 naturalness (9, DEFAULT ON)**: `flutter, breath_aspiration,
  baseline_oq_delta, baseline_tilt_delta, micro_f0_hz, obstruent_f0_hz,
  final_lengthen, cluster_shorten, undershoot` (pinned 25 / 0.06 / +0.10 /
  −0.10 / 4 / 6 / 1.3 / 0.15 / 0.35; semantics in docs/phonology.md §9.2).

The layout is frozen append-only, so shorter blocks stay valid: a 7-, 63-, or
440-float block defaults everything past its end. **Do not hand-copy defaults**
— seed from `voksa_web::default_params()` (the console does this at runtime, so
UI defaults equal the engine tables by construction).

The attitudinal mappings are voksa's **invention** — the CLL defines the
*meaning* of an attitudinal, never its sound. That is exactly why community
tuning matters; the console says so on its attitudinal rack.

---

## Secondary path: raw C-ABI (non-Rust hosts)

Skip this if you are using the Dioxus component. For a plain JS host, embed the
zero-import wasm directly:

- Build it with `wasm-pack build --release --target web crates/voksa-web` (or
  take `crates/voksa-web/pkg/voksa_web_bg.wasm` after `cargo xtask wasm-size`,
  which builds + verifies it — ~42 KB gzipped, the gate asserts < 43 KB **and
  zero imports**).
- The `.wasm` declares **zero imports**: instantiate with
  `new WebAssembly.Instance(module, {})` inside a worklet; compile on the main
  thread (`WebAssembly.compileStreaming(fetch(url))`) and pass the Module via
  `processorOptions`. Serve it with `application/wasm` MIME.
- Encode text with `TextEncoder` on the **main** thread and pass bytes —
  Firefox's AudioWorkletGlobalScope has no TextEncoder. Cache-bust the worklet
  URL (`addModule` caches aggressively). Serve over http(s), not `file://`. No
  COOP/COEP needed. Audio starts only after a user gesture.
- Exports: `voksa_alloc(len)->ptr` / `voksa_dealloc(ptr,len)`;
  `voksa_render_params(text_ptr, text_len, flags, sample_rate, params_ptr,
  params_len)->ptr` (f32 PCM base; length via `voksa_out_len()`; copy OUT
  immediately — memory growth detaches views — then `voksa_free_f32(ptr,len)`);
  `voksa_transcribe(text_ptr, text_len, flags)->ptr` (UTF-8; free with
  `voksa_dealloc`); `voksa_default_params()->ptr` (the canonical block; seed
  every slider from it). `flags` bits: `1` flat, `2` xu, `4` dotside, `8`
  buffer. The wasm-wasip2 WIT component (`crates/voksa-component`,
  `voksa:synth@0.1.0`) is a third option for WASI hosts (ADR 0002).

## Attribution

voksa: MIT OR Apache-2.0, github.com/dhilipsiva/voksa. Synthesis engine:
vendored fork of klattsch-core 0.1.1 (MIT, Tony Gies). No trackers; the page
works fully offline after load.
