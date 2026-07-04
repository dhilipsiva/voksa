# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`voksa` — a pure-Rust, rule-based parallel-formant (Klatt-style) speech synthesizer for Lojban (jbo). No ML, no eSpeak data, no training corpus. Pipeline: text → deterministic parameter schedule → PCM. Targets: browser (WASM, single-threaded AudioWorklet) and native (cpal). Metric units throughout (Hz, ms, semitones).

`voksa` = Lojban gismu: x1 is the voice/speech sounds of x2.

## Current state

Phase 7 complete and tagged (CP1 signed off 2026-07-03; scores in docs/listening/phase7.md). The prosody transform (`voksa_core::prosody::apply_prosody` — stressed-span stretch 1.5× → additive declination 120→95 Hz → +20 Hz/×1.2 in-span → optional xu +25 Hz rise; constants in docs/phonology.md §9.1), `render_utterance_prosodic`, and the real `cargo xtask listening-battery` all shipped. F0 measurement in voksa-testkit is a hand-rolled NSDF (pitch-detection 0.3 formant-locks; rejected by the smoke gate). CP1 verdict: ABX favored the FLAT baseline 8/10 because the whole-span stress stretch lengthened onset clusters — FIXED in Phase 7.1 (nucleus-scoped stretch: `SyllableSpan.nucleus_off_ms`, rhyme-only stretch window; onsets keep unit rate). Remaining CP1 backlog: items 7–8 (segment tuning, oracle comparability) + rules-only naturalness levers 1–5, all in PLAN.md "Naturalness backlog".

Phase 8 complete and tagged: the native CLI (`voksa-cli`). `voksa [FLAGS] <text>` plays live via cpal (whole utterance rendered up front → rtrb SPSC ring → callback only pops + zero-fills, no alloc); `voksa --out FILE <text>` renders a 48 kHz mono WAV without touching an audio device (CI-safe). Flags: `--flat --xu --dotside --buffer`. Hand-rolled RIFF writer keeps hound dev-only; the no-alloc callback path is proven by a hand-rolled counting global allocator (assert_no_alloc is BSD-1-Clause, banned).

Phase 9 complete and tagged: the browser adapter (`voksa-web`). It exposes a **raw C-ABI** (no wasm-bindgen runtime → the `.wasm` declares ZERO imports), so the hand-written AudioWorklet (`www/voksa-processor.js`) instantiates the Module — compiled on the main thread, passed via `processorOptions` — with `new WebAssembly.Instance(module, {})`, synthesizes the whole utterance once (`voksa_render`), copies the PCM out of wasm memory, and plays it 128 frames at a time (alloc-free `process()`). `www/index.html` is the demo (text + flag checkboxes). `-C target-feature=+simd128` (via .cargo/config.toml) with `--enable-simd` in the wasm-opt flags; `cargo xtask wasm-size` is real now (gzips the `.wasm`, asserts < 43 KB — currently ~33 KB — and asserts zero imports). Browser test (`wasm-pack test --headless --chrome`) is ADVISORY per the CLAUDE.md decision gate (the runner↔chromedriver handshake is flaky in WSL; identical `synth` logic is covered by required native unit tests + the import-free/size gate).

