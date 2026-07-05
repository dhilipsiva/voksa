# Handoff: voksa tuning console — ground-up UI/UX redesign

## Overview

**voksa** is a pure-Rust, rule-based formant speech synthesizer for **Lojban** —
no ML, no recorded voices; deterministic `text → parameter schedule → PCM`
running as a ~42 KB, zero-import WASM module inside an AudioWorklet. This
**tuning console** is its public face: a community instrument where Lojban
speakers type a sentence, listen, tweak any of ~465 acoustic parameters of the
voice, and **download a JSON config** that replays bit-identically in the native
CLI (`voksa --config file.json`). Shared configs are the project's bug reports
and its crowdsourced ground truth.

This package is a **from-scratch redesign** of the reference page
(`crates/voksa-web/www/index.html` in the voksa repo — a working but unstyled
prototype). It rethinks the information architecture: one screen, a pinned
**source** column (what is spoken) beside a scrolling **tuning** column ordered
by audience (broadest knobs first, the 377-parameter voice table last).

The end target is a **Dioxus (Rust / WASM) component library** embedded into
**dhilipsiva.dev**. Design in reusable components, not one bespoke page.

## About the design files

The files in this bundle are **design references authored as HTML/JS Design
Components** — prototypes that show the intended look, states, and behavior.
They are **not production code to copy**. The task is to **recreate these designs
as Dioxus components** in the dhilipsiva.dev codebase, wiring them to the real
voksa WASM engine (the reference page's *logic* is the source of truth for
engine wiring — port it; redesign only the *look*, which is what this package
specifies).

The interactive prototype is silent (no AudioWorklet). Its transcription and
waveform are deterministic **approximations** that read the live parameter state
so the interaction loop is demonstrable — the port replaces both with the real
wasm exports (`voksa_transcribe`, `voksa_render_params`).

## Fidelity

**High-fidelity.** Final colors, typography, spacing, component anatomy, and
interaction states are all specified against the **QUINE design system** (the
dhilipsiva personal system — terminal-first, void canvas, ember accent, IBM Plex
Mono voice). Recreate the UI faithfully using QUINE tokens/components as they
exist in the codebase. Every value in this bundle is a QUINE `var(--*)` token;
do not invent colors, type, or spacing.

## The four design files (open these)

> The `.dc.html` files render against the QUINE design system in `_ds/` (bundled
> here) and the shared `support.js` runtime — all included, all referenced by
> **relative** paths, so opening any file from this folder over a local http
> server renders it. `Voksa Phone Preview` embeds the workbench in an `<iframe>`.

1. **Voksa Workbench.dc.html** — the console itself. The reference
   implementation; inspect its logic class for any behavior detail not written
   below. Try the topbar **⎇ demo states** menu to see the five key states
   (fresh / dirty / config-loaded / flat-disabled / wasm-fail).
2. **Voksa IA & Flows.dc.html** — context + assumptions, the IA map, three
   personas (listener / tuner / reporter), the hard behavioral contracts, and
   the voice-table rationale.
3. **Voksa Phone Preview.dc.html** — the *same* responsive component in an iOS
   frame at 390 px (single column + fixed speak dock).
4. **Voksa Component Spec.dc.html** — the primary engineering artifact:
   ParamSlider anatomy with all four states drawn to spec, a full
   component → Dioxus mapping table, the state model, and the embed contract.
   **Read this one closely.**

## ⚠ Embed contract — read first

- **dhilipsiva.dev owns the navbar.** The console ships **no site chrome**: no
  logo, no nav, nothing sticky at `top:0`. Its top element is a non-sticky
  page-title row (display-font **voksa** 22px + `tuning console` label + version
  chip + Δ chip + λ about + ◐ theme).
- **Theme follows the host site** (`data-theme` inherit / `initialTheme` prop).
  The in-console ◐ toggle is a preview convenience — droppable in the port.
- **Sticky offsets are page-relative** (source column `top:12px`, rack-nav
  `top:8px`) so they clear whatever height the host navbar has.
- **Load / export live only in the share card.** The **⎇ demo-states** menu is
  design-review tooling — do not ship it (or gate it behind a dev flag).

## Screens / views

Single responsive screen, two columns; racks stack ≤1100px; fixed bottom speak
dock ≤720px.

### Source column (pinned left, ~392px, sticky)
- **Utterance** — mono text input (16px), QUINE Select of 18 phonetic-coverage
  test sentences + `⟳` next button + `NN/18` counter + English gloss, and four
  **FlagChips**: `flat` (no prosody), `xu` (question rise), `dotside`, `buffer`.
  `flat` disables the xu chip and all voice-shaping controls.
