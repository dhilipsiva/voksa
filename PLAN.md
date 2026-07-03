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
| 1 | Engine spike: klattsch-core integration, hardcoded vowel → WAV, FFT formant harness | [x] | phase1-complete | — | 2026-07-03 / f9e53cf | 2 new crates (voksa-engine-klattsch std adapter + voksa-testkit dev harness) — klattsch-core is std-only, cannot live in no_std core; klattsch pinned =0.1.1; project sample rate 48 kHz; PhonemeTable trait is std-side → gate input: own IR in core, adapter lowers |
| 2 | Lojban phoneme table: 17 C + 6 V + 16 diphthongs, LojbanTable: PhonemeTable | [x] | phase2-complete | — | 2026-07-03 / ca86dac | Own IR in voksa-core instead of klattsch's lossy PhonemeTable trait; loqa-voice-dsp broken → hand-rolled LPC in testkit; Klatt alternating A2 polarity + gain 1.0 in lowering; measurement F0 105 grid + whitened harmonic FFT interpolation; GATE: klattsch KEPT (ADR 0001), Phase-10 = vendored fork for OQ/DI |
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
- 2026-07-02 — policy — Auto-push enabled at owner request: PostToolUse hook pushes `origin HEAD --follow-tags` after every git commit/tag. Guard hook now blocks only force pushes and stray rm -rf (push-to-main block removed).
- 2026-07-03 — Phase 1 — 1 session — Engine spike green: klattsch-core 0.1.1 renders steady /a/ measured at F1 719.6 / F2 1079.6 / F3 2519.9 Hz vs targets 730/1090/2440 (1.4% / 1.0% / 3.3% error — inside ±10% gate, near ±5% Phase-2 band). FFT harness (voksa-testkit, spectrum-analyzer 1.7 + hound dev-only) self-tested on synthetic tones. Red-first honored: acceptance test committed failing against a silent skeleton, then implementation. Gate input for Phase 2: klattsch PhonemeTable trait is std-side — plan is own schedule IR in voksa-core, adapter lowers to klattsch; OQ/diplophonia params still absent from klattsch (known, decides at gate). 13/13 tests, clippy/fmt clean, a.wav rendered.
- 2026-07-03 — Phase 2 — 1 session — Full inventory green, 47/47 tests. All 6 vowels within ±5% (F1 floor 50 Hz) by BOTH FFT and LPC (e.g. /a/ LPC 725/1081/2432 vs 730/1090/2440); sibilant centroids s 6363 / z 6436 / c 3425 / j 3291 / x 2458 Hz all in-band; /ai/ F2 glide monotonic 1080→2280. Debug journey worth remembering: (1) klattsch default gain 3.5 soft-clips → corrupted LPC + intermodulation ghosts — adapter renders at gain 1.0; (2) loqa-voice-dsp 0.5 empirically broken (garbage on synthetic vowels) — LPC hand-rolled in testkit (Levinson-Durbin + Durand-Kerner); (3) parallel-topology spectral zeros biased LPC F2 +100–170 Hz — fixed by Klatt-1980 alternating A2 polarity in the lowering (errors → single-digit Hz); (4) FFT harmonic-grid quantization — measurement F0 105 Hz + source-whitened log-parabolic interpolation across ±F0 neighbours. ENGINE GATE: klattsch-core KEPT (ADR 0001); Phase-10 OQ/DI via vendored fork of its glottal source.
