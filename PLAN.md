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
| 3 | Syllabifier + word classifier (pure fns, proptest, CLL vectors) | [x] | phase3-complete | — | 2026-07-03 / ae01af9 | CLL-text corrections folded into phonology.md: forbidden triples ndj/ndz/ntc/nts added; classifier = ANY pair (not permissible; CLL §4.3 bisycla) + ends-in-y→cmavo; deterministic syllabic-sonorant rule (vowel-less regions only) + onset maximization documented as the chosen CLL-valid variants |
| 4 | Stress + pause insertion (penultimate w/ exclusions; mandatory + --dotside) | [x] | phase4-complete | — | 2026-07-03 / 335975f | phonology.md gained two MISSING mandatory rules (CLL §4.9 r4 pause-before-cmevla w/ la-family exemption; §4.2 .y. double pause) + y-final generalization (§17.2) + §4.2 stressed+stressed rule; iy/uy ruled uncountable; capitals map char→syllable, brivla misplacement = error |
| 5 | Schedule compiler: text → deterministic param schedule; --buffer flag | [x] | phase5-complete | — | 2026-07-03 / d1bc60f | Engine-neutral IR realized in core (schedule.rs: Frame limited to current engine vocabulary — OQ/TL/FL/DI deferred to Phase-10 fork); adapter reduced to 1:1 lowering (gain + A2-polarity quirks isolated); Phase-2 snapshots byte-identical through the refactor; buffering = fully-buffered dialect minus trailing buffer; every written period honored as pause; syllabic nuclei reuse consonant steady spec (revisit at CP1) |
| 6 | Number/lerfu normalization (PA cmavo, pi, ki'o, hex; by/.abu/.y'y) | [x] | phase6-complete | — | 2026-07-03 / 48c3e15 | ki'o emitted as FULL 3-digit groups (elision never emitted); hex = vocabulary only, not auto-detected (letters in figures = typed error); ":"→pi'e; h/q/w lerfu resolved per CLL §17.5 (.y'y.bu / ky.bu / vy.bu); lerfu written dots reproduced by existing pause rules — zero new pause code |
| 7 | Prosody layer: declination, stress realization, xu rise | [x] | phase7-complete | **CP1** | 2026-07-03 / 4a40e53 | pitch-detection 0.3 REJECTED by smoke gate (formant-locks ~490 Hz on the buzzy source; clarity/power gates reject everything at recommended settings) → hand-rolled NSDF in testkit (lag range 70–200 Hz, 0.8 peak gate, parabolic refinement, 5-pt median); xu rise also raises later in-span events (a following vowel event re-set F0 down); acoustic RMS check = peak 30 ms window (stop closures stretch); human tags after listening |
| 8 | Native CLI adapter: cpal + rtrb, offline WAV mode | [x] | phase8-complete | — | 2026-07-03 / b800d85 | cpal 0.18 (Apache-2.0) + rtrb 0.3; whole-utterance render → SPSC ring → callback only pops + zero-fills (no alloc, verified by a hand-rolled counting allocator since assert_no_alloc is BSD-1-Clause); hand-rolled RIFF writer (hound stays dev-only); render at the negotiated device rate (no resampling); flake gains alsa-lib + pkg-config + LD_LIBRARY_PATH, CI gains libasound2-dev; playback degrades gracefully to a WAV-render hint when headless |
| 9 | Web adapter: wasm-bindgen, AudioWorklet, demo page, size budget | [x] | phase9-complete | — | 2026-07-03 / 1e00312 | Option B (wasm instantiated IN the worklet, per CLAUDE.md). voksa-web dropped the wasm-bindgen RUNTIME dep for a raw C-ABI (voksa_alloc/dealloc/render/out_len/free_f32) → the `.wasm` declares ZERO imports, so the worklet uses `new WebAssembly.Instance(module, {})`; wasm-bindgen-test stays a dev-dep only. simd128 via .cargo/config.toml + `--enable-simd` wasm-opt flag (else silent wasm-opt skip). `cargo xtask wasm-size` gzips (~33 KB < 43 KB budget) + asserts zero imports (wasm-dis, best-effort). chromium+chromedriver in the flake; the headless-chrome test is ADVISORY (continue-on-error) — the wasm-bindgen-test-runner↔chromedriver handshake resets in WSL though the driver works standalone; identical synth logic is covered by required native unit tests |
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

## Naturalness backlog (rules-only; owner-approved 2026-07-03)

Clarified intent: "robotic" in the original brief meant MECHANICAL AND
ALGORITHM-DRIVEN (deterministic, no ML, no recorded data) — NOT "must sound
like a robot". The v1 monotone baseline is a deliberate starting point, not
the target. Ceiling for rules-only ≈ high-end 1990s formant TTS (DECtalk
class); neural-TTS naturalness is out of scope by design. Levers, in impact
order:

1. **Voice source realism** — LF-model glottal pulse, open quotient (OQ),
   spectral tilt (TL), aspiration noise in vowels (AH, breathiness). Lands in
   **Phase 10** (vendored klattsch glottal fork): use these for the DEFAULT
   voice baseline, not only the attitudinal overlay.
2. **F0 flutter/jitter** — Klatt FL-style slow quasi-random F0 wobble via a
   SEEDED PRNG (deterministic ≠ constant; schedule stays reproducible and
   snapshot-testable). Baseline-on in **Phase 10** (FL is already in the
   overlay param set).
3. **Microprosody** — consonant-driven F0 perturbations (dips around voiced,
   rises after voiceless obstruents), intrinsic vowel F0/duration. NEW work —
   schedule-compiler transform, post-Phase-10 (fold into Phase 11 or a new
   phase at owner's call).
4. **Duration rules** — phrase-final lengthening, cluster shortening (Klatt
   1976 duration model). NEW work, same placement as (3).
5. **Deeper coarticulation** — duration-dependent formant target undershoot
   on top of the existing consonant loci. NEW work, same placement as (3).

All five stay inside the project constraints: pure Rust, deterministic
pipeline, CLL-only linguistics, no training data.

### CP1 findings (Phase 7 listening, 2026-07-03) → queued fixes

Rater: dhilipsiva. Caveat recorded in docs/listening/phase7.md: self-taught
Lojban (never heard another speaker), scored with eSpeak as the reference.
MOS intelligibility avg 2.6, naturalness avg 2.3; ABX: flat preferred 8/10.

6. ✅ **DONE (Phase 7.1, 2026-07-03)** — **Nucleus-scoped stress stretch.**
   The whole stressed span (incl. onset cluster transitions) stretched ×1.5 →
   "CC strong + long" on gismu-initial stressed syllables (pre/kla/zga/dja/
   DJO), which explained the 8/10 flat ABX preference. FIXED: `SyllableSpan`
   now carries `nucleus_off_ms`; the stretch window opens at the nucleus so
   only the rhyme stretches — onsets keep unit rate. Excursion + amplitude
   stay whole-span. Snapshots regenerated, battery re-rendered; awaiting a
   quick owner ABX re-check (the fix is committed, NOT re-tagged — phase7
   stays the tag).
7. **Segment tuning** — coi-munje heard as "soi-oon-shae" (MOS 1/1): /c/ [ʃ]
   not distinct enough from [s], nasal /m/ murmur weak, /j/ [ʒ] reads
   devoiced. Phoneme-table work (docs/formants.md), independent of prosody.
8. **Oracle comparability** — eSpeak jbo found WRONG twice: xu rendered
   [k]-like (CLL x = [x], German Bach — voksa judged better by the rater)
   and raw-digit reading of "4" in "li 3.14" (voksa's CLL §18 expansion to
   "vo" is correct). Battery tweak: also feed eSpeak the normalized
   PA-cmavo string for number utterances so the oracle column stays
   comparable; never treat eSpeak as ground truth (fixture policy already
   says regression-oracle-only).

## Session log
(append: date, phase, sessions used, notes)

- 2026-07-04 — Demo tuning console D1 (Basic) — interlude between Phases 9 and 10 (owner-requested crowdsourced tuning: fiddle → download JSON → share → replay). Made the 6 prosody knobs + a global `rate` runtime fields on `ProsodyOptions`, defaulted to the pinned constants so all 12 snapshots + acoustic tests stay byte-identical (guarded by `default_options_equal_pinned_constants`); `apply_prosody` reads the fields and adds a rate scale (1.0 = exact identity). Web: `voksa_render_params` C-ABI takes an f32 param block (JS writes it via `voksa_alloc`, like the text) → the wasm stays import-free at ~33 KB gzip (serde deliberately NOT in the wasm — it would blow the 43 KB gate). Demo `www/index.html`: tabbed console (Basic active, Advanced = D2 placeholder), 7 sliders + 4 flags + text driven by one descriptor list that IS the f32 layout + CLI key schema; presets, reset, Download/Load config JSON, Download WAV (JS RIFF encoder mirroring wav.rs), waveform canvas, notes field; worklet postMessages the PCM back to the main thread. Round-trip: `voksa --config <file.json>` (voksa-cli += serde/serde_json, native-only) maps text + flags + params onto the option structs and replays exactly. Also fixed the Phase-9 demo bug found on first listen: the worklet encoded text with `TextEncoder`, absent from Firefox's AudioWorkletGlobalScope → moved encoding to the main thread + `?v=` cache-bust. Red-first per chunk (core/web/cli). Tests: 198/198; wasm import-free + under budget. Tag `demo-basic`. D2 (Advanced tab: runtime `VoiceTable` for per-phoneme acoustics) still TODO.

- 2026-07-03 — Phase 9 — web adapter (Option B: wasm synthesized INSIDE the AudioWorklet, honoring the CLAUDE.md directive). Key finding: exposing the engine via a raw C-ABI (`#[unsafe(no_mangle)] extern "C"`, edition-2024 syntax) with NO wasm-bindgen runtime yields a `.wasm` with an EMPTY import section, so the worklet instantiates it synchronously with `new WebAssembly.Instance(module, {})` — no glue, no ESM-in-worklet problem. voksa-web now depends on voksa-engine-klattsch (verified wasm-compilable: klattsch-core has zero deps, no std::time/thread/random); wasm-bindgen-test is the only wasm-bindgen-family dep and it's dev-only (its `#[wasm_bindgen_test]` harness needs no `#[wasm_bindgen]` on the tested fn). Surface: `synth(text,flags,sr)->Result<Vec<f32>>` shared by 5 raw exports (alloc/dealloc/render/out_len/free_f32; length via an AtomicUsize register — clippy-clean vs `static mut`) + native unit tests. Worklet renders the whole utterance once in its constructor, copies PCM out of wasm memory (re-reading `.buffer` after each allocating call — grow detaches views), plays 128-frame chunks alloc-free. Demo: www/index.html (compileStreaming on main → processorOptions → AudioWorkletNode). Build plumbing: simd128 via .cargo/config.toml `[target.wasm32-unknown-unknown]` (target-scoped, native untouched) + `--enable-simd` in voksa-web's wasm-opt flags (without it wasm-pack 0.15 silently ships the unoptimized binary); `cargo xtask wasm-size` (real) gzips the wasm (33_091 B) vs a 43_000 B budget and asserts zero imports via wasm-dis (best-effort if binaryen absent); CI wasm-build now runs `cargo xtask wasm-size` (hard gate, + binaryen for wasm-dis). Browser test: chromium+chromedriver added to the flake; `wasm-pack test --headless --chrome` — chromedriver + chromium work standalone (manual W3C session succeeds) but the wasm-bindgen-test-runner gets "connection reset" from the driver it spawns in WSL (not IPv6 — all of 127.0.0.1/localhost/::1 reach it; wasm-bindgen 0.2.126 is current so not the old race). Per the CLAUDE.md gate the CI wasm-test job is `continue-on-error: true` (advisory); native voksa-web unit tests (identical `synth`) + the import-free/size gate are the required coverage. 187/187 workspace tests, wasm import-free at ~33 KB gzip. Tagged phase9-complete.

- 2026-07-03 — Phase 8 — native CLI. `voksa-cli` gains a lib (args/wav/playback) + thin bin. `voksa [FLAGS] <text>` plays; `--out FILE` renders a 48 kHz mono WAV (never touches the audio device — CI-safe); flags `--flat --xu --dotside --buffer`. Playback: render the whole utterance up front → prefill an rtrb SPSC ring → cpal output callback only pops + fans mono across channels + zero-fills underruns (docs/research/02-architecture-v2.md). Render happens at the *negotiated device rate* (schedule is in ms, so exact, not resampling). Realtime-safety proven by a hand-rolled counting `#[global_allocator]` in tests/realtime.rs (thread-local FORBID flag; `assert_no_alloc` is BSD-1-Clause, banned) — the callback path allocates 0×. Hand-rolled 44-byte RIFF writer keeps hound a dev-dep. 20 new tests (7 args, 2 wav + 1 hound round-trip, 4 realtime, 6 e2e via CARGO_BIN_EXE); 184/184 workspace. Build plumbing: cpal 0.18 (Apache-2.0) + rtrb 0.3 (MIT/Apache); flake devshell += alsa-lib, pkg-config, LD_LIBRARY_PATH; CI clippy+test jobs += libasound2-dev (wasm-build builds only voksa-web, untouched). API notes: cpal 0.18 `sample_rate()` returns u32 (no `.0`), `build_output_stream` takes `config` by value. Live playback bounded by an audio-length+2 s deadline so a broken device can't hang; degrades to the friendly "use --out" message when headless (WSL/CI). Tagged phase8-complete.

- 2026-07-03 — Phase 7.1 — nucleus-scoped stress stretch (CP1 backlog item 6). `SyllableSpan` gained `nucleus_off_ms` (onset consonants + [h] + onset-side buffer, computed in schedule_word via an is_nucleus flag on Entry); prosody's `stretch_stressed_spans` now uses a rhyme-only window `[start+nucleus_off, end)` while the F0 excursion + amplitude boost stay whole-span. Onsets keep unit rate (e.g. coi-munje "mun" 465→425 ms, total 1055→1015). 165/165 tests (6 new compiler/prosody + 2 rewritten to rhyme arithmetic); 12 core snapshots regenerated (8 compiler + normalize field-only, 3 prosody field+timing), engine snapshots byte-identical (lowering reads events, not spans). Battery re-rendered. Committed, NOT re-tagged (phase7-complete stands); owner to ABX-recheck the onset-cluster items.

- 2026-07-03 — Phase 7 CP1 — human sign-off: owner tagged phase7-complete (at e3f3c74). Scores in docs/listening/phase7.md: MOS int avg 2.6 / nat avg 2.3, ABX flat preferred 8/10. Dominant artifact = whole-span stress stretch lengthening onset clusters ("CC strong + long" on pre/kla/zga/dja/DJO) — the §9.1 stop-burst caveat confirmed; nucleus-only stretch queued (backlog item 6). Segmental c/m/j clarity issues on coi-munje (item 7). Rater caught eSpeak being WRONG twice (xu → [k]-like; raw-digit "4") — voksa correct per CLL; oracle-comparability tweak queued (item 8). Rater caveat recorded: self-taught Lojban, eSpeak-referenced scoring.
- 2026-07-03 — Phase 7 — 1 session — Prosody green, 156/156 tests (15 new: 10 schedule-level + 3 insta snapshots in core, 3 acoustic in the adapter, plus testkit F0 self-tests). Deterministic transform: stretch stressed spans 1.5× (piecewise time map; pauses shift, not stretch) → additive declination 120→95 → +20 Hz/×1.2 in-span → optional xu +25 Hz. Measured: declination slope negative with endpoints 120±8/95±8; stressed F0 > unstressed +5 Hz and peak-window RMS higher; xu ending >110 Hz vs flat ~98. pitch-detection 0.3 failed the smoke gate (McLeod formant-locked ~490 Hz) → dropped for a hand-rolled NSDF (plan's designated fallback). Fixed during green: xu rise must also raise later events inside the final span. Battery: 10 utterances × 3 WAVs (prosodic/flat/oracle) + index.html with MOS/ABX capture, clipping assert clean. COMMITTED, NOT TAGGED — CP1 human listening pending.
- 2026-07-02 — Phase 0 — 1 session — Workspace (4 crates), flake (fenix stable 1.96.1 + wasm32 std, espeak-ng), CI, portable Windows/WSL hooks, verifier subagent, skills, working oracle (coi-munje.wav RIFF-valid, 58690 B). TDD smoke: red (wrong assertion) → Stop hook blocked exit 2 → fix → 7/7 green → Stop hook passed. Found+fixed: wasm-pack 0.15 silently swallows wasm-opt failures (binaryen 129 rejects rustc's default bulk-memory ops) via per-crate wasm-opt feature flags; wasm 17171 → 15923 B. Fresh-context verifier: all criteria PASS.
- 2026-07-02 — policy — Auto-push enabled at owner request: PostToolUse hook pushes `origin HEAD --follow-tags` after every git commit/tag. Guard hook now blocks only force pushes and stray rm -rf (push-to-main block removed).
- 2026-07-03 — Phase 1 — 1 session — Engine spike green: klattsch-core 0.1.1 renders steady /a/ measured at F1 719.6 / F2 1079.6 / F3 2519.9 Hz vs targets 730/1090/2440 (1.4% / 1.0% / 3.3% error — inside ±10% gate, near ±5% Phase-2 band). FFT harness (voksa-testkit, spectrum-analyzer 1.7 + hound dev-only) self-tested on synthetic tones. Red-first honored: acceptance test committed failing against a silent skeleton, then implementation. Gate input for Phase 2: klattsch PhonemeTable trait is std-side — plan is own schedule IR in voksa-core, adapter lowers to klattsch; OQ/diplophonia params still absent from klattsch (known, decides at gate). 13/13 tests, clippy/fmt clean, a.wav rendered.
- 2026-07-03 — Phase 6 — 1 session — Normalization green, 141/141 tests (14 new: CLL §18 number vectors incl. 18.3.7 verbatim, round-trip proptest, lerfu table + spell, pause-reproduction of written lerfu forms, end-to-end figures). Proptest caught a real bug pre-commit: the canonical-grouping length check wrongly applied to ungrouped digit runs ("0000" rejected — would have broken CLL 18.2.3's ten-digit string). One test-expectation fix: no pause is mandated between abu and by (neither rule fires) — implementation was right. Tokenizer gained decimal lookahead (period between digits stays in the figure).
- 2026-07-03 — Phase 5 — 1 session — Schedule compiler green, 127/127 tests (20 new: 8 insta utterance snapshots, tokenizer/error vectors, determinism, buffer invariants incl. CLL 8.1 vrusi, writer-period merge). The Phase-2 gate architecture landed: core owns the IR + timing (schedule.rs/compiler.rs), the adapter shrank to a 1:1 event translation with klattsch quirks isolated — proven behavior-preserving by the Phase-2 snapshots passing byte-identical (zero pending) through the refactor. One test-helper fix during green: voiceless stop closures are acoustically silence, so pause counting discriminates by hold length (100 ms vs 60 ms). render_utterance gives end-to-end text→audio.
- 2026-07-03 — Phase 4 — 1 session — Stress + pauses green, 107/107 tests (19 new: CLL stress vectors incl. all §4.8 capital-marked names, complete §4.9 pause ruleset, 2 proptests). Research found phonology.md missing TWO mandatory pause rules (r4 pause-before-cmevla unless la/lai/la'i/doi — classic CLL, not dotside; .y. trailing pause) — both implemented + doc-fixed. iy/uy diphthongs ruled uncountable (CLL §3.4 phonetics + §3.9 weak-stress). One test-vector correction during red→green: e'U-initial utterances open with the r3 vowel-initial pause (CLL writes .e'u) — test amended pre-commit with citation, implementation untouched. Capital→syllable mapping via the Phase-3 round-trip guarantee.
- 2026-07-03 — Phase 3 — 1 session — Syllabifier + classifier green, 88/88 tests (41 new: letters/clusters units, ~30 CLL vectors, 5 proptest properties). Implementation passed the full CLL vector set on the FIRST green attempt (armstrong rm|str split, kat,r,in syllabic R, brlgan forced sonorants, ktraile typed error, bisycla impermissible-pair classification). CLL research corrected phonology.md: forbidden triples ndj/ndz/ntc/nts (§3.7) were missing; classifier needs ANY pair not a permissible one (§4.3); syllabicity is speaker-choice (§3.4) → voksa's deterministic rule documented. proptest 1.11 added (dev-only).
- 2026-07-03 — Phase 2 — 1 session — Full inventory green, 47/47 tests. All 6 vowels within ±5% (F1 floor 50 Hz) by BOTH FFT and LPC (e.g. /a/ LPC 725/1081/2432 vs 730/1090/2440); sibilant centroids s 6363 / z 6436 / c 3425 / j 3291 / x 2458 Hz all in-band; /ai/ F2 glide monotonic 1080→2280. Debug journey worth remembering: (1) klattsch default gain 3.5 soft-clips → corrupted LPC + intermodulation ghosts — adapter renders at gain 1.0; (2) loqa-voice-dsp 0.5 empirically broken (garbage on synthetic vowels) — LPC hand-rolled in testkit (Levinson-Durbin + Durand-Kerner); (3) parallel-topology spectral zeros biased LPC F2 +100–170 Hz — fixed by Klatt-1980 alternating A2 polarity in the lowering (errors → single-digit Hz); (4) FFT harmonic-grid quantization — measurement F0 105 Hz + source-whitened log-parabolic interpolation across ±F0 neighbours. ENGINE GATE: klattsch-core KEPT (ADR 0001); Phase-10 OQ/DI via vendored fork of its glottal source.
