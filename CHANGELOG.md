# Changelog

All notable changes to voksa are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[SemVer](https://semver.org/).

## [0.1.0] — pending CP3 (the human tags the release after the final listening battery)

The first release: the complete text → schedule → PCM pipeline (Phases 0–11).

### Added
- **CLL front-end** (`voksa-core`, no_std): 17 consonants / 6 vowels /
  16 diphthongs, §3.9 syllabification, penultimate stress with exclusions,
  cmevla/brivla/cmavo classification, all mandatory pause rules
  (+ `--dotside`), optional buffer vowels (`--buffer`), number/lerfu
  normalization (PA cmavo, `pi`, `ki'o`, `pi'e`, lerfu words).
- **Deterministic schedule compiler**: engine-neutral parameter IR
  (insta-snapshotted), compiled 1:1 into klattsch events by the adapter.
- **Engine**: vendored klattsch-core 0.1.1 fork with three added glottal
  lanes — open quotient, diplophonia, Klatt flutter (defaults byte-identical
  to upstream).
- **Prosody**: declination 120→95 Hz, nucleus-scoped stress stretch (×1.5),
  +20 Hz stress excursion, ×1.2 amplitude boost, optional xu terminal rise.
- **Naturalness layer** (Phase 11, default ON): F0 flutter, breath,
  OQ/tilt baselines, intrinsic vowel F0, obstruent F0 perturbation,
  phrase-final lengthening, cluster shortening, duration-dependent vowel
  undershoot. `naturalness off` reproduces the Phase-10 voice byte-exactly.
- **Attitudinals** (invented, non-normative): `.ui .uu .oi .ii .o'o .au
  .o'onai` deviation vectors with `cai`/`sai`/`ru'e`/`nai` intensity.
- **Native CLI** (`voksa`): live cpal playback with an alloc-free audio
  callback, offline WAV render, JSON tuning-config replay.
- **Browser demo** (`voksa-web`): zero-import wasm in an AudioWorklet
  (~41 KB gzip), tabbed tuning console — prosody + naturalness sliders,
  7 attitudinal panels, the full 377-parameter per-phoneme voice table,
  curated phonetic test sentences, live phonetic transcription, auto-speak,
  config JSON export/import, WAV download.
- **WIT component** (`voksa-component`): `voksa:synth` world for
  wasm32-wasip2 hosts (ADR 0002).
- **Quality infrastructure**: proptest fuzz suites (totality, schedule
  sanity, hostile param blocks) with a weekly deep-soak CI job; acoustic
  acceptance tests (FFT/LPC/NSDF harness); wasm size + zero-import gate;
  human listening checkpoints (CP1 Phase 7, CP2 Phase 10, CP3 Phase 11).

[0.1.0]: https://github.com/dhilipsiva/voksa/releases/tag/v0.1.0
