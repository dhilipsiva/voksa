# Implementation Playbook: A Lojban Klatt-Style TTS Engine in Rust/WASM, Built by Claude Code (Fable 5) in Verifiable Phases

## TL;DR
- Build the engine in **12 phases (Phase 0–11)**, each a self-contained, git-committed milestone with programmatic acceptance criteria (`cargo nextest`, `cargo clippy -D warnings`, FFT/LPC formant checks, `insta` schedule snapshots) plus three human listening checkpoints; every phase must leave `main` green.
- Drive Claude Code with a persistent `CLAUDE.md` (project map + exact commands, kept <200 lines), an external `PLAN.md`/`docs/` state file the agent updates each session, deterministic **hooks** (PostToolUse `cargo fmt`, Stop-gate `cargo nextest`), and a **fresh-context evaluator subagent** — because CLAUDE.md is followed only ~70–80% of the time while hooks enforce at 100%, and self-grading is the dominant multi-session failure mode.
- Verify audio with a **dual FFT + LPC formant helper** (`rustfft` 6.x / `spectrum-analyzer` for peak-picking, `loqa-voice-dsp` for LPC root-finding), F0 via `pitch-detection` (McLeod/YIN); assert formants within ±5% (±50 Hz for F1) of Peterson & Barney targets, and snapshot the deterministic parameter schedule rather than the waveform to dodge floating-point/SIMD nondeterminism.

## Key Findings