Demo tuning console (interlude between Phases 9 and 10; two parts, Basic tagged `demo-basic`, Advanced = `demo-advanced` TODO). Part 1 (Basic) made the 6 prosody knobs + a global `rate` RUNTIME params on `ProsodyOptions` (defaulted to the pinned constants, so all snapshots stay byte-identical — a `default_options_equal_pinned_constants` test guards it). They cross into the browser as an f32 block via `voksa_render_params` (wasm still import-free, ~33 KB gzip — serde never enters the wasm); the demo (`www/index.html`) is a tabbed console with sliders/presets/reset, Download+Load config JSON, Download WAV (JS RIFF encoder), a waveform, and a notes field. The worklet postMessages the PCM back to the main thread. Round-trip: `voksa --config <file.json>` (voksa-cli gained serde/serde_json, native-only) replays a submitted config exactly. Part 2a (tagged `demo-attitudinal`) put the 7 attitudinal deviation vectors on the Advanced tab: a runtime `AttitudinalTable` (`Default` = pinned, guarded; `apply_attitudinal_with`; `render_utterance_expressive`), the f32 block grows 7 → 63 (7 kinds × 8 fields in `AttitudinalKind::ALL` × `Deviation::to_array` order — JS mirrors it verbatim; 7-float/empty blocks stay valid), CLI config gains a delta-only `attitudinals` JSON block (per-cmavo keys, missing = pinned), and the UI is 7 Lojban-labeled panels (`.ui` + English gloss, 8 sliders, try-it examples). Loaded out-of-range values WIDEN slider bounds (never clamp) so hand-edited configs replay/re-export bit-identically to the CLI. Part 2b (COMMITTED, tag `demo-advanced` pending final review): the FULL per-phoneme `VoiceTable` — every independent segmental parameter runtime (377 floats: vowels/fricatives/nasals/liquids as `SteadyVoice`, stops as `StopVoice`, per-diphthong + [h] + buffer durations; diphthong glide endpoints DERIVE from the tuned vowels). `data::` seeds are `const fn` and `VoiceTable::PINNED` is built FROM them (one copy of the numbers → default byte-identity by construction; `spec`/`buffer_spec`/`compile` are thin delegates over `spec_with`/`buffer_spec_with`/`compile_with`; durations clamp `.max(0.0)`). f32 block = 440 (`FULL_PARAM_COUNT`; 7-/63-float blocks stay valid); CLI config gains a `phonemes` delta-merge block (per-letter keys, nested `closure`/`burst` for stops). The demo seeds ALL slider defaults from the wasm itself (`voksa_default_params` export, read by a main-thread instance) so UI defaults cannot drift from the engine; the Advanced tab has six per-phoneme panel sections, and a **phonetic sentence picker** (`www/sentences.json`, 18 curated coverage sentences + ⏭ cycle button) is gated by a native test that synthesizes every entry (`crates/voksa-web/tests/sentences.rs`; serde_json is a voksa-web DEV-dep only).

Phase 10 COMMITTED, awaiting CP2 (human tags after listening — do NOT tag): the attitudinal layer. The engine is now a VENDORED FORK — `crates/klattsch-core-fork/` (klattsch-core 0.1.1 verbatim, MIT; the workspace dep is a `path`) with two default-preserving glottal lanes added: **OQ** (open quotient, scales the Rosenberg open phase `Tp`; OQ=1.0 byte-identical to upstream, guarded) and **DI** (diplophonia, dips the excitation on odd glottal cycles → F0/2 subharmonic). `Frame` gained `oq/tilt/di/vibrato_hz` voice-quality lanes (`Frame::modal` + `NEUTRAL_*` + guard). New `voksa_core::attitudinal`: 7 INVENTED/non-normative categories (`.ui`→Joy `.uu`→Sadness `.oi`→Complaint `.ii`→Fear `.o'o`→Patience `.au`→Desire `.o'onai`→Anger) each a deviation vector (additive **Hz**, no semitones — core is no_std), `cai`/`sai`/`ru'e`/`nai` intensity (nai = −1 flip), scope = nearest preceding non-marker word; `compile` detects → `UtteranceSchedule.attitudinals`; `apply_attitudinal` overlays on prosody (docs/phonology.md §10). The adapter Option-gates the VQ lanes against the PREVIOUS frame → modal utterances lower byte-identically (engine snapshots unchanged) and colored words reset on exit (no bleed); `render_utterance_prosodic` runs the overlay so any UI cmavo speaks with affect on native, CLI, AND browser (no C-ABI change). testkit += `measure_spectral_tilt`/`measure_f0_variance`/`measure_diplophonia`. CP2 battery: `cargo xtask attitudinal-battery` → 7 items × 3 (affect / neutral-base / eSpeak oracle) in artifacts/listening/phase10/ (gitignored) + scoring index.html; template docs/listening/phase10.md. 257 tests; wasm 34965 B gzip, zero imports. Next after CP2: Phase 11 — polish (CP3). Engine: klattsch-core fork (ADR 0001); sample rate 48 000 Hz. See PLAN.md for live phase status.

