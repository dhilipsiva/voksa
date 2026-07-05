# voksa

A pure-Rust, rule-based parallel-formant (Klatt-style) speech synthesizer for
**Lojban**. No machine learning, no recorded voice data, no eSpeak data — the
entire pipeline is `text → deterministic parameter schedule → PCM`, with every
linguistic rule derived from the CLL (Complete Lojban Language) specification.

*voksa* = Lojban gismu: x1 is the voice/speech sounds of x2.

- **Deterministic**: the same text always compiles to the identical schedule
  (snapshot-tested). "Mechanical and algorithm-driven" is the design goal —
  the ceiling is high-end 1990s formant TTS, by choice.
- **Two targets**: the browser (a ~41 KB gzipped, zero-import wasm module
  played inside an AudioWorklet) and native (cpal playback / offline WAV).
- **Community-tunable**: every prosody knob, all 7 attitudinal deviation
  vectors, and the full 377-parameter per-phoneme voice table are runtime
  parameters with a tuning console UI and a shareable JSON config format.

## Try it

The tuning console is a Dioxus app:

```sh
nix develop
cd crates/voksa-console-demo
dx serve                       # then open the printed localhost URL
```

Type Lojban (try the curated phonetic test sentences), watch the live
phonetic analysis, retune prosody / naturalness / attitudinals / the full
per-phoneme voice table, and export your config JSON to share. The console is
also a reusable Dioxus **component** (`crates/voksa-console`), consumed by
dhilipsiva.dev as a git dependency — see `docs/handoff-dhilipsiva-dev.md`.

## CLI

```sh
cargo run -p voksa-cli -- "coi munje"                 # play live (needs audio)
cargo run -p voksa-cli -- --out hello.wav "coi munje" # offline 48 kHz WAV
cargo run -p voksa-cli -- --xu "xu do klama"          # question rise
cargo run -p voksa-cli -- --config tuned.json --out t.wav  # replay a demo config
```

Flags: `--flat` (no prosody), `--xu`, `--dotside`, `--buffer`. A config JSON
downloaded from the web demo replays **bit-identically** through `--config`.

## What it implements

- CLL morphology: 17 consonants, 6 vowels, 16 diphthongs; syllabification
  (§3.9), penultimate stress with exclusions, the word classifier
  (cmevla/brivla/cmavo), and all mandatory pause rules.
- Number/lerfu normalization (§18/§17): digits → PA cmavo, `pi`, `ki'o`,
  `pi'e`, lerfu words.
- Prosody: declination (120→95 Hz), nucleus-scoped stress stretch, F0
  excursion + amplitude boost, optional `xu` rise — plus the Phase-11
  naturalness layer (flutter, breath, micro-prosody, duration rules, vowel
  undershoot; see `docs/phonology.md` §9).
- Attitudinals (INVENTED, non-normative): `.ui .uu .oi .ii .o'o .au .o'onai`
  color the preceding word with F0 + voice-quality deviations
  (open quotient, spectral tilt, diplophonia, vibrato, breath), scaled by
  `cai`/`sai`/`ru'e` and flipped by `nai` (`docs/phonology.md` §10).

## Architecture

```
voksa-core            no_std text front-end + schedule compiler (all CLL rules)
voksa-engine-klattsch std adapter: schedule IR → klattsch parameter events
klattsch-core-fork    vendored klattsch-core 0.1.1 (MIT) + OQ/diplophonia/flutter
voksa-cli             native playback (cpal, alloc-free callback) + WAV render
voksa-web             raw C-ABI wasm + AudioWorklet (non-Rust embedding surface)
voksa-console         Dioxus tuning-console component (ADR 0003) — the primary UI
voksa-console-demo    standalone `dx serve` runner (vendored QUINE tokens)
voksa-component       wasm32-wasip2 WIT component (voksa:synth) — see ADR 0002
voksa-testkit         dev-only FFT/LPC/NSDF measurement harness
```

Authoritative docs: `docs/phonology.md` (the CLL distillation),
`docs/formants.md` (acoustic targets), `docs/research/` (architecture),
`docs/adr/` (decisions), `PLAN.md` (live phase status).

## Development

```sh
nix develop                    # everything below assumes the dev shell
cargo nextest run --workspace  # the test battery (incl. proptest fuzz suites)
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask wasm-size          # browser wasm gate: <43 KB gzip, ZERO imports
cargo xtask console-size       # Dioxus console bundle gate (gzip canary)
cargo xtask fuzz               # deep fuzz (PROPTEST_CASES=65536)
cargo xtask listening-battery  # CP1 human listening artifacts
```

Rules of the house: TDD red-first, deterministic schedule compiler (snapshot
WAVs never bit-compared — acoustic tests use tolerance bands), MIT/Apache
dependencies only, eSpeak NG used strictly as an out-of-process regression
oracle. MSRV: 1.85.

## Help tune the voice

voksa's segment values and attitudinal vectors are documented conventions,
not ground truth — if you speak Lojban, the tuning console is the
contribution surface: adjust, listen, and share the exported JSON (it carries
the phonetic analysis and your notes) in an issue.

## License

MIT OR Apache-2.0 (see `LICENSE-MIT` / `LICENSE-APACHE`). The vendored
`crates/klattsch-core-fork` is klattsch-core 0.1.1 © Tony Gies, MIT.
