# voksa — Phase Plan and Live Status

Source: docs/research/03-implementation-playbook.md (full acceptance criteria there).
This file is the LIVE status tracker. Claude Code updates it after every milestone.
Rule: main is always green. Phases are sequential; do not start N+1 before N is tagged.

## Status legend
- [ ] not started   [~] in progress   [x] done (tagged)

## Phases

| # | Phase | Status | Tag | Listening CP | Completed (date / SHA) | Deviations |
|---|-------|--------|-----|--------------|------------------------|------------|
| 0 | Repo scaffolding: workspace, Nix flake, CI, hooks, verifier subagent, eSpeak oracle xtask | [x] | phase0-complete | — | 2026-07-02 / 6256d1e | flake: crane check + lld override dropped (stale sketch); voksa-web wasm-opt flags (-Oz + bulk-memory/nontrapping-fptoint/sign-ext); fmt hook = fmt --all; oracle WAVs gitignored; CI validated by inspection (not yet pushed) |
| 1 | Engine spike: klattsch-core integration, hardcoded vowel → WAV, FFT formant harness | [ ] | phase1-complete | — | | |
| 2 | Lojban phoneme table: 17 C + 6 V + 16 diphthongs, LojbanTable: PhonemeTable | [ ] | phase2-complete | — | | |
| 3 | Syllabifier + word classifier (pure fns, proptest, CLL vectors) | [ ] | phase3-complete | — | | |
| 4 | Stress + pause insertion (penultimate w/ exclusions; mandatory + --dotside) | [ ] | phase4-complete | — | | |
| 5 | Schedule compiler: text → deterministic param schedule; --buffer flag | [ ] | phase5-complete | — | | |
| 6 | Number/lerfu normalization (PA cmavo, pi, ki'o, hex; by/.abu/.y'y) | [ ] | phase6-complete | — | | |
| 7 | Prosody layer: declination, stress realization, xu rise | [ ] | phase7-complete | **CP1** | | |
| 8 | Native CLI adapter: cpal + rtrb, offline WAV mode | [ ] | phase8-complete | — | | |
| 9 | Web adapter: wasm-bindgen, AudioWorklet, demo page, size budget | [ ] | phase9-complete | — | | |
| 10 | Attitudinal layer: F0/voice-quality overlay, OQ/DI, cai/sai/ru'e/nai | [ ] | phase10-complete | **CP2** | | |
| 11 | Polish: docs, optional WIT component, fuzzing, final battery | [ ] | v0.1.0 | **CP3** | | |

## Decision gates
- End of Phase 2: klattsch-core sufficient? (formant tolerances hit; OQ/diplophonia
  path feasible). If NO → switch to hand-rolled cascade/parallel engine or fundsp;
  record as ADR in docs/adr/.
- Phase 9: WASM size budget blown? → twiggy profile, panic=abort, dep audit
  BEFORE adding features.
- Any phase: browser CI flaky? → pin Chrome in the Nix flake; browser test becomes
  advisory, native tests remain required.

## Session log
(append: date, phase, sessions used, notes)

- 2026-07-02 — Phase 0 — 1 session — Workspace (4 crates), flake (fenix stable 1.96.1 + wasm32 std, espeak-ng), CI, portable Windows/WSL hooks, verifier subagent, skills, working oracle (coi-munje.wav RIFF-valid, 58690 B). TDD smoke: red (wrong assertion) → Stop hook blocked exit 2 → fix → 7/7 green → Stop hook passed. Found+fixed: wasm-pack 0.15 silently swallows wasm-opt failures (binaryen 129 rejects rustc's default bulk-memory ops) via per-crate wasm-opt feature flags; wasm 17171 → 15923 B. Fresh-context verifier: all criteria PASS.