Environment: the repo lives in WSL Ubuntu at `/home/dhilipsiva/projects/dhilipsiva/voksa`; all Rust/nix commands run inside `nix develop` there. From a Windows-side session, wrap every command as
`wsl.exe -d Ubuntu --cd /home/dhilipsiva/projects/dhilipsiva/voksa -- bash -lc "nix develop --command <cmd>"`
using the PowerShell tool (Git Bash mangles `/home/...` args via MSYS path conversion).

## Authoritative documents (read before designing anything)

- docs/research/01-architecture-v1.md — first-round architecture research
- docs/research/02-architecture-v2.md — VERIFIED v2; supersedes v1 on conflicts
- docs/research/03-implementation-playbook.md — the 12-phase plan this repo follows
- docs/phonology.md — CLL-derived rules (single source of truth)
- docs/formants.md — formant seeds + consonant loci; tests assert against this table
- PLAN.md — live phase table + status. UPDATE AFTER EVERY MILESTONE.
- docs/chatgpt-report.md, docs/gemini-report.md — supplementary first-round research inputs; v2 corrects known errors in them (e.g. Gemini's "21 consonants" — the correct count is 17)

Precedence on any conflict: v2 report > v1 report > other reports. CLL (Complete Lojban Language) spec > any report; docs/phonology.md is the CLL distillation to cite when implementing.

## Commands (use these exact invocations, inside `nix develop`)

- Test: `cargo nextest run --workspace`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Format: `cargo fmt --all` (check: `--check`)
- Snapshots: `cargo insta review`
- WASM build: `wasm-pack build --release --target web crates/voksa-web`
- WASM size gate: `cargo xtask wasm-size` (builds, gzips, asserts < 43 KB + zero imports)
- Browser test: `wasm-pack test --headless --chrome crates/voksa-web` (advisory; needs chromium+chromedriver — in the flake)
- Web demo: `wasm-pack build --release --target web crates/voksa-web`, then serve `crates/voksa-web/` over http (e.g. `python3 -m http.server`) and open `www/index.html` (not `file://`)
- eSpeak oracle: `cargo xtask oracle -- <lojban text>` → fixtures/oracle/<slug>.wav
- Listening battery: `cargo xtask listening-battery` (CP1, Phase 7)
- Attitudinal battery: `cargo xtask attitudinal-battery` (CP2, Phase 10 → artifacts/listening/phase10/index.html)
- CLI play: `cargo run -p voksa-cli -- <lojban text>` (needs an audio device)
- CLI render: `cargo run -p voksa-cli -- --out out.wav <lojban text>` (flags: `--flat --xu --dotside --buffer`)
- CLI replay a tuning config: `cargo run -p voksa-cli -- --config cfg.json --out out.wav` (JSON from the web demo)
- Dev shell: `nix develop`

## Workspace map

- crates/voksa-core — no_std + alloc. Front-end + schedule compiler + DSP.
- crates/voksa-cli — native binary: cpal playback + offline WAV render.
- crates/voksa-web — wasm-bindgen + AudioWorklet glue + demo page.
- crates/voksa-engine-klattsch — std adapter; the ONLY crate allowed to depend on klattsch-core; swappable at the Phase-2 engine gate.
- crates/klattsch-core-fork — VENDORED fork of klattsch-core 0.1.1 (MIT, Tony Gies); the workspace `klattsch-core` dep resolves here (Phase-10 OQ + diplophonia). Keep default output byte-identical to upstream.
- crates/voksa-testkit — dev-only FFT formant harness + WAV helpers; only ever referenced from [dev-dependencies].
- xtask/ — automation: wasm-opt, size gate, oracle, listening battery.
- tests/ — integration tests + insta schedule snapshots.
- fixtures/oracle/ — eSpeak-NG jbo WAVs — REGRESSION ORACLE ONLY. Never copy its data.
- docs/ — research reports, ADRs, phonology notes, listening logs.

## Architecture (settled — do not re-research; details in docs/)

- Engine: klattsch-core (MIT, pinned =0.1.1) parallel-formant synth — KEPT at the Phase-2 gate (docs/adr/0001-engine-choice.md). voksa-core owns the phoneme IR (`phonemes::SegmentSpec`; klattsch's PhonemeTable trait is lossy and deliberately not implemented); voksa-engine-klattsch lowers IR → schedules with Klatt-1980 alternating A2 polarity and gain 1.0 (linear range). Phase 10 (OQ/diplophonia): vendored fork of the glottal source.
- Front-end from the CLL specification ONLY: 17 consonants, 6 vowels (a e i o u y=[ə]), 16 diphthongs, NO triphthongs. Apostrophe=[h], period=pause/glottal stop, comma=syllable separator (never a pause).
- Syllabification per CLL §3.9: single C → following vowel; CC split unless valid initial pair (48 pairs); CCC split after first C.
- Stress: penultimate over COUNTABLE syllables. Countable excludes: y-syllables, syllabic-consonant syllables (l m n r as nucleus), buffer-vowel syllables.
- Word classifier: cmevla = ends in consonant; ends in y = cmavo; brivla = a consonant pair (ANY pair, permissibility not required — CLL §4.3) in the first five letters counted after deleting y and apostrophe + ends in vowel; cmavo = otherwise.
- Mandatory pauses: before vowel-initial words; after consonant-final words (all cmevla); BEFORE every cmevla unless preceded by la/lai/la'i/doi (CLL §4.9 r4); around zoi/la'o foreign text; after y-final cmavo unless another follows (generalizes Cy, §17.2); stressed-final word before brivla or before stressed-first word; before AND after hesitation .y.; boundaries merge to one pause. Flags: `--dotside` (drop the la-family exemption → leading pause before every cmevla), `--buffer` (short [ɪ] epenthesis, excluded from stress counting; default OFF).
- Normalization: digits → PA cmavo as separate words (no pa re ci vo mu xa ze bi so; pi decimal; ki'o thousands emitted as FULL 3-digit groups; pi'e for ":"; hex dau…vai = vocabulary only, not auto-detected). Lerfu: by cy dy…, .abu…, .y'y; h=.y'y.bu q=ky.bu w=vy.bu; denpa/slaka bu; case-insensitive, no shifts.
- Prosody: declination 120→95 Hz; stress = ~1.5× duration + F0 excursion (+10–30 Hz) + amplitude boost; optional xu terminal rise.
- Attitudinal overlay (UI cmavo): F0 + voice-quality per docs/research/02-architecture-v2.md §11 table (.ui joy, .uu sadness, .oi creak/diplophonia, .ii vibrato/flutter, .o'o monotone; intensity via cai/sai/ru'e/nai). Params: F0 AV AH OQ TL FL DI.

## Rules for working here

- TDD ALWAYS: write failing tests first, run them, confirm red, COMMIT the failing tests, then implement until green. NEVER modify tests to make them pass.
- The schedule compiler MUST be deterministic. Snapshot schedules with insta. NEVER bit-compare rendered WAVs (SIMD/platform float nondeterminism). Acoustic assertions use tolerance bands (formants ±5% or ±50 Hz for F1; ±10% in CI).
- Dependency licenses: MIT/Apache only. hound in dev-dependencies only.
- No SharedArrayBuffer, no COOP/COEP, no nightly Rust, no cpal-on-wasm. Web audio = hand-written AudioWorkletProcessor; WASM Module compiled on main thread, instantiated inside the worklet scope via processorOptions.
- WASM: `-C target-feature=+simd128`; wasm-opt -Oz; size budget enforced by xtask.
- No heap allocation in the audio callback / process() hot path.
- eSpeak NG (GPLv3) is an OUT-OF-PROCESS oracle only. Never copy its code, phoneme tables, or data files. All linguistic rules come from the CLL.
- Phases are sequential (see PLAN.md); do not start phase N+1 before N is tagged. main is always green.
- After completing a milestone: update PLAN.md (status, date, SHA, deviations), run the full verify battery, commit (Conventional Commits), tag phaseN-complete.
- Commits auto-push to origin (PostToolUse hook). Never force-push.
- Listening checkpoints (Phases 7, 10, 11): render the battery, then STOP for human review. The human tags the milestone, not you.
- When compacting: preserve current phase, modified files, test status, open TODOs.
