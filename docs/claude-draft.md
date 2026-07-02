# Lojban Robotic TTS in Rust/WASM — Verified v2 Technical Report

## TL;DR
- Build a pure-Rust, no-training-data formant synthesizer as the core; adopt the newly-verified, MIT-licensed, realtime-safe **`klattsch-core`** (Klatt-1980 parallel-formant engine) as the starting point instead of porting `chdh/klatt-syn`, but hand-roll the Lojban linguistic layer from the CLL.
- On the web, do **not** rely on cpal's AudioWorklet backend (it requires SharedArrayBuffer + COOP/COEP + nightly `-Zbuild-std`); use a hand-written single-threaded `AudioWorkletProcessor` that instantiates the WASM module *inside* the worklet scope, and use cpal only on native.
- Round-1 phonology stands with one refinement: syllabic-consonant syllables **and** y-syllables **and** buffer-vowel syllables are all excluded from stress counting (CLL §3.9, verbatim); Lojban has exactly **16 diphthongs and no triphthong units**; default to classic CLL pause rules with an optional "dotside" flag.

## Key Findings

### Corrections carried forward (adjudicated in round 1, restated for the record)
- Lojban has **17 consonants**, not 21. (The Wikipedia "Lojban grammar" article still says "6 vowels and 21 consonants," which is wrong; the CLL inventory is b c d f g j k l m n p r s t v x z = 17, with the apostrophe = [h] treated separately.)
- Threaded-WASM rustc flags (`+atomics,+bulk-memory,+mutable-globals`) are **not** required for a single-threaded, message-passing AudioWorklet design.
- A wasm-bindgen module instantiated on the main thread **cannot** be called from inside an `AudioWorkletProcessor`. The WASM instance must be created inside the worklet global scope — transfer the compiled `WebAssembly.Module` (or the bytes) via `processorOptions`/`port.postMessage` and instantiate in the processor. (Confirmed directly by the klattsch-wasm README, below.)

### 1. klattsch ecosystem — VERIFIED and RECOMMENDED
The klattsch ecosystem is real, active, and a strong fit.
- **`tgies/klattsch`** (JS): a "primitive parallel-formant speech synth in the browser," ~278 stars, by Tony Gies (Crash United, LLC). Zero runtime dependencies; ships as an npm package usable in Node, browser (AudioWorklet), and CLI. Voiced source = Rosenberg-style glottal pulse with tunable open/closed quotient; unvoiced = xorshift noise; three parallel bandpass biquads per formant; schedule-based prosody; vibrato, aspiration/breathiness, spectral tilt, glottal effort all controllable. It explicitly cites Klatt (1980). Self-described as "Late-70s / early-80s tier (Votrax, SAM)" — i.e., exactly the robotic aesthetic requested.
- **`tgies/klattsch-rs`** (Rust): a Rust port, 11 stars, 3 commits, MIT-licensed, MSRV 1.77, parity-tested against klattsch JS 0.3.0 via committed golden WAVs. Workspace of four crates:
  - **`klattsch-core`** — synthesis engine; `FormantSynth::process` is realtime-safe (no alloc, no locks, no I/O in the render path; `new`/`reset`/`queue_schedule` allocate off-thread). Modules: `dsp` (bandpass biquad, Rosenberg glottal pulse, 32-bit xorshift LFSR, soft-clip), `params`, `phonemes` (ARPABET table + `PhonemeTable` trait), `schedule`, `synth`. Optional deps `rtrb ^0.3` and `serde`. A `live-events` feature exposes an `event_channel` for realtime parameter updates. crates.io v0.1.1, MIT, ~50 KB / 1.5K SLoC, 42.2% documented.
  - **`klattsch-text`** — parser/schedule compiler for `.klatt` phoneme strings (ARPABET).
  - **`klattsch-wav`** — RIFF/WAVE encoder.
  - **`klattsch-wasm`** — wasm-bindgen build with an AudioWorklet shim. Its README documents exactly the correct worklet pattern: `const wasmModule = await WebAssembly.compileStreaming(fetch('./pkg/klattsch_wasm_bg.wasm')); new AudioWorkletNode(ctx, 'klattsch-formant-processor', { processorOptions: { wasmModule, text } })` — "The worklet uses synchronous wasm init; compile the module on the main thread and pass it via `processorOptions`." Live updates via `node.port.postMessage` with `frame`/`schedule`/`compile`/`reset` messages; target field names match the engine's PARAMS array (`F0`, `voicing`, `F1`, `BW1`, `A1…`, `vibratoDepth`, `vibratoRate`, `tremoloDepth`, `tremoloRate`, `aspiration`, `tilt`, `effort`, `gain`).

