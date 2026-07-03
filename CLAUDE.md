# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`voksa` — a pure-Rust, rule-based parallel-formant (Klatt-style) speech synthesizer for Lojban (jbo). No ML, no eSpeak data, no training corpus. Pipeline: text → deterministic parameter schedule → PCM. Targets: browser (WASM, single-threaded AudioWorklet) and native (cpal). Metric units throughout (Hz, ms, semitones).

`voksa` = Lojban gismu: x1 is the voice/speech sounds of x2.

## Current state

Phase 7 complete and tagged (CP1 signed off 2026-07-03; scores in docs/listening/phase7.md). The prosody transform (`voksa_core::prosody::apply_prosody` — stressed-span stretch 1.5× → additive declination 120→95 Hz → +20 Hz/×1.2 in-span → optional xu +25 Hz rise; constants in docs/phonology.md §9.1), `render_utterance_prosodic`, and the real `cargo xtask listening-battery` all shipped. F0 measurement in voksa-testkit is a hand-rolled NSDF (pitch-detection 0.3 formant-locks; rejected by the smoke gate). CP1 verdict: ABX favored the FLAT baseline 8/10 because the whole-span stress stretch lengthened onset clusters — FIXED in Phase 7.1 (nucleus-scoped stretch: `SyllableSpan.nucleus_off_ms`, rhyme-only stretch window; onsets keep unit rate). Remaining CP1 backlog: items 7–8 (segment tuning, oracle comparability) + rules-only naturalness levers 1–5, all in PLAN.md "Naturalness backlog".

Phase 8 complete and tagged: the native CLI (`voksa-cli`). `voksa [FLAGS] <text>` plays live via cpal (whole utterance rendered up front → rtrb SPSC ring → callback only pops + zero-fills, no alloc); `voksa --out FILE <text>` renders a 48 kHz mono WAV without touching an audio device (CI-safe). Flags: `--flat --xu --dotside --buffer`. Hand-rolled RIFF writer keeps hound dev-only; the no-alloc callback path is proven by a hand-rolled counting global allocator (assert_no_alloc is BSD-1-Clause, banned). Next: Phase 9 — web adapter (wasm-bindgen, AudioWorklet, demo page, size budget). Engine: klattsch-core (ADR 0001); sample rate 48 000 Hz. See PLAN.md for live phase status.

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
- WASM size gate: `cargo xtask wasm-size` (stub until Phase 9)
- Browser test: `wasm-pack test --headless --chrome crates/voksa-web` (from Phase 9)
- eSpeak oracle: `cargo xtask oracle -- <lojban text>` → fixtures/oracle/<slug>.wav
- Listening battery: `cargo xtask listening-battery` (stub until Phase 7)
- CLI play: `cargo run -p voksa-cli -- <lojban text>` (needs an audio device)
- CLI render: `cargo run -p voksa-cli -- --out out.wav <lojban text>` (flags: `--flat --xu --dotside --buffer`)
- Dev shell: `nix develop`

## Workspace map

- crates/voksa-core — no_std + alloc. Front-end + schedule compiler + DSP.
- crates/voksa-cli — native binary: cpal playback + offline WAV render.
- crates/voksa-web — wasm-bindgen + AudioWorklet glue + demo page.
- crates/voksa-engine-klattsch — std adapter; the ONLY crate allowed to depend on klattsch-core (pinned =0.1.1); swappable at the Phase-2 engine gate.
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
