# Building a Robotic Lojban Text-to-Speech Engine in Rust + WASM

## TL;DR
- **Build a pure-Rust rule-based formant (Klatt-style) synthesizer with a hand-tuned Lojban phoneme table** as the core crate, exposed to the browser via wasm-bindgen inside an AudioWorklet and to native via cpal. This gives full control, a deliberately robotic aesthetic, a small WASM binary, no training data, and no cloud dependency — and it maps cleanly onto the user's WIT/modular engineering style.
- Lojban is almost ideal for rule-based synthesis: its orthography is strictly phonemic (6 vowels /a e i o u y/, 17 consonants, apostrophe = /h/, period = pause/glottal stop), stress is deterministic (penultimate non-y syllable of brivla/cmevla), and word type is recoverable by a self-segmenting morphology parser you likely already have.
- eSpeak NG already ships a working Lojban voice (`jbo`, formant synthesis, `STRESSPOSN_2R` penultimate stress, apostrophe→[h], vowel-pause logic) and compiles to WASM — use it as the fastest reference/fallback and to validate your own engine, but reimplement in Rust for control, licensing cleanliness (eSpeak is GPLv3), and the attitudinal-pitch feature that no existing engine has.

## Key Findings

**Lojban phonology is a synthesizer's dream target.** Per CLL chapter 3, the language has 6 vowels, 17 consonants (b c d f g j k l m n p r s t v x z), plus the apostrophe /h/, and 16 diphthongs. Orthography is phonemic with audio-visual isomorphism — each grapheme maps to one phoneme with a narrow allowed range. There is no lexical stress irregularity to store: primary stress falls on the penultimate syllable (ignoring y-nucleus syllables) of brivla and cmevla; cmavo are unstressed or freely stressed. This means a correct synthesizer needs only (a) a phoneme→acoustic-parameter table, (b) a syllabifier, (c) a word-type classifier, and (d) a prosody layer.

**eSpeak NG's `jbo` voice is real, small, and instructive.** The synthesizer engine is formant-based. The Lojban phoneme table is trivially small — `ph_lojban` inherits Esperanto's phoneme set (`phonemetable jbo eo` / `include ph_lojban`) and overrides essentially one vowel (`e`). All the Lojban-specific logic lives in `tr_languages.c`. This is a strong signal that a from-scratch Rust engine is tractable: the hard linguistic work is a few hundred lines of rules, not a corpus.

**No Lojban training data is needed and none of the recommended paths require it.** Rule-based formant synthesis derives its targets from published acoustic phonetics: Peterson, G.E. & Barney, H.L. (1952), "Control methods used in a study of the vowels," *JASA* 24(2):175–184 (vowel formants); consonant locus theory; and the Klatt 1980 parameter set — Klatt, D.H., "Software for a cascade/parallel formant synthesizer," *Journal of the Acoustical Society of America* 67(3):971–995, March 1980. This sidesteps the fundamental problem that there is no large recorded Lojban speech corpus.

**The attitudinal-pitch feature is genuinely novel.** Lojban deliberately encodes emotion lexically (UI cmavo like `.ui` joy, `.u'u` repentance, `.oi` complaint/pain), and CLL explicitly states attitudinals "require no specific intonation or gestures." So there is no established prosodic convention to implement — the user would be inventing one. This is fine and can be principled by borrowing affective-prosody findings and mapping arousal/valence to F0 mean, F0 range, and rate.

## Details

### 1. Lojban phonology and phonetics (the ground truth)

**Consonants (canonical IPA, from CLL Table in §3.2):**

| Letter | IPA (preferred) | Variants | Description |
|---|---|---|---|
| b | [b] | | voiced bilabial stop |
| c | [ʃ] | [ʂ] | unvoiced coronal sibilant |
| d | [d] | | voiced dental/alveolar stop |
| f | [f] | [ɸ] | unvoiced labial fricative |
| g | [ɡ] | | voiced velar stop |
| j | [ʒ] | [ʐ] | voiced coronal sibilant |
| k | [k] | | unvoiced velar stop |
| l | [l] | [l̩] syllabic | voiced lateral approximant |
| m | [m] | [m̩] syllabic | voiced bilabial nasal |
| n | [n] | [ŋ], [n̩] | voiced dental/velar nasal |
| p | [p] | | unvoiced bilabial stop |
| r | [r] | [ɹ ɾ ʀ] + syllabic | any rhotic |
| s | [s] | | unvoiced alveolar sibilant |
| t | [t] | | unvoiced dental/alveolar stop |
| v | [v] | [β] | voiced labial fricative |
| x | [x] | | unvoiced velar fricative |
| z | [z] | | voiced alveolar sibilant |

