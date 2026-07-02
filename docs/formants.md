# voksa — Formant Seeds and Consonant Loci

Seed values for the LojbanTable phoneme parameters and the acceptance-test
targets. Sources: Peterson & Barney 1952 (JASA 24:175–184, adult-male means,
hVd context) mapped to nearest Lojban vowel categories; locus theory for
consonant transitions; Klatt 1980 (JASA 67(3):971–995) parameter model.

These are SEEDS for a robotic voice, not Lojban ground truth. Hand-tune by ear
against the eSpeak-jbo oracle and fluent-speaker recordings (listening only).
Tests assert against THIS table; if you retune, update table + snapshots together.

## Vowels (F1/F2/F3 Hz, adult-male reference)

| Lojban | IPA | F1  | F2   | F3   | Nearest P&B token | Target dur (ms) |
|--------|-----|-----|------|------|-------------------|-----------------|
| i | [i] | 270 | 2290 | 3010 | /iy/ heed  | 150 |
| e | [ɛ] | 530 | 1840 | 2480 | /ɛ/ head   | 150 |
| a | [a] | 730 | 1090 | 2440 | /ɑ/ hod    | 160 |
| o | [o] | 570 |  840 | 2410 | /ɔ/ hawed  | 150 |
| u | [u] | 300 |  870 | 2240 | /u/ who'd  | 150 |
| y | [ə] | 500 | 1500 | 2500 | central schwa | 100 (short; never stressed) |

Bandwidth seeds: B1 ≈ 60–90 Hz, B2 ≈ 90–110 Hz, B3 ≈ 120–150 Hz.
Stressed vowel: duration ×1.5, +10–30 Hz F0 excursion, small amplitude boost.
Diphthongs: linear/cubic glide from start-vowel targets to end-vowel targets over
one syllable (~180–220 ms); test = F2 moves monotonically toward end-vowel locus.
Buffer vowel (--buffer only): [ɪ]-like, ~400/1900/2600 Hz, 30–40 ms, weak.

## Consonant F2 loci (transition targets; 30–60 ms transitions)

| Place | Members | F2 locus (Hz) | Notes |
|-------|---------|---------------|-------|
| bilabial/labial | b p m f v | 700–1000 | low locus |
| dental/alveolar | d t n s z l r | 1700–1800 | |
| velar | g k x | 2000–2300 | velar pinch: F2≈F3 near closure |

## Noise sources (fricatives; parallel branch)

| Phoneme | Noise band / character |
|---------|------------------------|
| s | high, 4000–8000 Hz centered energy |
| z | as s + voicing |
| c [ʃ] | 2500–3500 Hz centered |
| j [ʒ] | as c + voicing |
| f | weak flat broadband |
| v | as f + voicing |
| x | 1500–2500 Hz velar frication |
| ' [h] | aspiration shaped by FOLLOWING vowel's formants (no locus of its own) |

## Stops
Closure (silence/voice-bar for voiced) ~40–70 ms + release burst:
- b/p: diffuse low-frequency burst
- d/t: burst energy ~alveolar region (3000–4000 Hz emphasis)
- g/k: compact mid burst near velar locus (velar pinch)
Voiced stops: low-amplitude voicing during closure (AV low, not zero).

## Nasals and liquids
- m n: nasal murmur — low F1 (~250–300 Hz), reduced amplitude, anti-resonance
  approximated by lowering mid-band amplitudes (parallel model: attenuate A2/A3).
- l: F1 ~360, F2 ~1300, F3 ~2700 (lateral); r: rhotic signature = lowered F3
  (~1600–1900 Hz).
- Syllabic l̩ m̩ n̩ r̩: same targets, vocalic amplitude envelope + full duration;
  excluded from stress counting (docs/phonology.md §4).

## Engine parameters (Klatt/KLSYN88 vocabulary)
F0 (Hz), AV (voicing amplitude), AH (aspiration), AF (frication),
F1–F5/B1–B5 (formants/bandwidths), OQ (open quotient %), TL (spectral tilt dB),
FL (flutter % of F0), DI (diplophonia %).
klattsch-core exposes: voicing, aspiration/breathiness, tilt, effort,
vibrato depth/rate, tremolo, gain, F0, F1–F3+A/BW. OQ modulation and a
diplophonia/alternate-pulse path likely need ADDING for the attitudinal layer
(.oi creak) — this is the Phase-2 decision-gate question.

## Test tolerances (from docs/research/03 §c)
- Formants: ±5% or ±50 Hz for F1 (whichever larger); F2/F3 ±5–8%; relax to ±10%
  in CI to avoid flakes. Verify with BOTH FFT peak-picking (2048-pt Hann,
  parabolic interpolation) AND LPC root-finding; both must agree.
- F0: ±5 Hz absolute on synthetic audio (McLeod/YIN, 25 ms window, 10 ms hop).
- Drive LPC formant tests with glottal-source-excited vowels, never pure tones.
