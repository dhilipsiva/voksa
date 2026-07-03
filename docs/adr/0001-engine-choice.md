# ADR 0001: Keep klattsch-core as the synthesis engine

Date: 2026-07-03 (end-of-Phase-2 decision gate, per PLAN.md)
Status: ACCEPTED

## Question

Is klattsch-core 0.1.1 sufficient as voksa's formant engine — formant
tolerances hit across the full phoneme inventory, and a feasible path to the
Phase-10 attitudinal voice-quality parameters (OQ, diplophonia) — or must it
be replaced by a hand-rolled cascade/parallel engine or fundsp graphs?

## Decision

**Keep klattsch-core, pinned `=0.1.1`, behind the voksa-engine-klattsch
adapter.** For Phase 10, plan to vendor/fork it (MIT permits) and extend the
glottal source with OQ and a diplophonia/alternate-pulse path, rather than
hand-rolling a new engine.

## Evidence (Phase-2 acceptance battery, all 47 tests green)

Vowels — measured by BOTH FFT (whitened harmonic interpolation) and
hand-rolled LPC (autocorr + Levinson-Durbin order 18 + Durand-Kerner roots),
tolerance ±5% (F1 floor ±50 Hz, F3 ±8%):

| V | target | FFT | LPC |
|---|--------|-----|-----|
| a | 730/1090/2440 | 753/1060/2433 | 725/1081/2432 |
| e | 530/1840/2480 | 537/1837/2468 | 526/1831/2507 |
| i | 270/2290/3010 | 263/2307/3015 | 307/2292/3022 |
| o | 570/840/2410 | 578/822/2415 | 572/852/2400 |
| u | 300/870/2240 | 350/857/2244 | 338/872/2240 |
| y | 500/1500/2500 | 522/1488/2509 | 513/1469/2522 |

Sibilant frication centroids (>1 kHz): s 6363 (4000–8000), z 6436, c [ʃ] 3425
(2500–3500), j [ʒ] 3291, x 2458 (1500–2500). Voicing split clean (z/j
low-band ratio 0.18/0.27 vs s/c/x ≤ 0.0001). Diphthong /ai/ F2 glide
monotonic 1080 → 2280 Hz. Stops, nasals, liquids, [h]: all in-band.

## Engine findings that shaped the adapter (quirks, not blockers)

1. **std-only crate** — must never enter no_std voksa-core; the adapter owns
   the dependency (decided in Phase 1, confirmed).
2. **`PhonemeTable`/`PhonemeParams` are lossy** (no duration, per-phoneme
   aspiration, noise bands, or F0) — voksa-core keeps its own richer IR
   (`phonemes::SegmentSpec`); the klattsch trait is deliberately NOT
   implemented.
3. **Parallel topology carves spectral zeros**: all three bandpass branches
   sum positively, so adjacent branches phase-cancel between resonances. The
   adapter applies Klatt-1980 alternating polarity (A2 negated) in
   `update_from` — this alone took LPC F2 errors from +100–170 Hz to
   single-digit Hz.
4. **Default gain 3.5 drives `soft_clip` nonlinear** — the adapter renders at
   gain 1.0; clipping had corrupted LPC poles and spawned intermodulation
   peaks.
5. **Noise IS spectrally shaped** (mixed into the excitation before the
   filters) — sibilants work by parking a resonator in the frication band,
   klattsch's own ARPABET pattern. But there is **no independent frication
   path** (Klatt's AF + parallel bypass): noise and voice share the three
   resonators, so a fricative "borrows" a formant slot. Quality ceiling for
   VCV transitions, acceptable for a deliberately robotic voice.
6. **Tilt is a 1st-order FIR**, not Klatt TL dB/oct; **no nasal
   anti-resonance** (approximated by attenuating A2/A3 — passed the
   attenuation tests).

## Phase-10 path (OQ / diplophonia)

klattsch-core has no OQ or DI parameters. Its Rosenberg glottal pulse is
already parameterized (`effort` maps to pulse shape), so a small vendored
fork can add: OQ = open-quotient control of the Rosenberg open phase; DI =
alternate-period amplitude/period perturbation. Both are localized changes in
the source generator (~1.5K SLoC crate, MIT, 3 commits — practical to own).
Re-evaluate at Phase 10; if the fork proves messy, the fallback remains a
hand-rolled source feeding the same adapter surface.

## Rejected alternative

Hand-rolled cascade/parallel engine or fundsp graphs now: unnecessary — every
Phase-2 acceptance criterion passed at ±5% with klattsch + a thin adapter,
and a rewrite would forfeit the parity-tested engine for speculative gains.