Note the affricates tc [tʃ], dj [dʒ], ts [ts], dz [dz] are **consonant clusters, not phonemes** — synthesize them as stop+fricative sequences. `x` [x] has no voiced counterpart.

**Vowels (canonical IPA):** a [a]/[ɑ], e [ɛ]/[e], i [i], o [o]/[ɔ], u [u], y [ə]. The vowel y [ə] (schwa) appears only in names, as a lujvo glue vowel, in letter-names, and as a hesitation filler; **y is never stressed and is skipped in stress counting.**

**Special characters:** apostrophe `'` = [h] (voiceless glottal spirant; permitted variant [θ] or any unused unvoiced fricative; occurs only intervocalically); period `.` = mandatory pause, shortest realization glottal stop [ʔ]; comma `,` = syllable separator with no pause (may be realized as a glide [j]/[w] or [h], never as a pause).

**Diphthongs (16 total):** off-glides ai [aj], ei [ɛj], oi [oj], au [aw] (used freely); on-glides ia [ja], ie [jɛ], ii [ji], io [jo], iu [ju], ua [wa], ue [wɛ], ui [wi], uo [wo], uu [wu] (words/names/borrowings only); iy [jə], uy [wə] (names only). Diphthongs are always one syllable.

**Grapheme→phoneme mapping** is essentially the identity: each letter → its IPA above; `'`→[h]; `.`→pause/[ʔ]; `,`→syllable break (glide). Capitalization in cmevla marks non-standard stress on the capitalized vowel/syllable (e.g. `DJOsefin.` → [ˈdʒosɛfinʔ]).

**Permissible consonant clusters.** There are exactly **48 permissible initial consonant pairs** (CLL §3.7): bl br cf ck cl cm cn cp cr ct dj dr dz fl fr gl gr jb jd jg jm jv kl kr ml mr pl pr sf sk sl sm sn sp sr st tc tr ts vl vr xl xr zb zd zg zm zv. Medial "permissible pairs" are a larger set governed by three rules (CLL §3.6): no doubled consonant; no voiced+unvoiced mix (except l m n r, which are exempt); not both from {c j s z}; and the specific forbidden pairs cx kx xc xk mz. Consonant triples occur medially and must be C1C2 = permissible pair + C2C3 = permissible initial pair. A synthesizer's phoneme-to-parameter stage should be able to synthesize any legal cluster and can either reject or best-effort render illegal ones.

**Buffer vowels.** Speakers who cannot pronounce a cluster may insert a very short epenthetic buffer vowel (conventionally [ɪ] or a schwa-like sound distinct from the 6 phonemic vowels), pronounced as short as possible; it does not affect word recognition or stress. For a robotic synth this is optional and probably best omitted for cleanliness.

**Syllabification.** Each vowel (or diphthong, or syllabic consonant) is one syllable nucleus; consonants attach around nuclei. y and apostrophe are not counted for the consonant-cluster/stress purposes of word classification. Formal algorithms: the BPFK PEG morphology algorithm and the "informal description of the PEG morphology algorithm" on mw.lojban.org give exact rules; camxes/valfendi implement them.

### 2. Lojban prosody rules

**Penultimate stress (CLL §3.9, §4):** Primary stress is required on the penultimate syllable of brivla, counting only syllables whose nucleus is not y (thus `.erNAce`, `VEcnu`, `POFygau`, `BRIvla`). Because gismu are always CVCCV or CCVCV (two syllables), stress always lands on their first syllable. cmavo may be stressed on any syllable or none; two-syllable cmavo conventionally take stress on the first vowel, giving the language a consistent penultimate rhythm. Cmevla may be stressed anywhere; non-penultimate stress must be capitalized in writing. Constraint: a syllable immediately preceding a brivla must not carry primary stress unless a pause separates them (else the words run together).

**Pause rules (denpa bu, CLL §3.3, §4.9):** The period is a mandatory pause (shortest form = glottal stop [ʔ]). Pauses are **mandatory**: (a) before any word beginning with a vowel; (b) after any word ending in a consonant (i.e. after every cmevla); (c) before and after non-Lojban text; (d) after a Cy-form cmavo unless another Cy cmavo follows; (e) after a finally-stressed cmavo when a brivla follows. Pauses are **optional/permitted** between any two words otherwise. The comma never becomes a pause.