- **Phonetic analysis** — the live transcript line: `CAPS` = stress, `.` =
  syllable boundary, `‖` = pause, `(ɪ)` = inserted buffer vowel, `'` = [h]. A
  `?` opens a notation legend. This line is how non-experts report wrong
  phonetics — display it prominently and refresh it on **every** text/flag
  change (including programmatic ones).
- **Transport** — QUINE Button (primary, lg, block) `▶ speak` + a Switch
  "on change" (default ON, 400ms debounce, replaces the live node), a waveform
  canvas (phosphor `#38E3A6` on a dark void-800 + blueprint panel — deliberately
  dark in *both* themes, it's an instrument screen), a status line with a
  Spinner while speaking, and a ghost `⤓ wav` button.
- **Share the tuning** — Δ summary, notes textarea (travels inside the exported
  JSON), export / load buttons, a full-card drop target, a phosphor "config
  loaded" chip, and a send-back CTA (GitHub issue + mailto) on a blueprint
  panel.

### Tuning column (scrolls right)
Sticky **rack nav** (anchor chips with live Δ counts + preset Select + `↺ reset
all`), then four racks:
- **A · prosody** — 7 ParamSliders (pitch start/end, stress stretch/boost/
  loudness, xu rise, rate).
- **B · naturalness** — 9 ParamSliders + a one-line explainer + an **A/B latch**
  (current ↔ "naturalness off", the frozen phase-10 voice). The A/B comparison
  is the **#1 listening task** — make it prominent.
- **C · attitudinals** — a `∴` "invented, non-normative" theorem callout, seven
  cmavo **EmotionChips** (`.ui .uu .oi .ii .o'o .au .o'onai`, English gloss
  beneath, Δ dot) selecting an editor of 8 ParamSliders + `▶ try example` + per-
  emotion `↺`.
- **D · voice table** — the hard part. A **phoneme-grid navigator** (27 keycaps
  grouped into 6 manner classes, each carrying a Δ dot) selects a **per-phoneme
  editor**: a 3×3 formant matrix (F1–F3 × freq/bw/amp) + voicing/aspiration/
  duration. Stops add closure + burst matrices and timing; diphthongs and [h]
  expose only their one free parameter with an explanatory note. A **changed-
  only** switch dims untouched cells (never hides). Per-phoneme `↺` + table `↺`.

## The core primitive: ParamSlider (×465)

Grid `148px | 1fr | 96px`; track 3px; thumb 13×13 radius-sm; a **native
`input[type=range]` is overlaid transparent** on the custom visuals so drag,
arrow-key stepping, focus ring, and screen-reader semantics all come for free.

**Four states** (drawn to spec in the Component Spec doc):
- **default** — grey thumb (`--text-muted` / `--border-strong`), faint default
  tick, no fill.
- **modified** — `fround(v) ≠ fround(def)`: ember dot + ember fill from the
  default tick to the thumb + ember value; a `↺` reset appears on row hover;
  double-click resets.
- **widened** — a value loaded/typed outside the descriptor range **widens**
  min/max and relaxes step to `any`; an amber `⤢` marker shows; the readout
  always shows the true value. **Never clamp.**
- **disabled** — `flat` mode: opacity 0.45, real `disabled` attr.

The readout is a **tap-to-type** input (`inputMode=decimal`); Enter/blur commits
(may widen), Esc reverts. `MatrixCell` is a compact label-above variant for the
formant matrix (11px thumb).

## Interactions & behavior
- **Auto-speak** — any mutator kicks a 400 ms debounce when enabled + engine
  ready; a new render **replaces** the live AudioWorkletNode, never stacks.
- **First play requires a user gesture** (browser autoplay) — create/resume the
  AudioContext in the speak handler.
- **Sentence picker** applies text + the sentence's flags then auto-speaks;
  manual typing reverts the Select to "— free text —".
- **Presets** = reset-to-defaults + knob overrides. **Deviation from the
  reference page (deliberate): flags are preserved**, not cleared.
- **flat** greys prosody + attitudinals + voice table + the xu flag; text,
  playback, and share stay live.
- **wasm-fail** — an error Callout with a retry button; sliders stay live,
  speak is disabled.
- **Responsive** — ≤1100px columns stack; ≤720px single column + fixed bottom
  speak dock, phoneme grid wraps above the editor (34px touch keycaps).

## State model & engineering contracts
- **paths** — `k.<knob>` (16) · `a.<cmavo>.<i>` (7×8) · `v.<phoneme>.<i>` (377,
  flat per-item arrays in `VoiceTable::to_array` order).
- **ranges map** — `path → {min,max,step:'any'}` exists only while widened;
  reset removes it.
- **diffing** — in f32 space (`fround`), driving dots, Δ counts, and export.
- **export** — flat text/flags/16 knobs always; attitudinals + phonemes
  **delta-only** (stops nest `closure`/`burst` objects); plus `phonetics`,
  `notes`, `sampleRate`, `voksaVersion`.
- **load = REPLACE** — rebuild from engine defaults, apply config keys, widen as
  needed; absent keys → defaults; preset → custom. Never merge over dirty state
  (must equal what `voksa --config` plays).
- **defaults come from the engine** — seed every control from
  `voksa_default_params()` at runtime. `voksa-engine-data.json` in this bundle
  mirrors that block for design only; **do not hand-copy defaults** in the port.

The full f32 layout (449 floats: prosody 0–6, attitudinals 7–62, voice table
63–439, naturalness 440–448) and the C-ABI are documented in the voksa repo's
`docs/handoff-dhilipsiva-dev.md` and `crates/voksa-core/src/phonemes.rs`.

## Design tokens (QUINE — all `var(--*)`)
- **Color** — dark-first. `--bg-base` void `#0B0B10`; surfaces `--surface-card
  #181822` / `--bg-raised #111119` / `--surface-inset`. Text `--text-strong
  #F4F3EF` (warm off-white, never pure) / `--text-body` / `--text-muted` /
  `--text-faint`. Accent **ember** `--accent #F2542D` (+ `--ember-300` hover,
  `--accent-soft` fill). **phosphor** `--alive #38E3A6` (alive/success). Signal
  `--amber-500` / `--sky-500` / `--crimson-500`. `[data-theme="light"]` inverts
  to paper & ink; ember keeps burning. **Use semantic tokens, not raw palette
  tokens** — raw ones don't flip under light theme.
- **Type** — `--font-mono` IBM Plex Mono (the default UI voice), `--font-sans`
  IBM Plex Sans (reading), `--font-display` Space Grotesk (headlines),
  `--font-serif` (the one italic pull-quote). Scale 11→88px, 1.25 ratio.
- **Spacing** — strict 4px grid (`--space-1..10`). Controls 36/28/44px.
- **Radii** — tight: `--radius-sm 4` / `--radius-md 6` / `--radius-lg 10`.
- **Effects** — hairline `--border-subtle`; the signature lift is a colored
  **glow** (`--glow-ember` / `--glow-phosphor`), not a drop shadow; focus =
  `--ring` (2px ember, offset); `--bg-grid` blueprint lattice on engineered
  panels; glass (`--blur-glass`) only on sticky bars.

Icons: mathematical Unicode glyphs in IBM Plex Mono (`∀ ∃ λ ⊢ ⊥ ∴ ≡ ⎇ ⟳ ↺`).
**No emoji.**

## Data & assets in this bundle
- `voksa-engine-data.json` — every descriptor, default, range, the 18 sentences,
  presets, the demo config, capabilities/size copy. **Regenerate from the wasm
  in the port** (`voksa_default_params()`); this is a design-time mirror.
- `voksa-help-text.json` — a **keyed skeleton** for the `?` help popovers. The UI
  falls back to `// help pending — <key>`. Field keys are shared across
  phonemes/emotions (`vt.fields.*`, `att.fields.*`). Fill the copy; keys stay
  stable.
- `assets/voksa-mark.svg`, `assets/mark-mono.svg` — the logomark (from QUINE).
- `voksa-DESIGN-NOTES.md` — the working design log (rationale, decisions).
- `_ds/` — the QUINE design system bundle the prototypes render against. In the
  port, use the codebase's own QUINE integration, not this copy.
- `ios-frame.jsx`, `support.js` — prototype scaffolding (device frame + DC
  runtime); not part of the design, needed only so the HTML renders.

## Source of truth
The **voksa repo** is authoritative for engine wiring: port the *logic* of
`crates/voksa-web/www/index.html` (config round-trip, `setSliderExact` range
widening, delta export, auto-speak, transcription refresh) and copy
`crates/voksa-web/pkg/voksa_web_bg.wasm` + `www/voksa-processor.js` verbatim.
This bundle governs the *look and structure* only. See
`docs/handoff-dhilipsiva-dev.md` in that repo for the hard WASM constraints
(zero imports, main-thread compile, TextEncoder on the main thread, cache-bust
the worklet, `application/wasm` MIME, gesture-gated audio).
