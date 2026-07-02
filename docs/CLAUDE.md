# voksa — Rule-based Lojban Klatt-style TTS (Rust → WASM + native)

## What this is
A pure-Rust, rule-based parallel-formant (Klatt-style) speech synthesizer for
Lojban (jbo). No ML, no eSpeak data, no training corpus. Pipeline: text →
deterministic parameter schedule → PCM. Targets: browser (WASM, single-threaded
AudioWorklet) and native (cpal). Metric units throughout (Hz, ms, semitones).

`voksa` = Lojban gismu: x1 is the voice/speech sounds of x2.

## Authoritative documents (read before designing anything)
- docs/research/01-architecture-v1.md   — first-round architecture research
- docs/research/02-architecture-v2.md   — VERIFIED v2; supersedes v1 on conflicts
- docs/research/03-implementation-playbook.md — the 12-phase plan this repo follows
- docs/phonology.md                     — CLL-derived rules (single source of truth)
- docs/formants.md                      — formant seeds + consonant loci
- PLAN.md                               — live phase table + status. UPDATE AFTER EVERY MILESTONE.

On any conflict: v2 report > v1 report. CLL spec > any report.

## Workspace map
- crates/voksa-core   # no_std + alloc. Front-end + schedule compiler + DSP.
- crates/voksa-cli    # native binary: cpal playback + offline WAV render.
- crates/voksa-web    # wasm-bindgen + AudioWorklet glue + demo page.
- xtask/              # automation: wasm-opt, size gate, oracle, listening battery.
- tests/              # integration tests + insta schedule snapshots.
- fixtures/oracle/    # eSpeak-NG jbo WAVs — REGRESSION ORACLE ONLY. Never copy its data.
- docs/               # research reports, ADRs, phonology notes, listening logs.

## Commands (use these exact invocations)
- Test:         cargo nextest run --workspace
- Lint:         cargo clippy --workspace --all-targets -- -D warnings
- Format:       cargo fmt --all
- Snapshots:    cargo insta review
- WASM build:   cd crates/voksa-web && wasm-pack build --release --target web
- WASM size:    cargo xtask wasm-size
- Browser test: wasm-pack test --headless --chrome crates/voksa-web
- Oracle:       cargo xtask oracle -- <lojban text>
- Battery:      cargo xtask listening-battery
- Dev shell:    nix develop

## Architecture (settled — do not re-research; details in docs/)
- Engine: klattsch-core (MIT) parallel-formant synth; implement LojbanTable: PhonemeTable.
  Fallback (decide by end of Phase 2): hand-rolled cascade/parallel or fundsp graphs.
- Front-end from the CLL specification ONLY: 17 consonants, 6 vowels
  (a e i o u y=[ə]), 16 diphthongs, NO triphthongs. Apostrophe=[h], period=pause/
  glottal stop, comma=syllable separator (never a pause).
- Syllabification per CLL §3.9: single C → following vowel; CC split unless valid
  initial pair (48 pairs, see docs/phonology.md); CCC split after first C.
- Stress: penultimate over COUNTABLE syllables. Countable excludes: y-syllables,
  syllabic-consonant syllables (l m n r as nucleus), buffer-vowel syllables.
- Word classifier: cmevla = ends in consonant; brivla = consonant cluster in first
  five letters (ignoring y and apostrophe) + ends in vowel; cmavo = otherwise.
- Mandatory pauses: before vowel-initial words; after consonant-final words (all
  cmevla); around zoi/la'o foreign text; after Cy cmavo; stressed-final-cmavo
  before brivla. Flags: --dotside (leading pause before every cmevla), --buffer
  (short [ɪ] epenthesis, excluded from stress counting; default OFF).
- Normalization: digits → PA cmavo (no pa re ci vo mu xa ze bi so; pi decimal;
  ki'o thousands; dau fei gai jau rei vai hex). Lerfu: by cy dy…, .abu…, .y'y.
- Prosody: declination 120→95 Hz; stress = ~1.5× duration + F0 excursion
  (+10–30 Hz) + amplitude boost; optional xu terminal rise.
- Attitudinal overlay (UI cmavo): F0 + voice-quality per docs/research/02 §11
  table (.ui joy, .uu sadness, .oi creak/diplophonia, .ii vibrato/flutter,
  .o'o monotone; intensity via cai/sai/ru'e/nai). Params: F0 AV AH OQ TL FL DI.

## Rules for working here
- TDD ALWAYS: write failing tests first, run them, confirm red, COMMIT the failing
  tests, then implement until green. NEVER modify tests to make them pass.
- The schedule compiler MUST be deterministic. Snapshot schedules with insta.
  NEVER bit-compare rendered WAVs (SIMD/platform float nondeterminism). Acoustic
  assertions use tolerance bands (formants ±5% or ±50 Hz for F1; ±10% in CI).
- Dependency licenses: MIT/Apache only. hound in dev-dependencies only.
- No SharedArrayBuffer, no COOP/COEP, no nightly Rust, no cpal-on-wasm.
  Web audio = hand-written AudioWorkletProcessor; WASM Module compiled on main
  thread, instantiated inside the worklet scope via processorOptions.
- WASM: -C target-feature=+simd128; wasm-opt -Oz; size budget enforced by xtask.
- No heap allocation in the audio callback / process() hot path.
- eSpeak NG (GPLv3) is an OUT-OF-PROCESS oracle only. Never copy its code,
  phoneme tables, or data files. All linguistic rules come from the CLL.
- After completing a milestone: update PLAN.md (status, date, SHA, deviations),
  run the full verify battery, commit (Conventional Commits), tag phaseN-complete.
- Listening checkpoints (Phases 7, 10, 11): render the battery, then STOP for
  human review. The human tags the milestone, not you.
- When compacting: preserve current phase, modified files, test status, open TODOs.