**Question/declarative intonation.** Lojban does not require intonation for questions — `xu` (yes/no) and `ma`/`mo` (fill-in) questions are lexical. CLL does not prescribe interrogative pitch. So declination (gradual F0 fall over a bridi) for statements is a reasonable default you invent; a mild terminal rise on `xu` questions is an optional, non-canonical nicety.

**Attitudinal pitch (the requested novel feature).** The primary attitudinals are the 39 cmavo of form VV or V'V (all 25 V'V pairs, the four diphthongs .ai .au .ei .oi, and ten more diphthongs). CLL §13/§2.16 is explicit that these are written-and-spoken words that "require no specific intonation or gestures" — emotion is lexical, not prosodic. **There is therefore no prior convention to follow**, only community recordings (e.g. la selpa'i's readings, discussed below) that a builder could listen to for taste. To add pitch treatment, map each attitudinal (and its scale via `nai` polar negation / `cu'i` neutral / intensity markers `sai`, `ru'e`, `cai`) onto an arousal–valence point and modulate the following/preceding scope (a lone attitudinal at bridi start scopes the whole bridi; elsewhere it scopes the word to its left).

Concrete parameter targets can be lifted from affective-prosody measurements. Tamuri's Estonian emotional read-out speech studies (2015) give directly usable multipliers and offsets relative to neutral speech:
- **Speech rate:** joy ≈ 1.1× (10% faster), sadness ≈ 0.9× (10% slower), anger ≈ 1.24× (24% faster).
- **Mean F0 / F0 range (Model 2):** joy ≈ +2.5 semitones mean F0 with F0 range +2.5 semitones; sadness ≈ −4 semitones mean F0 with range −1.15 semitones.
- **F0 range medians:** widest for anger and joy (105.1 Hz and 105.0 Hz respectively) and narrowest for sadness (89.1 Hz) — per Tamuri, "Fundamental Frequency in Estonian Emotional Read-Out Speech," *Journal of Estonian and Finno-Ugric Linguistics* 6(1), 2015.

Mapping: high-arousal positive (joy, `.ui`) → higher mean F0, wider range, faster rate, descending contours; low-arousal negative (sadness, `.u'u`) → lower mean F0, narrow range, slower rate, flat contour; anger/complaint (`.oi`) → high intensity, wide range, fast rate. Intensity suffixes scale the magnitude of the excursion.

### 3. Synthesis approaches (robotic acceptable, no Lojban audio data)

**Rule-based formant synthesis (RECOMMENDED core).** The Klatt 1980 cascade/parallel formant synthesizer (Klatt, *JASA* 67(3):971–995, 1980) is the canonical architecture: a voicing source (parameters F0, AV = amplitude of voicing) and a noise/aspiration source (AF, AH) feed a cascade branch (good for vowels/sonorants: resonators R1–R5 with formant frequencies F1–F5 and bandwidths B1–B5) and a parallel branch (good for fricatives and stop bursts: resonators with independent amplitudes A1–A6). Per the comp.speech Klatt 3.04 documentation (Iles & Ing-Simmons, implementing Klatt 1980): "Forty parameters are specified per frame ... Each frame of parameters usually represents 10ms of output speech." Open reference implementations to study/port: the original 1980 FORTRAN (github.com/jh4xsy/klatt80); Klatt's own C `klsyn`; the comp.speech `klatt 3.04` C reimplementation by Jon Iles and Nick Ing-Simmons; Christian d'Heureuse's clean TypeScript `klatt-syn` (github.com/chdh/klatt-syn), which is the most readable modern reference and directly informs a Rust port; and `rsynth`. eSpeak NG also has a Klatt option in addition to its default spectral-frame formant synthesis. Deriving Lojban formant targets from published data is straightforward given the tiny vowel set.

**Vowel formant targets (adult-male reference; derived from Peterson & Barney 1952 nearest categories):** Starting values in Hz to hand-tune for a robotic voice. Per a VoiceScience/Ladefoged summary of Peterson & Barney (1952): "For adult male speakers, F1 ranges roughly from 250 to 900 Hz depending on the vowel, F2 from 600 to 2,500 Hz, and F3 from about 1,500 to 3,500 Hz," with "typical F1/F2 values fall near 300/870 Hz for /u/, 710/1,100 Hz for /ɑ/, and 590/880 Hz for /ɔ/."