**Verdict:** `klattsch-core` implements a **parallel-only** resonator model (three parallel bandpass biquads), not a full cascade/parallel Klatt. For Lojban's clean 5-vowel + [ə] system and a deliberately robotic voice, parallel-only formant synthesis is sufficient and is easier to keep realtime-safe. It is a **better starting point than porting `chdh/klatt-syn`** (TypeScript) because it is already Rust, already MIT, already realtime-safe, already has a WASM+AudioWorklet story matching our architecture, and already has a `PhonemeTable` trait designed for exactly the swap we need. The plan: implement a `LojbanTable: PhonemeTable`, feed it schedules compiled by our own CLL front-end, and bypass `klattsch-text`'s ARPABET parser. I found no maintained, realtime-safe crate literally named `klatt` on crates.io comparable to klattsch-core; the klattsch family is the most mature Rust option.

### 2. cpal WASM backend — SETTLED: custom worklet on web, cpal on native
Current cpal has two WASM paths:
- Default **`wasm-bindgen`** backend: a generic Web Audio backend that schedules audio via rotated `AudioBufferSourceNode`s (PR #372). It works on Firefox/Chrome but is a scheduling hack; the PR author estimated a latency floor "on the order of at least 100 ms" and a WIP lower bound of 0.33 s. This is **not** an AudioWorklet.
- **`audioworklet`** feature backend: real AudioWorklet, but the docs are explicit — it "additionally requires `-Zbuild-std` with atomics support enabled," `RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals"`, **Rust nightly**, and the web server must send **Cross-Origin headers for SharedArrayBuffer**. cpal MSRV for the audioworklet WASM path is nightly.

So "one cpal codebase for native + web" forces you into either the high-latency `AudioBufferSourceNode` hack or the SharedArrayBuffer/COOP-COEP/nightly threaded path. Neither is desirable for a single-voice robotic TTS in a message-passing model.

**Decision:** Platform-agnostic realtime-safe core (klattsch-core-style `process(&mut [f32])`), with:
- **Native:** cpal (stable Rust, Apache-2.0), callback pulls from an `rtrb` SPSC queue.
- **Web:** a hand-written `AudioWorkletProcessor` (~40 lines of JS glue) that instantiates the WASM core inside the worklet global scope and calls `process()` per 128-frame render quantum. No SharedArrayBuffer, no COOP/COEP, no nightly, no `-Zbuild-std`. This is exactly what klattsch-wasm already does, and what `waw-rs` automates. Message-passing (parameters/schedules over `port.postMessage`) means there is no cross-thread Rust at all on the web.

### 3. Syllabic consonants and stress counting — VERIFIED, round-1 refined
CLL §3.9 (and §3.4), verbatim: *"Most Lojban words are stressed on the next-to-the-last, or penultimate, syllable. In counting syllables, however, syllables whose vowel is y or which contain a syllabic consonant (l, m, n, or r) are never counted... Similarly, syllables created solely by adding a buffer vowel, such as [ɪ], are not counted."* And: *"Weak stress is required for syllables containing y, a syllabic consonant, or a buffer vowel."*

The external reports are correct and round 1 was incomplete: the stress-counting exclusion set is **{y-syllables, syllabic-consonant syllables (l/m/n/r as nucleus), buffer-vowel syllables}** — all three, not just y. Algorithm change: when locating the penultimate stressed syllable, first build the list of *countable* nuclei (proper vowels + diphthongs that are NOT y, NOT syllabic consonants, NOT epenthetic buffers), then stress the penultimate of that filtered list. Note §4.1: "Syllabic l, m, n, and r always count as consonants for the purposes of [morphology]" — the syllabic-consonant nucleus counts as a consonant for word-shape rules but is excluded from *stress counting*.

### 4. Triphthongs — RESOLVED: none exist as units
Round 1 is correct. CLL §3.4: *"There exist 16 diphthongs in the Lojban language... Diphthongs always constitute a single syllable."* CLL §4.1 confirms VV = diphthong or apostrophe-separated pair only. Sequences of three vowels are pronounced as paired diphthong + vowel across syllables ("When more than two vowels occur together in Lojban, the normal pronunciation pairs vowels..."). The Wikipedia "Lojban grammar" claim that "Triphthongs exist as combinations of a rising and a falling diphthong, e.g. <iau>" is **not** CLL doctrine — <iau> is realized as glide+diphthong (i + au) spanning the onset+nucleus of one syllable, not a defined triphthong phoneme. The Linguifex wiki agrees: "There are 16 diphthongs (and no triphthongs)." **Implement exactly 16 diphthongs; no triphthong table.**

### 5. The Dot Side — RESOLVED: default to classic CLL, offer a dotside flag
The Dot Side (originally by Robin Lee Powell) is a community convention: *"All cmevla in all context require a period or space (in writing) or a glottal stop or pause (in speech) both before and after them, regardless of grammatical context. You can ignore anything about la, lai, or doi being impermissible inside cmevla."* Two concrete changes: (1) cmevla may freely contain la/lai/la'i/doi; (2) each cmevla must be preceded by a pause as well as followed by one. It does **not** change morphology or word-parsing rules (confirmed by xorxes in the lojban list: "there is no change at all in the morphology rules"); it is an error-detection aid. It is a popular unofficial change but not mandated by BPFK.

For TTS this is nearly a no-op at the *audio* level: classic CLL already requires a pause after every cmevla (consonant-final) and before every vowel-initial word, and dotside only adds a mandatory *leading* pause before cmevla — which classic rules already produce in almost all contexts (a cmevla is nearly always preceded by `la`/`doi`/another consonant-final word or is sentence-initial). **Default: classic CLL pause placement. Provide a `--dotside` flag that forces a leading pause before every cmevla and disables the la/lai/doi substring warnings in the tokenizer.**

### 6. Neural fallback sizing — RECONCILED
- Piper/VITS voices span roughly **22 MB (int8-quantized) to 75 MB (float32)**: per Grokipedia's Piper page, Piper "features compact model sizes ranging from 22 MB for int8 quantized versions to 75 MB for float32 models"; medium voices are "a single ONNX file in the tens of megabytes." The 63 MB (medium) / 114 MB (high) figures from a prior report are plausible for specific float32 voices but are not a floor; the 15–30 MB "quantized VITS" claim is validated by the 22 MB int8 figure.
- **x-low/low:** there are effectively no separate x-low English checkpoints. A Piper maintainer states verbatim on the rhasspy/piper-checkpoints discussion #8: *"I've noticed that the low-quality models are actually of medium size, the only difference is it's trained on data preprocessed with 16khz resolution, so they can only be trained with --quality 'medium'. There wasn't enough of a difference in performance for me to spend time training x-low quality versions."* So do not expect a dramatically smaller "x-low" ONNX.
- **int8 quantization works** for VITS/Piper ONNX in browser runtimes; onnxruntime-web docs explicitly recommend "prefer uint8 quantized models" for the WASM EP.
- **In-browser latency (2025–2026):** onnxruntime-web WASM EP is the portable CPU path (all ONNX ops supported; WebGPU supports only a subset and pays a CPU↔GPU transfer cost that makes it *slower* for small single-pass models). Multi-threaded WASM needs crossOriginIsolated (COOP/COEP). Piper hits real-time on a Raspberry Pi 5 CPU and ~10× real-time on a modern desktop CPU natively; in-browser WASM is slower but small VITS voices remain usable. `tract-wasm` is a pure-Rust alternative but generally slower with narrower op coverage than onnxruntime-web for VITS.
- **Cross-lingual host voice:** eSpeak's jbo voice inherits Esperanto phonemes. For a neural fallback, a Spanish or Italian VITS voice is a good cross-lingual host (pure 5-vowel system, native [x], trilled r) — a public-domain Spanish Piper voice exists (`friyin/vits-piper-es_ES-carlfm-high`, trained from scratch, public domain). There is no first-party Esperanto Piper/VITS voice in the rhasspy set I could confirm; RHVoice ships Esperanto voices but RHVoice is a different (statistical) engine, not VITS. **Recommendation: if a neural fallback is ever built, host on a permissively-licensed Spanish VITS voice with a custom Lojban phoneme map — but this is explicitly out of scope for the no-training-data core.**

### 7. VoiRS — VERIFIED, largely NOT reusable
`cool-japan/voirs` is real: a pure-Rust neural TTS/ASR/sound framework (beta 0.1.0-beta.1, 2026-02-26) on the cool-japan SciRS2/NumRS2 stack, VITS + DiffWave/HiFi-GAN, Kokoro-82M ONNX support (9 languages, 54 voices), Apache-2.0, builds for wasm32-unknown-unknown. But it is a **neural** framework: every path needs trained models (Kokoro is 86 MB quantized). Its `voirs-g2p` crate (multi-backend G2P, 20+ languages, IPA output) does not target Lojban and pulls in the heavy SciRS2 stack. **Not reusable for a no-training-data formant engine; note only as prior art and a possible neural-fallback host.**

### 8. web-audio-api-rs (orottier) — VERIFIED, native-only if at all
`orottier/web-audio-api-rs` is mature (v1.2.0, 331+ stars, MIT), a pure-Rust implementation of the W3C Web Audio API for **non-browser** contexts (it powers IRCAM's node-web-audio-api bindings). It uses cpal for cross-platform I/O by default (experimental cubeb backend). Recent versions renamed `AudioProcessor` → `AudioWorkletProcessor` and `RenderScope` → `AudioWorkletGlobalScope` and added message-port functionality to `AudioWorkletNode`/`AudioWorkletProcessor` — i.e., it now offers an AudioWorklet-equivalent processing model natively. **Assessment:** using it on native to mirror browser Web Audio semantics is defensible if you want one graph abstraction across native + a future node.js target, but it is heavier than "cpal + our own `process()`." For a single-voice robotic TTS, prefer the thin cpal adapter; keep web-audio-api-rs in reserve if you later want a node.js/Deno server-side renderer mirroring the browser graph.

### 9. rtrb and realtime-safety — CONFIRMED with scope note
`rtrb` (mgeier) is a wait-free SPSC ring buffer, MIT/Apache-2.0, `no_std`-capable (needs `alloc`), MSRV 1.38, purpose-built for the audio thread (it originated from the crossbeam SPSC PR and is endorsed on the Rust Audio forum). It is the right crate for passing parameters/schedules into a **native cpal callback**. Alternatives: `ringbuf` (agerasev) is also lock-free SPSC, supports `no_std` *without* `alloc` (static storage), overwriting mode, async — slightly more featureful; `heapless::spsc` is fixed-capacity for embedded. klattsch-core already optionally depends on `rtrb ^0.3`, so standardizing on rtrb aligns with the chosen engine.

**Scope note (important):** rtrb is only needed where there is a real cross-thread Rust boundary — i.e., the **native cpal callback thread ↔ control thread**. In the browser message-passing worklet design, parameters arrive via `port.postMessage` (JS→worklet) and are applied inside the single worklet thread; there is no second Rust thread, so **rtrb is not needed in the wasm32 build**. Only if you switch to threaded WASM (SharedArrayBuffer) would you reintroduce it.

### 10. LPC as a middle paradigm — REFUTED
Round 1 stands. Rule-driven LPC without recorded speech reduces to specifying spectral envelopes (LPC coefficients) by hand, which is strictly harder and less perceptually meaningful than specifying formant frequencies/bandwidths directly. LPC's advantage is *analysis* — deriving coefficients from recorded audio — which we forbid (no training/recorded data). LPCNet needs a trained neural network and is irrelevant to the no-data constraint. **Formant synthesis is strictly more controllable for this use case; do not pursue rule-driven LPC.**

### 11. Attitudinal DSP mapping — grounded in affective-vocalization literature
CLL explicitly states attitudinals require no specific intonation, so any pitch/voice-quality treatment is an invented but principled convention. Grounding:
- **Voice quality communicates affect:** Christer Gobl & Ailbhe Ní Chasaide, "The role of voice quality in communicating emotion, mood and attitude," *Speech Communication* 40(1–2):189–212 (2003). Their and Yanushevskaya et al.'s stimuli (Yanushevskaya et al. 2013, PMC3684800) were "generated by manipulating a set of the KLSYN88a parameters" starting from a modal LF-model inverse-filtered utterance. The LF→KLSYN88a mapping used: **EE→AV** (excitation strength → amplitude of voicing), **RA→TL** (return phase → spectral tilt), **RG→SQ** (glottal frequency → speed/skew), **RK→OQ** (skew → open quotient), plus **DI** (diplophonia) and **AH** (aspiration) used directly.
- Klatt (1980) cascade/parallel architecture: Dennis H. Klatt, "Software for a cascade/parallel formant synthesizer," *JASA* 67(3):971–995 (March 1980). KLSYN88 source: D.H. Klatt & L.C. Klatt, "Analysis, synthesis, and perception of voice quality variations among female and male talkers," *JASA* 87(2):820–857 (February 1990), adds **OQ** (open quotient, %), **FL** (F0 flutter, % of F0), **TL** (spectral tilt, dB), **DI** (diplophonia/double-pulsing, %), **AH** (aspiration, dB), **AV** (voicing amplitude, dB). Cascade branch = 5 formants (F1–F5, B1–B5) at 10 kHz for a 17-cm tract.
- Correlates from the literature: **breathy** = loosely closed glottis, strong F0, weak upper harmonics, audible aspiration (OQ↑, TL↑, AV↓, AH↑; AH is the single most important breathiness cue per Klatt & Klatt 1990; B1↑ also contributes); **creaky/vocal fry** = double-pulsing/period-doubling, low irregular F0, associated with utterance-final and low-pitched positions (DI↑, OQ↓, low F0); **tense/pressed** = abrupt complete closure, strong harmonics, associated with lexical stress and high pitch (OQ↓, AV↑, TL↓); **harsh** = tense + irregularity (DI↑ + F0 perturbation on a constricted source); **vibrato/jitter** associated with fear/nervousness (F0 sine modulation + FL↑); **monotone/flattened contour** associated with calm/patience. Yanushevskaya's loudness scaling factors: whispery 0.43, tense 1.43, harsh 1.35.
- **Estonian emotional-speech quantitative seeds** (Kairi Tamuri): per Tamuri, "Fundamental frequency in Estonian emotional read-out speech," *JEFUL* 6(1):9–21 (2015): *"F0 was highest for joy and lowest for anger. The F0 range, however, was widest for anger and narrowest for sadness."* Rate ranking (Tamuri & Mihkla, "Emotions and speech temporal structure," *Linguistica Uralica* 3:209–217, 2012): **anger > joy > neutral > sadness**. Parametric TTS model values (Tamuri 2015, Model 2): rate anger ×1.24, sadness ×0.9, joy ≈×1.1; mean F0 joy +2.5 st and range +2.5 st; sadness −4 st and range −1.15 st; emotional speech 6–45% quieter than neutral.

This yields a defensible per-attitudinal parameter table (see Details).

### 12. Buffer vowels — DEFAULT OMIT, optional flag
CLL frames buffer vowels ([ɪ]-like, "as short as possible," never stressed, excluded from counting) as a speaker-side accommodation for those who cannot pronounce a cluster. A synthesizer renders all legal clusters exactly, so inserting buffers adds ambiguity risk (a buffer could be misheard as a vowel and shift word boundaries). Community guidance frames buffering as optional/personal. **Default: omit buffer vowels; provide an optional `--buffer` flag that inserts a very short [ɪ] (excluded from stress/counting) between cluster consonants for maximum clarity in noisy conditions.**

### 13. Number/lerfu normalization — CONFIRMED complete
- Digit cmavo: **no pa re ci vo mu xa ze bi so** (0–9), plus **dau fei gai jau rei vai** (hex A–F), **pi** (decimal point), **ki'o** (thousands/comma), **pi'e** (mixed-base separator). (BPFK number subgrammar.)
- Consonant letterals (selma'o BY2): **by cy dy fy gy jy ky ly my ny py ry sy ty vy xy zy** and **.y'y** (apostrophe). Vowel letterals via `bu`: **.abu .ebu .ibu .obu .ubu .ybu**. Special: **denpa bu** = the period/pause symbol ".", **slaka bu** = the comma symbol.
- Since eSpeak's jbo voice disables numbers (`langopts.numbers=0`), our front-end must do number→cmavo expansion itself. camxes' `camxes_preproc.js` already does "replacing digits with the corresponding PA cmavo" — a reference implementation for the normalizer.

### 14. Morphology classifier — CONFIRMED sufficient, with rigorous fallback
The minimal classifier works for stress+pause placement:
- **cmevla** = ends in a consonant (CLL §4.8: "always followed in speech by a pause after the final consonant").
- **brivla** = contains a permissible consonant pair within the first five letters ignoring y and apostrophe, ends in a vowel, penultimate stress (CLL §4.3/§4.6).
- **cmavo** = everything else (one/two vowels, or C + one/two vowels, or Cy).
For rigorous tokenization of running text, defer to the PEG morphology. **`lojban/camxes-rs`** is real (Rust PEG parser generator; crates.io `camxes-rs = "0.1.1"`; the lojban-org repo was updated Feb 27, 2026 — 6 commits, low but non-zero activity). The canonical reference remains the JS `camxes`/`ilmentufa` PEG grammars and the vlatai/valfendi morphology algorithms. **Recommendation: ship the minimal classifier for the synthesizer's own front-end; optionally link camxes-rs (or port the relevant PEG productions) for exact word-boundary detection of dense text.**

### 15. WASM SIMD and size reality check
- **wasm `simd128`** is stable in Rust on wasm32 via `-C target-feature=+simd128`; usable for the biquad/oscillator inner loops.
- **`wasm32-unknown-unknown`** has full `core`+`alloc` support (partial `std`); `wasm32v1-none` is the clean `no_std`+`alloc` target for minimal builds. A parallel-formant engine is arithmetic-light (a handful of biquads + a glottal pulse + noise), so post-`wasm-opt` binaries in the **tens to low hundreds of KB** are realistic (klattsch-core is ~50 KB of source; the JS engine has zero runtime dependencies).
- **fundsp** supports `no_std` via `default-features = false` (v0.23) — audio file I/O and the rustfft-based convolution engine are disabled in no_std, which we don't need. It builds for wasm32; its graphs compile to stack-allocated inlined Rust types (no macros). fundsp is a reasonable alternative DSP substrate if you prefer FunDSP graph notation over klattsch-core's explicit engine, but klattsch-core is the more turnkey speech-specific choice.

## Details

### Corrected phonology/prosody summary (with CLL citations)
- **Vowels (6):** a e i o u + y=[ə] (schwa; limited distribution — Lojbanized names, letter names, glue vowel, space-filler). Peterson & Barney 1952 (JASA 24:175–184) male formant seeds (Hz, F1/F2/F3), exact match confirmed: **i 270/2290/3010; e 530/1840/2480; a 730/1090/2440; o 570/840/2410; u 300/870/2240**; y ≈ 500/1500/2500 (central).
- **Consonants (17):** b c d f g j k l m n p r s t v x z; apostrophe = [h] intervocalic only; period = mandatory pause/glottal stop [ʔ]; comma = syllable separator, never a pause.
- **Diphthongs (16):** ai ei oi au (off-glide, used everywhere); ia ie ii io iu ua ue ui uo uu (rising, standalone/names); iy uy (names only). No triphthongs.
- **Syllabication (CLL §3.9):** single C → following vowel; a CC pair → split, unless it is a valid initial pair → both to following vowel; CCC → split after the first C. Syllabic l/m/n/r count as consonants for morphology but form syllable nuclei.
- **Stress (CLL §3.9):** penultimate over *countable* syllables; countable excludes y-syllables, syllabic-consonant syllables, and buffer-vowel syllables. brivla/gismu/lujvo get primary penultimate stress; cmavo default unstressed; cmevla may be stressed anywhere (capitalized in writing if non-penultimate). Gismu are CVC/CV or CCVCV → always first-syllable stress.
- **Mandatory pauses:** before vowel-initial words; after consonant-final words (all cmevla); around zoi/la'o non-Lojban text; after Cy cmavo (e.g. `.y.`); and — per CLL §4.5 — "If the final syllable of one word is stressed, and the first syllable of the next word is stressed, you must insert a pause or glottal stop between the two stressed syllables"; a stressed final cmavo syllable immediately before a brivla requires a pause.

### Stress algorithm (pseudocode)
```rust
fn stress_index(word: &Word) -> Option<usize> {
    // nuclei: already split by syllabication
    let countable: Vec<usize> = word.nuclei.iter().enumerate()
        .filter(|(_, n)| !n.is_y() && !n.is_syllabic_consonant() && !n.is_buffer())
        .map(|(i, _)| i)
        .collect();
    match word.class {
        Class::Brivla => countable.iter().rev().nth(1).copied(), // penultimate countable
        Class::Cmevla => word.explicit_capital_stress
                             .or_else(|| countable.iter().rev().nth(1).copied()),
        Class::Cmavo  => None, // unstressed by default
    }
}
```

### Recommended attitudinal pitch + voice-quality table
Invented but principled convention; Klatt/KLSYN88 parameter names in parentheses. Baseline robotic voice: modal, flat F0 ≈ 110–120 Hz, monotone.

| UI cmavo (attitudinal) | Pitch contour (F0) | Rate | Voice quality (Klatt/KLSYN88 param) | Grounding |
|---|---|---|---|---|
| `.ui` (happiness) | +2.5 st mean, +2.5 st range (Tamuri Model 2 joy) | ×1.1 | modal, slightly raised AV | joy = high F0, wide range |
| `.u'i` (amusement) | rising–falling, wide range | ×1.1 | modal | high activation |
| `.oi` (complaint/pain) | low, falling | ×0.95 | **creaky** (DI↑, OQ↓, low F0) | creak ↔ complaint/low-pitched final |
| `.ii` (fear) | high, unstable | ×1.15 | **vibrato/jitter** (F0 sine mod + FL↑) | jitter/tremor ↔ fear/nervousness |
| `.o'o` (patience) | flat/monotone | ×0.95 | modal, minimal range | monotone ↔ calm |
| `.e'u` (suggestion) | terminal rise | ×1.0 | modal | rise ↔ non-final/interrogative |
| `.a'o` (hope) | gentle rise | ×1.05 | modal | mild positive activation |
| `.ie` (agreement) | mid, slight fall | ×1.0 | modal | neutral-positive |
| `.au` (desire) | mid-rise | ×1.0 | slightly breathy (OQ↑, AH↑) | breathy ↔ intimacy/desire |
| `.o'onai` / anger | low mean, widest range (Tamuri anger) | ×1.24 | **tense/harsh** (OQ↓, AV↑, TL↓, DI↑) | anger = widest range, tense/harsh |
| `.uu` (pity/sadness) | −4 st mean, −1.15 st range (Tamuri sadness) | ×0.9 | slightly breathy, low AV | sadness = low F0, narrowest range |

Attitudinal intensity (`cai`/`sai`/`ru'e`/`nai`) scales the depth of the F0/quality deviation: `cai` = full, `sai` = strong, `ru'e` = weak, `nai` = opposite polarity — implement as a multiplier on the deviation vector.

**Engine parameters to expose for the attitudinal layer:** **F0** (Hz), **AV** (voicing amplitude, dB), **AH** (aspiration, dB), **OQ** (open quotient, %), **TL** (spectral tilt, dB), **FL** (F0 flutter, % of F0), **DI** (diplophonia, %), plus F1–F5/B1–B5. klattsch-core already exposes voicing, aspiration/breathiness, spectral tilt, glottal effort, vibrato depth/rate — a close but not complete match; you will likely need to add explicit OQ modulation and a diplophonia (alternate-pulse) path for creak, which the Rosenberg-pulse source can support.

### Licensing analysis (GPL contamination — unchanged and important)
- **eSpeak NG** is **GPLv3**. Use it only as an out-of-process week-1 baseline/oracle (run the `jbo` voice compiled to WASM via Emscripten, or as a native binary) to A/B your output. **Do not** copy eSpeak's phoneme data, the `ph_lojban`/Esperanto tables, or any GPL source into the Rust engine — reimplement all linguistic rules from the CLL (a specification, not GPL code). Keeping eSpeak as a separate process avoids GPL derivation.
- **MBROLA** voices are non-commercial-only and there is no jbo MBROLA voice anyway — irrelevant, avoid.
- **klattsch / klattsch-rs / klattsch-core / klattsch-wav / klattsch-text / klattsch-wasm:** **MIT** — safe to depend on and to build a proprietary or permissively-licensed product on.
- **cpal:** Apache-2.0; **rtrb / ringbuf:** MIT/Apache-2.0; **fundsp:** MIT/Apache-2.0; **web-audio-api-rs:** MIT; **VoiRS:** Apache-2.0. **camxes-rs:** confirm the repo license before linking (port PEG productions if unclear). **Piper:** the current OHF `piper1-gpl` is **GPL**; the older rhasspy/piper was MIT — if a neural fallback is ever built, mind the engine license and the per-voice MODEL_CARD (some voices carry restrictive licenses; the Spanish carlfm voice is public domain).
- **Pendrokar/xvasynth_lojban** exists but was trained on la selpa'i's Lojban Corpus Readings — violates the no-training-data constraint; noted as prior art only.

### Recommended architecture
```
                 ┌─────────────────────────────────────────────┐
                 │  lojban-tts-core  (no_std + alloc, MIT)      │
                 │  ├─ normalize (numbers→cmavo, lerfu)         │
                 │  ├─ tokenize/classify (min classifier;       │
                 │  │    optional camxes-rs for dense text)     │
                 │  ├─ syllabify + stress (CLL §3.9)            │
                 │  ├─ pause insertion (classic CLL / dotside)  │
                 │  ├─ attitudinal prosody layer (F0/quality)   │
                 │  └─ LojbanTable: PhonemeTable  ─┐            │
                 │         drives klattsch-core     │ realtime   │
                 │         FormantSynth::process    │ safe        │
                 └──────────────────┬───────────────┘            │
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        │ native adapter            │            web adapter      │
        │ cpal + rtrb SPSC          │   hand-written              │
        │ (stable Rust)             │   AudioWorkletProcessor:    │
        │                           │   compile Module on main,   │
        │                           │   instantiate in worklet,   │
        │                           │   params via port.postMessage│
        └───────────────────────────┴──────────────────────────┘
        optional: wasm32-wasip2 WIT component (wasm-component-model)
```

### Updated 6-week plan
- **Week 1 — Baseline/oracle + skeleton.** Compile eSpeak NG `jbo` to WASM (GPL, out-of-process) as an A/B oracle. Stand up the Rust workspace; pull in `klattsch-core` (MIT) and render "coi" / "coi munje" via a hand-built schedule to validate the parallel-formant engine and the realtime-safe `process()` path. Confirm the klattsch-wasm worklet pattern (Module compiled on main thread, instantiated in worklet).
- **Week 2 — Phonology core.** Implement vowel/consonant/diphthong tables (17 C, 6 V, 16 diphthongs; P&B + locus-theory seeds) as a `LojbanTable: PhonemeTable`. Implement syllabication (CLL §3.9) and the corrected stress algorithm (exclude y + syllabic-consonant + buffer syllables). Unit-test against gismu (first-syllable stress) and multi-syllable lujvo.
- **Week 3 — Pauses, morphology, normalization.** Minimal word classifier (cmevla/brivla/cmavo); mandatory-pause insertion (vowel-initial, consonant-final/cmevla, Cy cmavo, stressed-final-cmavo-before-brivla, zoi/la'o); number→cmavo and lerfu expansion (own module, since jbo disables numbers). Add `--dotside` and `--buffer` flags. Optionally wire camxes-rs for dense-text boundaries.
- **Week 4 — Native path.** cpal output stream (stable Rust) pulling from an `rtrb` SPSC queue; control thread pushes `ParamUpdate`/`Schedule`. Verify no allocations in the callback (klattsch-core guarantees this for `process`).
- **Week 5 — Web path.** Hand-written `AudioWorkletProcessor` + ~40 lines of JS glue; instantiate WASM in the worklet scope; parameters/schedules over `port.postMessage` (no SharedArrayBuffer/COOP-COEP, no nightly). Build with `-C target-feature=+simd128`; run `wasm-opt -Oz`; confirm binary in tens-to-low-hundreds of KB. Optional: wasm32-wasip2 WIT component for the wasm-component-model target.
- **Week 6 — Attitudinal layer + polish.** Implement the F0-contour + voice-quality attitudinal table (expose F0/AV/AH/OQ/TL/FL/DI; add OQ modulation + alternate-pulse DI for creak/`.oi`, F0 vibrato for `.ii`); wire intensity markers (cai/sai/ru'e/nai). A/B against the eSpeak oracle; tune for the robotic aesthetic. Document the invented attitudinal convention as non-normative.

## Recommendations
1. **Adopt `klattsch-core` (MIT) as the synthesis engine**; implement a `LojbanTable: PhonemeTable` and feed it schedules from your own CLL front-end. Do not port chdh/klatt-syn. **Threshold to change:** if klattsch-core proves unmaintained or you need true cascade synthesis for nasals, fall back to a hand-rolled cascade/parallel engine or fundsp graphs.
2. **Web = custom single-threaded AudioWorklet; native = cpal + rtrb.** Do not use cpal's WASM backends. **Threshold to change:** only adopt threaded-WASM (SharedArrayBuffer, COOP/COEP, nightly `-Zbuild-std`) if a single 128-frame `process()` cannot keep up — unlikely for one robotic voice.
3. **Implement the corrected stress rule** (exclude y + syllabic-consonant + buffer syllables); **16 diphthongs, no triphthongs**; **default classic CLL pauses with a `--dotside` flag**; **omit buffer vowels with a `--buffer` flag**.
4. **Keep the no-training-data core purely formant-based.** Treat neural (VoiRS/Piper) as an explicitly out-of-scope, permissively-hosted (Spanish VITS) optional fallback only.
5. **Guard the GPL boundary:** eSpeak stays out-of-process; reimplement rules from CLL; keep the dependency tree MIT/Apache.
6. **Ship the attitudinal layer as a documented, non-normative convention** grounded in Gobl & Ní Chasaide voice-quality findings and Tamuri Estonian F0/rate seeds, exposing Klatt/KLSYN88 parameters.

## Caveats
- klattsch-rs is young (3 commits, 11 stars, v0.1.1, parity-tested only against klattsch JS 0.3.0); it is realtime-safe and MIT but small — audit before depending, and pin the version. `process` is realtime-safe but `new`/`reset`/`queue_schedule` allocate (call off-thread).
- The Klatt/KLSYN88 voice-quality parameter definitions (OQ, TL, FL, DI, AV, AH) were compiled largely from secondary/reference documentation and the Gobl & Ní Chasaide / Yanushevskaya literature rather than a fully verified copy of Klatt & Klatt 1990; verify exact mnemonics/magnitudes against the original paper (JASA 87(2):820–857) before finalizing parameter ranges. Gobl & Ní Chasaide specified manipulations in LF-model parameters (EE, RA, RG, RK, OQ) mapped to KLSYN88a (AV, TL, SQ, OQ + DI, AH); exact per-quality numeric settings are in the 2003 stimulus-description tables. klattsch-core does not expose all of these by name (it exposes voicing, aspiration, tilt, effort, vibrato); OQ modulation and a diplophonia/alternate-pulse path for creak may need to be added.
- The attitudinal pitch-contour convention is invented (CLL mandates none); the Tamuri figures come from Estonian read-out speech and are used as ratios/semitone offsets, not as Lojban-validated targets.
- The Wikipedia "Lojban grammar" article contains at least two errors relative to CLL (21 consonants; "triphthongs exist … e.g. <iau>") — prefer the CLL directly.
- cpal feature/version details evolve; verify the current `audioworklet` requirements against the cpal repo at build time.
- camxes-rs activity is low; confirm its license and completeness before linking, or port only the PEG productions you need.