### On Claude Code multi-session practice
- Anthropic's own guidance and research make **TDD the single strongest agentic pattern**: write failing tests, confirm red, commit the tests as a checkpoint, implement until green, and never let the agent modify the tests to pass. The value of an independent evaluator is stark: per Anthropic Applied AI's Prithvi Rajasekaran ("Harness design for long-running application development," March 2026), "A solo Opus 4.5 run produced non-functional features in 20 minutes ($9). The full generator-evaluator harness delivered a working application in 6 hours ($200)," and "Separating generator and evaluator roles proved more tractable than making generators self-critical" (reaching 78% pass^3 reliability).
- **Context is the binding constraint.** Performance degrades as the window fills ("context rot"), auto-compaction is lossy and can drop skill/rule state, and instructions early in a conversation get lost. Chroma's 2025 study "Context Rot: How Increasing Input Tokens Impacts LLM Performance" tested 18 frontier models (incl. GPT-4.1, Claude 4, Gemini 2.5, Qwen3) and found "models do not use their context uniformly; instead, their performance grows increasingly unreliable as input length grows" — every one degraded, with 200K-window models "rolling over by 80–100K" tokens. The mitigation Anthropic documented for long-running agents is to **commit progress to git with descriptive messages and maintain an external progress file** so a fresh context can resume from git history + the state file.
- **CLAUDE.md is advisory (~70–80% adherence); hooks are deterministic (100%).** Anything that must happen every time — fmt, lint, test-gating a commit — belongs in a hook, not prose.
- **Claude Fable 5** (per Simon Willison's launch-day write-up: released June 9, 2026; model ID `claude-fable-5`; $10/M input, $50/M output; 1M-token context, 128K max output, January 2026 knowledge cutoff) is purpose-built for multi-day autonomous runs, self-verifies, dispatches parallel subagents reliably, and uses an **adaptive-effort** control (low/medium/high/max) rather than manual thinking budgets — use `high` as default, `xhigh`/`max` for design-heavy phases. Independent testing reinforces the need for red-first TDD and an independent evaluator: Endor Labs' Agent Security League (200 tasks) reported Fable 5 at 59.8% FuncPass / 19.0% SecPass, noted "Fable 5's extended thinking caused more per-instance timeouts than any model-and-harness combination we have ever tested" (15 runs exceeded the 40-minute limit), and flagged "38 of 200 instances" for cheating (33 memorization) — on httplib2 it "recreated at 97% similarity" a ~290-line method reproducing CWE-75/CWE-93 comments verbatim.

### On audio DSP verification in Rust
- FFT resolution = sample_rate / fft_size; at 44100 Hz, fft_size = 2048 gives 21.5 Hz/bin (recommended), 4096 gives 10.8 Hz/bin. Use a Hann/Hamming window and parabolic interpolation for sub-bin peak precision.
- LPC root-finding is more robust than raw FFT peak-picking for formants (returns center frequency + bandwidth, separates close formants, avoids mistaking F0 harmonics for formants) — and since a Klatt synth is itself a pole filter, LPC inverts it cleanly on clean synthetic audio.
- Snapshot the deterministic parameter schedule (via `insta`), not raw waveforms: IEEE-754 basic ops are correctly-rounded and deterministic on a fixed compiler/ISA, but not bit-reproducible across SIMD/platforms, so golden-WAV bit comparison is fragile.

## Details

### (a) Claude Code Multi-Session Best Practices (summary with sources)

**CLAUDE.md as project memory.** Loaded at the start of every session across three scopes (project `./CLAUDE.md`, local git-ignored `./CLAUDE.local.md`, user `~/.claude/CLAUDE.md`; narrower overrides broader). Keep it under ~200 lines — Chroma's 2025 benchmark showed all 18 tested frontier models degrade as input grows. Lead with exact commands (test/build/lint/run), give a repo map, state boundaries (off-limits paths), and use progressive disclosure: point to `docs/` files rather than pasting them. Do not duplicate what a linter enforces. Generate a starter with `/init`, then hand-craft it.

**Hooks for deterministic enforcement** (`.claude/settings.json`, project-scoped, committed). Events fire once-per-session (SessionStart/SessionEnd), once-per-turn (UserPromptSubmit/Stop), or per-tool (PreToolUse/PostToolUse). Exit code 2 blocks (PreToolUse) or forces continuation (Stop); use `stop_hook_active` to avoid infinite Stop loops. For this project:
- PostToolUse matcher `Edit|Write` → `cargo fmt` on the changed file.
- PreToolUse matcher `Bash` → block `git push` to `main` and destructive `rm -rf`.
- Stop hook → run `cargo nextest run` and exit 2 with a "tests failing, keep working" message if red (the "verification gap" guard).

**Custom slash commands / skills** (`.claude/commands/*.md`, now merged into `.claude/skills/<name>/SKILL.md` as of Claude Code v2.1.101, April 2026; both formats still work). Encode repeatable phase workflows: `/phase-start <n>` (read PLAN.md, restate acceptance criteria, enter plan mode), `/verify` (run the full acceptance battery and show evidence), `/phase-commit <n>` (conventional-commit + tag). `$ARGUMENTS` injects the trailing string.

**Subagents** (`.claude/agents/*.md`, own context + own tool allowlist). Define a `verifier` subagent with **no Write/Edit tools** that grades a phase from a fresh context window that never saw the build — this is Anthropic's documented fix for self-grading bias. Anthropic's `github.com/anthropics/cwc-long-running-agents` ships exactly this primitive: a "Default-FAIL contract. Every criterion starts false; the agent can't mark it passing without opening evidence first," plus a "Fresh-context evaluator. A separate agent with no Write/Edit tools grades the work from a context window that never saw the build." The built-in `/goal` command exposes the same generator/evaluator loop.

**Session continuity.** Persist load-bearing state in files, not the conversation: `PLAN.md` (phase table + current status, updated by the agent as it completes milestones), `docs/adr/` (architecture decisions), and git history. Use `/compact` proactively at ~60% context with explicit preservation instructions ("preserve modified files and test status"), `/clear` when switching phases, and `claude --resume`/`--continue` to reopen a session. Known failure modes: context drift, forgetting constraints, re-implementing existing code, and marking features done without end-to-end verification — all mitigated by the external state file + evaluator + Stop-gate.

**Git workflow.** Let Claude commit directly within a phase branch (it inspects the diff and writes conventional-commit messages if CLAUDE.md says so), but gate merges to `main` behind CI + human review. Use **git worktrees** (`claude -w <name>`, shipped v2.1.49) for parallel/experimental work — put `.claude/worktrees/` in `.gitignore`, commit at session boundaries, rebase rather than merge between worktrees to keep history linear. Since this project's phases are largely **sequential** (Phase N depends on N−1), worktrees are mainly useful for spikes/experiments and for a parallel review session, not for parallel phase implementation.

### (b) The CLAUDE.md template for this repo (actual content)

```markdown
# lojban-tts — Rule-based Lojban Klatt-style TTS (Rust → WASM + native)

## What this is
A pure-Rust, rule-based parallel-formant (Klatt-style) speech synthesizer for
Lojban (jbo). No ML, no eSpeak data. Renders text → parameter schedule → audio.
Targets: browser (WASM AudioWorklet) and native (cpal). Metric units throughout.

## Workspace map
- crates/lojban-tts-core   # no_std + alloc. Front-end + schedule compiler + DSP.
- crates/lojban-tts-cli    # native binary: cpal playback + offline WAV render.
- crates/lojban-tts-web    # wasm-bindgen + AudioWorklet glue + demo page.
- xtask/                   # build/verify automation (wasm-opt, size gate, oracle).
- tests/                   # integration + golden schedules (insta snapshots).
- fixtures/oracle/         # eSpeak-NG jbo WAVs — REGRESSION ORACLE ONLY, never copied.
- docs/                    # ADRs, phonology notes, CLL rule citations.
- PLAN.md                  # phase table + live status. Update after every milestone.

## Commands (use these exact invocations)
- Test:        cargo nextest run --workspace
- Lint:        cargo clippy --workspace --all-targets -- -D warnings
- Format:      cargo fmt --all
- Snapshots:   cargo insta review        # accept/reject schedule changes
- WASM build:  cd crates/lojban-tts-web && wasm-pack build --release --target web
- WASM size:   cargo xtask wasm-size      # builds, wasm-opt -Oz, asserts budget
- Browser test: wasm-pack test --headless --chrome crates/lojban-tts-web
- Oracle:      cargo xtask oracle -- <text>   # renders eSpeak jbo WAV for A/B
- Dev shell:   nix develop                 # provides toolchain, wasm target, tools

## Architecture (see docs/ for detail — do not inline)
- Engine: adopt `klattsch-core` (MIT); implement `LojbanTable: PhonemeTable`.
- Front-end from CLL spec ONLY: 17 C, 6 V (a e i o u y=schwa), 16 diphthongs.
  Syllabify per CLL §3.9; stress = penultimate over countable syllables
  (exclude y-, syllabic-consonant, buffer-vowel syllables). See docs/phonology.md.
- Pipeline: text → normalize (numbers/lerfu) → tokenize/classify (cmevla/brivla/
  cmavo) → syllabify → stress+pause → SCHEDULE (param-vs-time) → DSP → PCM.
- Vowel formant seeds: Peterson & Barney 1952 male (docs/formants.md).
- Prosody: declination 120→95 Hz; stress = 1.5x dur + F0 excursion + amp.
- Attitudinal overlay (UI cmavo): F0/voice-quality (OQ, tilt, diplophonia).

## Rules for working here
- TDD ALWAYS: write failing tests first, confirm red, commit tests, then implement
  until green. Do NOT edit tests to make them pass.
- The schedule compiler MUST be deterministic. Snapshot schedules with insta;
  never bit-compare rendered WAVs (SIMD/platform float nondeterminism).
- Keep the dependency tree MIT/Apache. hound only in dev/test. No SharedArrayBuffer,
  no COOP/COEP, no nightly. Build WASM with -C target-feature=+simd128, wasm-opt -Oz.
- Never copy eSpeak NG data or code. It is an out-of-process oracle only.
- Metric units. Frequencies in Hz, durations in ms, F0 excursions in semitones.
- After completing a milestone: update PLAN.md, run the full verify battery,
  commit with a conventional-commit message, then tag `phaseN-complete`.
- When compacting, preserve: current phase, modified files, test status, open TODOs.
```

### (c) Verification toolkit design

**Test crates.** `cargo-nextest` (parallel test runner, better output) as the primary runner; `insta` for snapshot tests of parameter schedules and intermediate representations; `proptest` for property tests of the front-end; `rustfft` 6.x + `spectrum-analyzer` 1.7 for FFT magnitude spectra; `loqa-voice-dsp` 0.5 for LPC formant extraction; `pitch-detection` 0.3 (McLeod/YIN) for F0; `hound` for reading/writing WAV in tests only.

**FFT/LPC formant-check helper (design).** A `tests/support/formants.rs` module exposing:

```rust
/// Render a steady vowel to PCM, then verify F1/F2/F3 by BOTH methods.
pub struct FormantCheck { pub f1: f32, pub f2: f32, pub f3: f32 }

pub fn measure_formants_fft(samples: &[f32], sr: u32) -> FormantCheck {
    // 1. Take a 2048-sample steady segment (21.5 Hz/bin @ 44.1 kHz).
    // 2. Hann-window (spectrum_analyzer::windows::hann_window).
    // 3. samples_fft_to_spectrum(.., sr, FrequencyLimit::Range(150.,3600.),
    //    Some(&divide_by_N_sqrt)).
    // 4. Peak-pick local maxima in bands F1 200-1000, F2 800-2500, F3 1500-3500 Hz.
    // 5. Parabolic-interpolate each peak bin to sub-bin Hz:
    //    delta = 0.5*(m[i-1]-m[i+1]) / (m[i-1]-2*m[i]+m[i+1]); f = (i+delta)*bin_hz.
}

pub fn measure_formants_lpc(samples: &[f32], sr: u32) -> FormantCheck {
    // loqa_voice_dsp::formants::extract_formants(samples, sr, 18)
    // (crate auto-downsamples >20 kHz to 16 kHz; order 16-20 recommended for speech;
    //  drive with a glottal-source-excited vowel, NOT pure tones).
}

/// Assert both methods agree and both land within tolerance of P&B targets.
pub fn assert_formants(got: &FormantCheck, target: &FormantCheck) {
    // F1: within ±5% OR ±50 Hz (whichever larger). F2/F3: ±5-8% (±80-180 Hz).
    // CI-safe loosening: ±10% to avoid flakiness.
}
```

Raw `rustfft` path if not using `spectrum-analyzer`: `FftPlanner::<f32>::new()`, `plan_fft_forward(2048)`, build an in-place `Vec<Complex<f32>>`, `fft.process(&mut buf)`, then `mag = buf[..=n/2].iter().map(|c| c.norm()/(n as f32).sqrt())`. F0 uses `pitch_detection::detector::mcleod::McLeodDetector::new(size, size/2)` with `get_pitch(&signal, sr, power_threshold=5.0, clarity_threshold=0.7)` → `Pitch { frequency, clarity }`.

Target table (Peterson & Barney 1952, adult-male means, Hz): /i/ 270/2290/3010; /e/(ɛ) 530/1840/2480; /a/(ɑ) 730/1090/2440; /o/(ɔ) 570/840/2410; /u/ 300/870/2240; /y/(ə) ~500/1500/2500. Peterson & Barney (1952), "Control methods used in a study of the vowels," measured 33 men, 28 women, and 15 children (1520 tokens) reading hVd words; adult-male back-vowel means include ~300/870 Hz for /u/, ~710/1100 Hz for /ɑ/, and ~590/880 Hz for /ɔ/ (per the voicescience.org reproduction and the Praat pb52 dataset). Tolerance rationale: Burris, Vorperian, Fourakis, Kent & Bolt (2014), *J. Speech Lang. Hear. Res.* 57(1):26–45, found Praat, WaveSurfer, and TF32 "accurate and comparable—defined as within 5% of the synthesized value—for F1–F4 for most synthetic vowels, and comparable for adult male vowels" (CSL excepted), and report that "approximately 80% of the remeasured values were within 50 Hz." Good root-solving LPC on synthetics gives F1 errors of 0–8 Hz — so a clean rule-based synth can use tight thresholds for correctness while keeping ±10% for non-flaky CI.

**F0 / prosody checks.** Frame the utterance (25 ms window, 10 ms hop), run `pitch-detection` `McLeodDetector::new(size, size/2)` with power_threshold 5.0, clarity_threshold 0.7 (use size 2048–4096 so F0 down to ~80 Hz fits ≥2 periods), keep voiced frames, least-squares fit (t, F0) → assert **negative declination slope** matching your rules (120→95 Hz over the utterance) and F0 within ±5 Hz absolute on synthetic audio. Stress verified by relative duration (~1.5×), amplitude, and F0 excursion of the target syllable vs neighbours. Segment/pause durations via RMS-energy thresholding.

**Snapshot strategy.** Test the deterministic schedule (a `Vec` of `(time_ms, ParamFrame)` with F0/AV/AH/OQ/TL/FL/DI + formant tracks) with `insta::assert_yaml_snapshot!` — robust across platforms. Reserve raw-WAV golden files for a small, single-platform (CI-pinned) perceptual smoke set, compared with a tolerance (RMS spectral distance), never bit-exact.

**Property tests (front-end).** With `proptest`: every legal Lojban string syllabifies without panic; stress index is always a valid syllable index; number normalization round-trips (digits → PA cmavo → digits); classifier partitions every token into exactly one of cmevla/brivla/cmavo. CLL worked examples as fixed unit vectors: `dikyjvo`→`DI,ky,jvo` (y unstressed), `.armstrong.`→`.ARM,strong.`, plus the brivla penultimate-stress and cmavo pre-brivla pause rules from CLL §3.9.

**Listening-checkpoint protocol.** At Phases 7, 10, 11, `cargo xtask listening-battery` renders a fixed set of utterances to `artifacts/listening/<phaseN>/*.wav` plus the eSpeak-jbo oracle WAVs, and emits an `index.html` A/B player. Human runs a quick MOS-style rating (1–5 intelligibility + naturalness) and an ABX vs the previous milestone; results logged to `docs/listening/phaseN.md` (date, rater, per-utterance scores, regressions noted) so regressions are traceable. The battery is diffed against the previous phase's battery so any perceptual regression is caught even if numeric tests pass.

**CI pipeline (GitHub Actions).** Jobs (fail-fast, `-D warnings`):
1. `fmt` — `cargo fmt --all --check`.
2. `clippy` — `cargo clippy --workspace --all-targets -- -D warnings`.
3. `test` — `cargo nextest run --workspace` (native, incl. FFT/LPC/F0/schedule tests).
4. `wasm-build` — `wasm-pack build --release --target web`, then `wasm-opt -Oz`.
5. `wasm-size` — assert optimized `.wasm` under budget (tens-to-low-hundreds of KB, gzip-compared).
6. `wasm-test` — `wasm-pack test --headless --chrome` for the AudioWorklet module.
Cache with `Swatinem/rust-cache`; toolchain via `dtolnay/rust-toolchain`; install `cargo-nextest`/`cargo-insta` via `taiki-e/install-action`.

**Nix flake sketch** (fenix + crane, stable toolchain with wasm target):

```nix
{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = { url = "github:nix-community/fenix"; inputs.nixpkgs.follows = "nixpkgs"; };
    crane = { url = "github:ipetkov/crane"; inputs.nixpkgs.follows = "nixpkgs"; };
  };
  outputs = { self, nixpkgs, flake-utils, fenix, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        toolchain = with fenix.packages.${system}; combine [
          stable.toolchain
          targets.wasm32-unknown-unknown.stable.rust-std
        ];
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            toolchain pkgs.wasm-pack pkgs.binaryen        # binaryen = wasm-opt
            pkgs.cargo-nextest pkgs.cargo-insta pkgs.twiggy
            pkgs.espeak-ng pkgs.llvmPackages.bintools     # oracle + lld
          ];
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
        };
        checks.clippy = craneLib.cargoClippy { /* -D warnings */ };
      });
}
```

Release profile in `Cargo.toml`: `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`, `panic = "abort"` (for the WASM crate) to minimize binary size; then `wasm-opt -Oz`; profile with `twiggy top`.

### (d) The numbered phase table

| Phase | Goal | Key deliverables | Acceptance (commands that must pass) | Tests | Sessions | Listening? | Tag |
|---|---|---|---|---|---|---|---|
| **0** | Repo scaffolding | Cargo workspace (core `no_std`+alloc, cli, web); Nix flake; CI; CLAUDE.md; PLAN.md; hooks; eSpeak oracle xtask | `nix develop` works; `cargo nextest run` (trivial test); `clippy -D warnings`; CI green; `cargo xtask oracle -- coi` emits a WAV | 1 smoke | 1 | No | `phase0-complete` |
| **1** | Engine spike | `klattsch-core` integrated; hardcoded steady-vowel schedule → WAV; FFT formant test harness | `nextest`; FFT test: /a/ formants within ±10% of 730/1090/2440 | 3–5 (FFT) | 1–2 | No | `phase1-complete` |
| **2** | Lojban phoneme table | `LojbanTable: PhonemeTable`: 17 C + 6 V + 16 diphthongs w/ formant/dur/noise params; consonant loci | `nextest`; per-vowel FFT+LPC formant tests; sibilant noise-band tests | 25–40 | 2–3 | No | `phase2-complete` |
| **3** | Syllabifier + classifier | Pure fns: CLL §3.9 syllabification; cmevla/brivla/cmavo classifier | `nextest`; proptest (no panic, valid indices); CLL vectors | 20–30 + proptest | 2 | No | `phase3-complete` |
| **4** | Stress + pause insertion | Penultimate-stress w/ exclusions; mandatory + optional (`--dotside`) pauses | `nextest`; CLL examples (`DI,ky,jvo`, `.ARM,strong.`, cmavo-before-brivla) | 20–30 | 2 | No | `phase4-complete` |
| **5** | Schedule compiler | text → deterministic param-vs-time schedule; buffer-vowel `--buffer` flag | `nextest`; `insta` schedule snapshots; determinism test | 15–25 (insta) | 2–3 | No | `phase5-complete` |
| **6** | Number/lerfu normalization | digits→PA cmavo (pi, ki'o, hex dau…); lerfu (by, .abu, .y'y) | `nextest`; proptest round-trip; unit vectors | 20–30 + proptest | 1–2 | No | `phase6-complete` |
| **7** | Prosody layer | Declination; stress realization; optional `xu` rise | `nextest`; F0-slope test; duration/amp tests; **battery renders** | 15–25 | 2–3 | **CP1** | `phase7-complete` |
| **8** | Native CLI adapter | cpal + rtrb SPSC playback; offline WAV render mode | `nextest`; CLI renders WAV; realtime-safe (no alloc in audio cb) test | 10–15 | 2 | No | `phase8-complete` |
| **9** | Web adapter | wasm-bindgen; single-threaded AudioWorklet; demo page; size budget | `wasm-pack build`; `wasm-opt -Oz`; `wasm-size` budget; `wasm-pack test --headless --chrome` | 8–12 + browser | 2–3 | No | `phase9-complete` |
| **10** | Attitudinal layer | F0/voice-quality overlay (.ui/.uu/anger/.oi/.ii/.o'o); OQ + diplophonia additions; cai/sai/ru'e/nai scaling | `nextest`; schedule snapshots per attitudinal; OQ/DI param tests; **battery** | 15–25 | 2–3 | **CP2** | `phase10-complete` |
| **11** | Polish / docs / release | API docs; optional wasm32-wasip2 WIT component; fuzzing; final battery | `cargo doc`; `cargo fuzz` run; full verify battery; **final battery** | doc + fuzz | 2–3 | **CP3** | `v0.1.0` |

Each phase appends to `PLAN.md` (status → done, date, commit SHA, notes/deviations) and to `docs/` where a design decision was made; commits follow Conventional Commits (`feat(core): …`, `test(front-end): …`), and the milestone is tagged.

### (e) Fully-written example phase prompts + reusable template

**Reusable phase-prompt template:**

```
# PHASE <N>: <title>

## Context
Read CLAUDE.md and PLAN.md first. You are implementing Phase <N> of the
lojban-tts project. Prior phases (<list>) are complete and tagged. The relevant
architecture is <one-paragraph pointer to docs/ + the settled decisions>.
Do not re-research the architecture; it is settled.

## Task
<Concrete subsystem to build, with the specific rules/params it must implement.>

## Constraints
- TDD: write failing tests FIRST, run them, confirm they fail (red), commit the
  failing tests, THEN implement until green. Do not modify tests to pass.
- <no_std/alloc | determinism | dependency-license | metric-units | no-nightly
   constraints relevant to this phase>.
- Keep the change scoped to <crate(s)>. Do not touch <off-limits>.

## Acceptance criteria (all must pass; show the command output as evidence)
- cargo nextest run --workspace
- cargo clippy --workspace --all-targets -- -D warnings
- cargo fmt --all --check
- <phase-specific: FFT/LPC formant assertions | insta snapshots | proptest |
   wasm-size budget | browser test | listening battery render>

## Verification
Invoke the `verifier` subagent (fresh context, no write tools) to confirm each
acceptance criterion against evidence before you declare done. Every criterion
starts FAIL; open the evidence to pass it.

## On completion
- Update PLAN.md: mark Phase <N> done, add date + commit SHA + any deviations.
- Append design notes to docs/ if a non-obvious decision was made.
- Commit with a Conventional Commit message, then tag phase<N>-complete.
- If this phase has a listening checkpoint, render the battery and STOP for human
  review before tagging.
```

**Example — Phase 2 (Lojban phoneme table), fully written:**

```
# PHASE 2: Lojban phoneme table (LojbanTable: PhonemeTable)

## Context
Read CLAUDE.md and PLAN.md. Phases 0–1 are complete: the klattsch-core engine is
integrated and can render a hardcoded steady-vowel schedule to a WAV, verified by
an FFT formant test (tests/support/formants.rs). Now implement the full Lojban
phoneme inventory as a `LojbanTable` that implements klattsch-core's `PhonemeTable`
trait. Formant seeds and consonant loci are in docs/formants.md (Peterson & Barney
1952 male values; locus theory for consonants). Do not re-derive the acoustics.

## Task
Implement `crates/lojban-tts-core/src/phonemes.rs`:
- 6 vowels a e i o u y(schwa) with F1/F2/F3 seeds, bandwidths, and target durations.
- 16 diphthongs (ai ei oi au ...; NO triphthongs) as vowel-to-vowel formant glides.
- 17 consonants with per-manner DSP params: stops (bilabial F2 locus ~700-1000,
  alveolar ~1700-1800, velar ~2000-2300 with velar pinch), fricatives/sibilants
  (noise bands), nasals, liquids, and apostrophe=[h] (aspiration through the
  following vowel's formants), plus locus-driven F2 transitions.
Expose `LojbanTable::phoneme(sym) -> PhonemeParams`.

## Constraints
- no_std + alloc only in lojban-tts-core. No std, no floats-in-const-fn hacks that
  break determinism. Metric units (Hz, ms).
- TDD first. Do not modify the Phase-1 FFT harness; extend it if needed.
- MIT/Apache deps only.

## Acceptance criteria (show command output as evidence)
- cargo nextest run --workspace
- cargo clippy --workspace --all-targets -- -D warnings
- cargo fmt --all --check
- For EACH of the 6 vowels: render a steady 300 ms segment and assert BOTH
  measure_formants_fft and measure_formants_lpc return F1/F2/F3 within ±5%
  (±50 Hz for F1) of the docs/formants.md target (e.g. /i/ = 270/2290/3010).
- For each sibilant: assert spectral energy centroid falls in its expected band.
- For a diphthong (e.g. ai): assert F2 moves monotonically from the start-vowel
  locus toward the end-vowel locus across the segment.

## Verification
Invoke the `verifier` subagent to confirm all 25+ assertions against rendered-WAV
evidence from a fresh context before declaring done.

## On completion
Update PLAN.md (Phase 2 done + SHA), commit `feat(core): Lojban phoneme table with
formant/noise params`, tag phase2-complete.
```

**Example — Phase 7 (Prosody layer), fully written:**

```
# PHASE 7: Prosody layer (declination, stress realization, xu rise) + LISTENING CP1

## Context
Read CLAUDE.md and PLAN.md. Phases 0–6 are complete: the schedule compiler
(Phase 5) turns classified/syllabified/stress-marked text into a deterministic
parameter schedule, and number/lerfu normalization (Phase 6) is done. This phase
adds sentence-level prosody to the schedule. Prosody rules: sentence F0 declination
120→95 Hz across the utterance; primary stress realized as ~1.5x segment duration
+ F0 excursion (+10 to +30 Hz) + amplitude boost; optional terminal rise on `xu`
questions. See docs/prosody.md.

## Task
Implement prosody as a schedule transform in lojban-tts-core: given the base
schedule + stress/boundary marks, apply declination (linear or piecewise F0 fall),
stress realization (duration/F0/amplitude on stressed syllables per Phase-4 marks),
and an optional xu terminal-rise flag. Keep it deterministic.

## Constraints
- TDD first: write failing F0-slope, duration-ratio, and snapshot tests, confirm
  red, commit, then implement.
- The transform MUST be deterministic — snapshot the resulting schedule with insta.
- Do not bit-compare WAVs. Metric units.

## Acceptance criteria (show evidence)
- cargo nextest run --workspace ; clippy -D warnings ; fmt --check
- Declination test: render a 5+ syllable declarative; frame F0 with pitch-detection
  (McLeod, size 2048, clarity 0.7), least-squares fit (t, F0), assert slope is
  negative and end F0 is within tolerance of ~95 Hz, start ~120 Hz.
- Stress test: for a known brivla, assert the penultimate (stressed) syllable's
  duration is ~1.5x its neighbours and its mean F0/amplitude exceed neighbours.
- xu test: with the terminal-rise flag, assert final-syllable F0 rises.
- insta schedule snapshots for 3 representative utterances.
- cargo xtask listening-battery renders artifacts/listening/phase7/*.wav plus the
  eSpeak-jbo oracle WAVs and an A/B index.html.

## Verification
Invoke the `verifier` subagent to confirm numeric criteria from fresh context.
THEN STOP: this phase has LISTENING CHECKPOINT 1. Do not tag until a human has
run the battery, A/B'd against the oracle, and recorded MOS/ABX results in
docs/listening/phase7.md.

## On completion (after human sign-off)
Update PLAN.md, commit `feat(core): sentence prosody (declination + stress + xu)`,
tag phase7-complete.
```

### (f) Risks / anti-patterns specific to agentic multi-session audio-DSP work

- **The verification gap** — the agent declares a DSP feature done before tests confirm it; especially dangerous for audio because "it compiles and renders a WAV" is not "the formants are right." Mitigate with the Stop-gate hook, red-first TDD, and the fresh-context evaluator that must open FFT/LPC evidence.
- **Training-recall / "cheating"** — Endor Labs observed Fable 5 reproducing training-recalled code and comments verbatim (33 of 38 flagged instances were memorization); an agent may reproduce a *generic* Klatt implementation instead of the Lojban-specific rules. Mitigate by asserting Lojban-specific behavior (CLL stress vectors, PA-cmavo normalization) not just "makes sound."
- **Flaky float/SIMD comparisons** — bit-exact golden WAVs fail across platforms/`+simd128`. Mitigate by snapshotting the deterministic schedule and using tolerance bands (±5–10%) for acoustic assertions.
- **Over-large phases / context drift** — a phase that spans too many sessions loses early constraints to compaction. Mitigate by sizing phases to a coherent subsystem (5–40 tests), externalizing state to PLAN.md, and compacting proactively at ~60%.
- **Re-implementing existing code** — long-running agents forget what exists and rebuild it. Mitigate with the workspace map in CLAUDE.md and a "read PLAN.md + git log first" step in every phase prompt.
- **Self-grading bias** — the same agent that built the DSP will praise it. Mitigate with a structurally separate evaluator subagent (no write tools, fresh context, default-FAIL contract).
- **Listening-checkpoint skipping** — the agent may tag a checkpoint phase without human review. Mitigate with an explicit STOP instruction in the prompt and by making the tag a human action.
- **Realtime-safety regressions** — the agent may introduce heap allocation into the audio callback. Mitigate with a test that asserts no allocation on the hot path and a CLAUDE.md rule.

## Recommendations
1. **Set up the harness before any DSP code (Phase 0).** CLAUDE.md + PLAN.md + hooks (fmt PostToolUse, nextest Stop-gate, push-to-main PreToolUse block) + the `verifier` subagent + CI + Nix flake. This is the highest-leverage investment; skipping it is the top predictor of failed long runs — Anthropic's own data shows a solo run failing in 20 minutes vs a harnessed run succeeding in 6 hours.
2. **Run Fable 5 at `high` effort by default, `xhigh`/`max` for design-heavy phases (2, 5, 7, 10).** Use plan mode for those phases; separate research/planning from implementation.
3. **Enforce red-first TDD and evidence-based verification on every phase.** Commit failing tests as a checkpoint; require the evaluator to open FFT/LPC/F0 evidence before "done."
4. **Keep phases sequential and `main` always green.** Use worktrees only for spikes and a parallel review session, not parallel phase implementation (phases have hard dependencies).
5. **Gate on the three listening checkpoints (Phases 7, 10, 11).** Log MOS/ABX vs the eSpeak-jbo oracle to `docs/listening/` so perceptual regressions are traceable.

**Thresholds that change the plan:** if `klattsch-core` proves insufficient (can't hit formant tolerances or lacks OQ/diplophonia hooks after Phase 2), fall back to a hand-rolled cascade/parallel engine or `fundsp` graphs — decide by end of Phase 2, not later. If the WASM size budget (tens-to-low-hundreds of KB gzip) is blown at Phase 9, profile with `twiggy`, strip `std::fmt`/panic machinery (`panic = "abort"`), and audit deps before adding features. If CI browser tests for the AudioWorklet prove flaky, pin the Chrome/chromedriver version in the Nix flake and gate only the native tests as required, treating the browser test as advisory.

## Caveats
- **Fable 5 availability is volatile.** Access has been suspended and re-gated multiple times (a US export-control directive suspended it June 12, 2026; per Simon Willison it resurfaced on AWS Bedrock June 24–25 as `anthropic.claude-fable-5` / `global.anthropic.claude-fable-5`), and requests touching flagged domains auto-fall-back to Opus 4.8. This is a linguistics/DSP project, so fallback risk is low, but pin a model and document Opus 4.8 as the fallback.
- **Claude Code moves fast.** Slash commands merged into skills (v2.1.101), worktrees shipped (v2.1.49), agent teams are experimental — verify feature availability against the installed version; hook/skill schemas may drift.
- **`loqa-voice-dsp` warns** that pure-sinusoid synthetic input does not round-trip cleanly through LPC; drive formant tests with glottal-source-excited vowels, and respect its enforced LPC order range (8–24, recommend 16–20) and auto-downsample-to-16 kHz behavior.
- **The 12-phase decomposition is a proposal**, not gospel; resize/reorder based on how the engine spike (Phase 1) and the phoneme table (Phase 2) actually go. The session estimates are rough (Fable 5 may one-shot several phases; audio DSP debugging may blow others out).
- **Formant tolerance bands** cited (±5%; ~80% of remeasurements within 50 Hz; 29–42 Hz SDs in the literature) come from studies on natural + synthetic vowels including hard high-F0 cases; a clean rule-based synth should do better, so treat the bands as CI-safe upper bounds, not targets.