| Lojban vowel | IPA | F1 (Hz) | F2 (Hz) | F3 (Hz) | Nearest P&B token |
|---|---|---|---|---|---|
| i | [i] | 270 | 2290 | 3010 | /iy/ heed |
| e | [ɛ]/[e] | 530 | 1840 | 2480 | /ɛ/ head |
| a | [a]/[ɑ] | 730 | 1090 | 2440 | /ɑ/ hod |
| o | [o]/[ɔ] | 570 | 840 | 2410 | /ɔ/ hawed |
| u | [u] | 300 | 870 | 2240 | /u/ who'd |
| y | [ə] | 500 | 1500 | 2500 | schwa (central) |

(For a female/child or higher-pitched robot, scale all formants up ~15–20%. P&B recorded these vowels in hVd words — heed /iy/, hid, head /ɛ/, had, hod /ɑ/, hawed /ɔ/, hood, who'd /u/, hud, heard — from 76 men, women and children.) Consonant targets come from locus theory: bilabials (b p m f v) low F2 locus ~700–1000 Hz; alveolars/dentals (d t n s z) F2 locus ~1700–1800 Hz; velars (g k x ŋ) F2 locus ~2000–2300 Hz (velar pinch, F2≈F3); sibilants c/ʃ noise centered ~2500–3500 Hz, s noise ~4000–8000 Hz; x [x] noise ~1500–2500 Hz; h [h] = breathy aspiration through the following vowel's formants.

**Articulatory synthesis (Pink Trombone).** Pink Trombone models the vocal tract as a 1-D/2-D digital waveguide with an LF-style glottal source (Story 2005 area-function work underlies it). It has **already been ported to Rust**: the `pink-trombone` crate (crates.io/lib.rs/docs.rs, v0.2.1) and `lostmsu/pink-trombone`, converted from the TypeScript, connects to any f32 audio framework (rodio example). Feasibility for Lojban is high but control is harder: you'd need to script tract-area gestures per phoneme rather than set formants directly. More organic, less "robotic," and more effort to make intelligible for stops/fricatives. Reasonable as an experimental alternate voice, not the primary.

**Concatenative/diphone (MBROLA) without Lojban recordings.** MBROLA (github.com/numediart/MBROLA, engine AGPLv3 since 2018) is diphone concatenation; it needs a phoneme+prosody input and a diphone database. **No Lojban (jbo) MBROLA voice exists** (confirmed via search of MBROLA-voices and the mw.lojban.org synthesis page). You could in principle drive an existing language's diphone set (e.g. a Spanish or Esperanto voice whose phoneme inventory overlaps Lojban's cleanly) by mapping Lojban phonemes to the nearest diphones, as eSpeak does for its MBROLA front-ending. But MBROLA voice databases are cost-free only for non-commercial/non-military use and are **not open source**, and the missing /x/, /ʃ/, /ʒ/ coverage varies by voice. This conflicts with a clean, minimal, potentially-commercial Rust project and is not recommended.

**Small neural (Piper/VITS) with no Lojban training.** Piper (now OHF-Voice/piper1-gpl, v1.4.2 April 2026; VITS exported to ONNX, espeak-ng phonemizer) uses phoneme IDs, so in principle you could feed Lojban-phonemized IPA to an existing multilingual voice. But VITS voices are trained on a specific language's phoneme distribution; feeding Lojban phonemes to, say, an English voice produces an accented approximation, not correct Lojban, and quality on unseen phoneme sequences is unpredictable. Medium Piper voices are single ONNX files ~63 MB (~15M params), high ~114 MB (~28M params), producing 22.05 kHz. In-browser inference is possible via onnxruntime-web (WASM/WebGPU) or in Rust via `tract` (sonos/tract, pure Rust, WASM target, ~68 KiB core crate) or `ort` (pykeio/ort, ONNX Runtime bindings, WASM via tract/candle backend). A Lojban-specific neural voice (Pendrokar's `xvasynth_lojban`, trained on Lojban Corpus Readings of la selpa'i) does exist and shows it's *possible*, but training your own violates the "no training data" constraint, and 60–114 MB models plus multi-hundred-ms latency conflict with a minimal WASM aesthetic. Not recommended as the core; interesting as an optional high-quality mode later.

**eSpeak NG `jbo` specifically.** The engine is formant synthesis (small, fast, robotic — exactly the desired aesthetic). The exact Lojban logic, recovered from `src/libespeak-ng/tr_languages.c`:

```c
case L3('j', 'b', 'o'): // Lojban
{
    static const short stress_lengths_jbo[8] = { 145, 145, 170, 160, 0, 0, 330, 350 };
    static const wchar_t jbo_punct_within_word[] = { '.', ',', '\'', 0x2c8, 0 }; // allow period and comma within a word, also stress marker
    SetupTranslator(tr, stress_lengths_jbo, NULL);
    tr->langopts.stress_rule = STRESSPOSN_2R;   // penultimate (second-from-right)
    tr->langopts.vowel_pause = 0x20c;           // pause before vowel-initial word / after consonant-final word
    tr->punct_within_word = jbo_punct_within_word;
    tr->langopts.param[LOPT_CAPS_IN_WORD] = 1;  // capitals indicate stressed syllables
    SetLetterVowel(tr, 'y');
    tr->langopts.max_lengthmod = 368;
    tr->langopts.numbers = 0;                    // numbers disabled pending _list completion
}
break;
```

Key takeaways: stress rule is `STRESSPOSN_2R` (penultimate); apostrophe is inside `punct_within_word` and its `jbo_rules` maps `'`→[h] (verified in eSpeak's own test output where `ce'u`→`S,ehu`); the vowel-pause flag `0x20c` implements exactly the CLL mandatory-pause rules; capitalization drives stress; **numbers are disabled** (a known incompleteness). The phoneme table (`ph_lojban`) is built on Esperanto (`eo`) plus a dedicated file that overrides `e`. jbo is credited to Juho Hiltunen and xunsku, and eSpeak's own docs class it among "initial naive implementations." eSpeak NG is GPLv3 — wrapping it means your product inherits GPL. It compiles to WASM via Emscripten (espeak-ng/emscripten, ianmarmour/espeak-ng.js, echogarden's build; Chrome OS ships a WASM+AudioWorklet build), so a working in-browser Lojban TTS is achievable in a day using the existing `jbo` voice.

### 4. Rust + WASM audio implementation

**DSP/synthesis crates:**
- **fundsp** (v0.23, MIT/Apache): composable graph-notation audio DSP, zero-cost typed graphs, supports `no_std` (disable default `std` feature; still needs `alloc`), 32-bit and 64-bit preludes. Excellent for building the resonator/source network declaratively; WASM-compatible. Best fit for expressing a Klatt cascade/parallel network.
- **dasp** (RustAudio, formerly `sample`): low-level PCM/DSP fundamentals, no dynamic allocation, no dependencies, `no_std` (needs nightly `core_intrinsics` for trig in no_std). Ideal for the sample-frame plumbing and format conversion. (Note: the unrelated `dasp-rs` crate requires OpenBLAS and is not the same thing — avoid confusing them.)
- **hound**: simple WAV read/write — use for native file export and test fixtures.
- **cpal**: cross-platform native audio output (the native adapter).
- **oddio**, **knyst**, **glicol**, **synfx-dsp**: higher-level engines/frameworks. glicol is a good reference for a dual-target (native + WASM AudioWorklet) Rust audio architecture. synfx-dsp has ready filters/oscillators.

**wasm-bindgen / AudioWorklet pattern.** The standard, working approach (per the wasm-bindgen guide's `wasm-audio-worklet` example, Paul Batchelor's `rust-wasm-audioworklet`, Casey Primozic's wavetable-synth writeup, and the `waw-rs` framework): compile the core synth crate to WASM (cdylib), instantiate the WASM module *inside* an `AudioWorkletProcessor`, and fill output buffers from Rust in `process()`. Two sub-patterns: (a) message-passing (post phoneme/parameter frames to the worklet; simplest, no special headers) or (b) SharedArrayBuffer for zero-copy control, which **requires cross-origin isolation** — serve with `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp`. Threaded WASM (atomics/bulk-memory) additionally needs a nightly toolchain rebuild of std with `-C target-feature=+atomics,+bulk-memory,+mutable-globals`. For a single-voice TTS, **message passing without SharedArrayBuffer is sufficient and avoids the COOP/COEP deployment burden** — synthesis is cheap enough to run single-threaded in the worklet. `waw-rs` (Marcel-G/waw-rs) provides a clean `Processor` trait that hides most of the glue. Note historical caveat: older writeups mention Firefox lacking AudioWorklet, but modern Firefox/Chrome both support it.

**Dual-target architecture.** Design the core as a platform-agnostic, pull-based sample generator: a `Synth` struct with `fn render(&mut self, out: &mut [f32], sample_rate: f32)` and no I/O or platform deps, ideally `no_std + alloc`. Then thin adapters: cpal callback (native) and AudioWorkletProcessor (web) both just call `render`. This mirrors glicol and typical Rust synth projects and matches the user's modular preference.

**Performance/size.** Formant synthesis at 44.1/48 kHz is trivially real-time (it was real-time on 1980s DSPs); a single voice is microseconds per buffer on any modern CPU/WASM. Binary size: a pure formant engine with fundsp/dasp compiles to a small WASM (tens to low-hundreds of KB after `wasm-opt`), versus 60–114 MB for a neural voice. Use `wasm simd128` (stable in Rust via `-C target-feature=+simd128` and `core::arch::wasm32`) for the resonator inner loops if needed, though it likely isn't for one voice. Prefer the default allocator over wee-alloc (known corruption reports).

**WIT / component model.** The WASM Component Model + WASI could express a clean TTS interface (`text -> stream<f32>` or `text -> list<f32>`), which fits the user's WIT-based engine. However, **Web Audio has no component-model binding today** — browsers consume plain wasm-bindgen/JS-glue modules, and AudioWorklet expects a JS class. Practical recommendation: ship the browser build with wasm-bindgen now; optionally *also* define a WIT interface and build a `wasm32-wasip2` component for server-side/native/embedding use (driven by the same core crate). Keep the core crate free of both wasm-bindgen and WIT so both adapters are thin.

### 5. Pipeline design

**Text normalization.** Numbers must be read as digit-cmavo strings: no[0] pa[1] re[2] ci[3] vo[4] mu[5] xa[6] ze[7] bi[8] so[9], concatenated (pano = 10), pi = decimal point, ki'o = comma/thousands. (This is exactly the gap eSpeak's `jbo` leaves open with `numbers = 0`.) Letterals (bu-suffixed, e.g. `.abu` for "a") read as their letter-names. `zoi`-delimited foreign quotations must be bracketed by pauses and either spelled out, passed to a fallback voice, or read literally per your policy. `la'o` handles non-Lojban names similarly.

**Tokenization/morphology.** A TTS needs word-boundary detection and word-type classification (cmavo vs brivla vs cmevla) to place stress and pauses — it does **not** need full syntactic parsing. Lojban's self-segmenting morphology (valfendi/vlatai algorithms; the BPFK PEG morphology) makes this decidable from spelling alone: cmevla end in a consonant; brivla contain a consonant cluster in the first five letters (ignoring y and ') and end in a vowel; cmavo lack the cluster. Existing tools: camxes (Java/PEG), la ilmentufa / la zantufa (JS PEG), jbofihe (C), valfendi (phma, the baseline lexer), and in Rust: **camxes-rs** (github.com/lojban/camxes-rs, a PEG parser generator with a Lojban grammar) and a crate providing a "Lojban PEG parser with semantic analysis — integrated camxes parser and tersmu semantic engine." Since the user's neuro-symbolic engine (nibli) already uses Lojban as its inference language, it very likely already has a parser — the TTS should consume its word-type/boundary output and only implement the lightweight classifier as a fallback.

**Phoneme→parameter mapping.** Build a table keyed by phoneme with: steady-state formant targets (F1–F3 + bandwidths for vowels/sonorants; noise spectrum center/amplitude for fricatives; burst spectrum + closure for stops), intrinsic duration (ms), voicing flag, and manner. Apply locus-theory transitions: interpolate F2 (and F1/F3) between a consonant's locus and the adjacent vowel target over a 30–60 ms transition, giving coarticulation approximation. For stops, insert a closure silence + burst. For [h] and apostrophe, run aspiration noise shaped by the following vowel's formants. Diphthongs = glide from first to second vowel target over the syllable.

**Prosody layer.** Implement in order: (1) syllabify and mark the penultimate non-y syllable of each brivla/cmevla as primary-stressed (or the capital-marked syllable in cmevla); (2) realize stress as duration increase (~1.5×), an F0 excursion (+10–30 Hz peak), and slight amplitude boost; (3) insert pauses per the denpa bu rules (mandatory before vowel-initial words, after consonant-final words, around non-Lojban text, after Cy cmavo); (4) apply sentence-level declination (linear F0 fall, e.g. 120→95 Hz across a bridi) for statements; (5) optional mild terminal rise for `xu`; (6) apply the attitudinal pitch overlay: when an attitudinal is encountered, look up its arousal/valence, and modulate F0 mean/range/rate and contour over its scope (whole bridi if bridi-initial, else the preceding word), scaled by any intensity marker, using the Tamuri multipliers in §2 as concrete starting values.

### 6. Prior art specific to Lojban speech

- **eSpeak NG `jbo`** — the only mature, general Lojban TTS; formant synthesis; details in §3 above; runs via `espeak-ng -v jbo`, in-browser at eeejay.github.io/espeak/emscripten, and the sourceforge GUI.
- **Pendrokar `xvasynth_lojban`** (Hugging Face) — a neural (xVASynth) Lojban voice of la selpa'i, trained on "Lojban Corpus Readings"; demo at huggingface.co/spaces/Pendrokar/xVASynth. Useful as a naturalness reference and evidence that Lojban neural TTS works, but requires the training data you're avoiding.
- **lojban_diphone_speech_synthesizer** — a Praat + Festival/FestVox diphone project (by "Xavier," ~2005); artifacts include `ljbdiph.list`, `toi_ljb_phones.scm`, diphone wav sets; **most external links are now dead**. Historical interest only. (Not MBROLA-based, contrary to some secondary claims.)
- **mw.lojban.org "Lojban Speech Synthesis"** page aggregates the above.
- **Speech recognition / phonetic documentation**: the BPFK PEG morphology algorithm, "distinctive features in Lojban phonology" (mw.lojban.org), and CLL chapters 3–4 are the authoritative phonetic references for tuning (not training). No significant Lojban ASR corpus exists.
- **Reference audio for tuning:** recordings by fluent speakers (la selpa'i, and community readings of CLL examples) can be listened to for prosodic taste, but should be used only to calibrate parameters by ear, never as training data.

### 7. Recommended architecture (three options)

**Option A — Pure-Rust Klatt-style formant synth with hand-tuned Lojban table (RECOMMENDED).**
- *Stack:* core `no_std+alloc` crate using fundsp (resonator graph) + dasp (frame plumbing); hound for WAV tests; cpal native adapter; wasm-bindgen + AudioWorklet (waw-rs) web adapter. Optional wasm32-wasip2 + WIT component for the engine.
- *Data:* phoneme table (~24 phonemes) with formant/noise/duration params seeded from Peterson & Barney 1952 + Klatt 1980 + locus theory; 48 initial-pair table; stress/pause rules from CLL §3–4.
- *Complexity:* medium. The synth engine is the main effort (port/adapt chdh/klatt-syn logic to Rust, ~1–2k LOC); the linguistic front-end is small (~500 LOC) and can reuse the existing nibli parser.
- *Pros:* full control incl. the novel attitudinal pitch; deliberately robotic; tiny WASM; no training data; no GPL (your license); clean dual-target; matches user's minimal/modular style.
- *Cons:* you tune formants by ear; intelligibility depends on that tuning.
- *Refs:* Klatt, *JASA* 67(3):971–995 (1980); chdh/klatt-syn; comp.speech klatt 3.04; fundsp; waw-rs; CLL ch.3–4.

**Option B — eSpeak NG `jbo` compiled to WASM (fastest to working audio; use as reference/fallback).**
- *Stack:* espeak-ng Emscripten build (strip mbrola/sonic/klatt/speechplayer), AudioWorklet output; or ianmarmour/espeak-ng.js.
- *Complexity:* low (hours). Working Lojban audio immediately.
- *Pros:* proven `jbo` voice, correct stress/pause/apostrophe out of the box; robotic aesthetic.
- *Cons:* C dependency; **GPLv3** (viral licensing); little control; numbers unimplemented (`numbers=0`); no attitudinal pitch; not idiomatic Rust; harder to extend.
- *Verdict:* build this first as a baseline and regression oracle, then supersede with Option A.

**Option C — Rust reimplementation driven by eSpeak's jbo rules feeding a Rust Klatt engine (middle ground).**
- *Stack:* port the tiny `jbo` front-end logic (STRESSPOSN_2R stress, vowel_pause 0x20c, apostrophe→h, caps→stress) and `ph_lojban`/`eo` phoneme intents into Rust rules, driving the Option-A engine.
- *Complexity:* medium-high (you reimplement both front-end and engine, but the front-end is a faithful port of well-understood C).
- *Pros:* correctness inherited from eSpeak's tested rules; full Rust; no GPL if you reimplement from the CLL spec rather than copying eSpeak's data files.
- *Cons:* most total work; risk of accidentally deriving from GPL source.

**Overall recommendation:** Ship **Option B in week 1** as a working fallback and correctness oracle, then build **Option A** as the real product — a small, modular, pure-Rust formant synthesizer with a hand-tuned Lojban phoneme table, dual-targeted to native (cpal) and browser (wasm-bindgen + AudioWorklet), with the attitudinal-pitch prosody layer as the differentiating feature. Reimplement all linguistic rules from the CLL specification (not from eSpeak's GPL data) to keep licensing clean.

## Recommendations
1. **Week 1 — baseline:** Compile espeak-ng with only `jbo` to WASM, wire to an AudioWorklet, confirm end-to-end browser audio. Capture its output on a test sentence set (CLL examples, `coi rodo`, number-heavy strings) as a regression oracle. Benchmark WASM size and latency.
2. **Weeks 2–4 — core engine:** Build the `no_std+alloc` formant core in Rust (fundsp graph: voicing source + noise source + cascade R1–R5 + parallel branch). Port chdh/klatt-syn's parameter logic. Validate single vowels against the formant table, then CV syllables, then clusters.
3. **Weeks 3–5 — front-end:** Implement syllabifier, penultimate-stress marker (skip y), the 48-initial-pair table, and the denpa bu pause rules. Reuse the nibli Lojban parser for word-type/boundary info; add a standalone classifier fallback. Implement number→digit-cmavo normalization (the gap eSpeak leaves).
4. **Weeks 5–6 — prosody + attitudinals:** Add declination, stress realization (duration/F0/amplitude), and the attitudinal arousal/valence→F0-mean/range/rate overlay with intensity-marker scaling (seed from Tamuri: joy rate ×1.1 / +2.5 st mean F0 / range 105 Hz; sadness ×0.9 / −4 st / range 89 Hz; anger ×1.24). Calibrate by ear against fluent-speaker recordings (la selpa'i) — listening only, no training.
5. **Adapters:** cpal native binary + wasm-bindgen/AudioWorklet web build (message-passing, no SharedArrayBuffer initially to avoid COOP/COEP). Optionally add a WIT/`wasm32-wasip2` component sharing the same core.
6. **Decision thresholds:** If hand-tuned formant intelligibility is unacceptable after calibration, escalate to (a) porting Pink Trombone (`pink-trombone` crate) for a more organic voice, or (b) a neural Lojban voice via tract/ort running an ONNX VITS model — but only if you accept a ~60 MB model and relax the no-training-data constraint (or reuse Pendrokar's existing Lojban model). If WASM size/latency of any neural path exceeds your budget, stay with formant synthesis.

## Caveats
- **Formant targets are seed values, not ground truth for Lojban.** The Peterson & Barney figures are American-English vowels in /hVd/ context; Lojban's /a e i o u/ are cardinal-ish and will need by-ear adjustment. Treat the table as a starting point.
- **The attitudinal-pitch feature is an invention, not a standard.** CLL is explicit that Lojban attitudinals carry no required intonation; any mapping you build is your own convention and may feel unnatural to purists. The Tamuri multipliers are from Estonian read-out speech, not Lojban, and are only a principled starting point. Make the feature toggleable.
- **Licensing:** eSpeak NG and MBROLA are GPLv3/AGPLv3 and non-commercial respectively; do not copy their data/source into a permissively-licensed Rust project. Reimplement from the public CLL specification.
- **eSpeak's `jbo` is officially "naive"** and its number handling is disabled; don't treat its output as fully authoritative — validate stress/pause against CLL rules directly.
- **Neural in-browser figures** (63–114 MB Piper models, 22.05 kHz) come from vendor/community sources and describe English voices; a Lojban neural voice's quality and size would depend on its own training and is out of scope for the no-data constraint.
- Some community diphone/TTS resources (lojban_diphone_speech_synthesizer, older members.home.nl pages) have dead links and are unverifiable today.