# **Engineering a Dual-Target Rust and WebAssembly Text-to-Speech Synthesizer for Lojban**

## **Introduction to Mechanical Lojbanic Speech Synthesis**

The construction of a Text-to-Speech (TTS) synthesizer for the engineered language Lojban lies at a complex intersection of formal logic, computational linguistics, and low-level systems programming. Designed by the Logical Language Group to eliminate syntactic ambiguity and culturally specific grammatical irregularities, Lojban operates on the principles of predicate logic1. Unlike natural languages, where parsing relies heavily on semantic context and unpredictable prosody, Lojban possesses an audio-visual isomorphism: the spoken phonemes resolve into discrete words with mathematical certainty, provided the speaker strictly adheres to the language's morpho-phonological rules3.  
Developing a programmatic synthesizer that respects these rules requires the precise implementation of penultimate stress algorithms, mandatory pauses (denpa bu), the bypass of epenthetic buffer vowels, and the translation of grammaticalized emotions (attitudinals) into acoustic pitch contours5. Furthermore, the deployment targets demand a secure, offline, cross-platform architecture written in Rust. The synthesizer must be capable of executing natively (via ALSA, WASAPI, or CoreAudio) while simultaneously compiling to WebAssembly (WASM) to run in the browser using the Web Audio API8. This dual-target constraint strictly limits the reliance on bloated external dependencies or cloud-based generation, prioritizing deterministic audio callbacks that operate without heap allocations or thread locks.  
While legacy text-to-speech architectures rely on heavily parameterized formant synthesis (such as the Klatt model) to generate purely robotic, mechanical sounds, modern low-latency neural vocoders (such as VITS via the ONNX runtime) present an alternative. By leveraging cross-lingual phoneme mapping, it is possible to deploy a small neural model in WASM that generates high-fidelity audio without requiring native Lojban acoustic training data10. This report exhaustively details the linguistic frameworks, digital signal processing (DSP) algorithms, and systems architectures required to build an exhaustive Lojban TTS engine in Rust.

## **The Phonological Reality of Lojban**

A mechanical TTS system operates by mapping an incoming text string into a discrete sequence of acoustic targets. Lojban's orthography is intentionally phonetic, consisting of 27 valid letters (lerfu): 6 vowels, 21 consonants, and specific punctuation marks that dictate breath and pausing rather than grammatical syntax5. The orthography is merely a Latinate representation of the underlying International Phonetic Alphabet (IPA) phonemes, which constitute the true definition of the language5.

### **Vowel and Consonant Inventories**

The six Lojban vowels include five standard open, mid, and close sounds, alongside a central mid vowel or schwa (y). The synthesizer must assign target formant frequencies (F1, F2, and F3) to these vowels, which define their acoustic quality in the vocal tract.

| Lojban Grapheme | IPA Phoneme | English / Linguistic Approximation | Formant Profile Target (F1 / F2 / F3 in Hz) |
| :---- | :---- | :---- | :---- |
| a | \[a\] / \[ɑ\] | Open vowel (as in *father*) | 700 / 1200 / 2600 |
| e | \[ɛ\] / \[e\] | Front mid vowel (as in *bet*) | 500 / 1800 / 2500 |
| i | \[i\] | Front close vowel (as in *machine*) | 300 / 2200 / 3000 |
| o | \[o\] / \[ɔ\] | Back mid vowel (as in *open*) | 500 / 900 / 2500 |
| u | \[u\] | Back close vowel (as in *moon*) | 300 / 900 / 2500 |
| y | \[ə\] | Central mid vowel / schwa (as in *sofa*) | 500 / 1500 / 2500 |

The twenty-one consonants include fricatives, sibilants, stops, and approximants. The synthesizer must handle specific sounds that do not exist natively in all Western languages. For instance, the Lojban /c/ is an unvoiced coronal sibilant (\[ʃ\] or \[ʂ\]), and its voiced counterpart /j/ (\[ʒ\] or \[ʐ\]) maps to the "s" in the English word "vision"5. The letter /x/ represents the unvoiced velar fricative \[x\], requiring the synthesizer to generate high-frequency aspiration noise without glottal voicing, similar to the Scottish "loch" or Spanish "José"5.  
Furthermore, Lojban allows the sonorants l, m, n, and r to act as syllabic nuclei14. This is particularly common in Lojbanized names (cmevla) adapted from foreign languages. The TTS engine's text-to-phoneme front end must detect when these consonants are flanked by other consonants and assign them a vocalic amplitude envelope, enabling them to carry pitch and stress.

### **Diphthongs, Triphthongs, and Glides**

Lojban features 16 valid diphthongs. Unlike English, where diphthongs are often treated as distinct single phonemes, Lojban models them as a rapid transitional glide between a semi-vowel and a pure vowel, or vice versa5.

| Glide Type | Lojban Graphemes | IPA Transcription |
| :---- | :---- | :---- |
| Falling Diphthongs | ai, au, ei, oi | \[aj\], \[aw\], \[ɛj\], \[oj\] |
| Rising Diphthongs (i-based) | ia, ie, ii, io, iu, iy | \[ja\], \[jɛ\], \[ji\], \[jo\], \[ju\], \[jə\] |
| Rising Diphthongs (u-based) | ua, ue, ui, uo, uu, uy | \[wa\], \[wɛ\], \[wi\], \[wo\], \[wu\], \[wə\] |

Triphthongs exist as combinations of rising and falling diphthongs (e.g., iau)5. Computationally, the synthesizer handles these by identifying the start and end formant targets and executing a cubic interpolation over a window of 50 to 100 milliseconds17.  
The apostrophe (') is a unique morphological entity in Lojban. It only appears between two vowels and represents an unvoiced glottal spirant \[h\] or a similar unvoiced vowel glide (such as \[θ\])19. From a DSP perspective, the engine must briefly halt the periodic glottal pulse (![][image1]) and substitute it with aspiration noise (![][image2]) while smoothly interpolating the formants between the surrounding vowels18.

### **Anaptyxis: The Mechanics of Buffer Vowels**

Because Lojban constructs its core vocabulary (gismu) using an algorithm that mathematically weighted root words from six major languages (Chinese, English, Hindi, Spanish, Russian, Arabic), the resulting lexicon is rich with dense consonant clusters21. To aid pronunciation for speakers whose native languages lack these clusters, Lojban permits vocalic epenthesis—the insertion of "buffer vowels"4.  
Valid buffer vowels include \[ɪ\], \[ɨ\], \[ʊ\], and \[ʏ\]5. The critical constraint is that these vowels must be acoustically distinct from the standard six Lojban vowels and pronounced as briefly as possible. A robotic TTS engine can vastly improve its output clarity by programmatically inserting a 30 to 40-millisecond \[ɪ\] buffer between invalid or difficult consonant transitions (e.g., pronouncing sfani as \[s ɪ f a n i\]).  
Crucially, the syntactic parser must flag these buffer vowels and entirely exclude them from syllable counting, as they are grammatically invisible and must never receive stress5.

## **Algorithmic Syllabification and Prosody Generation**

The fundamental design of Lojban ensures that a continuous stream of audio can be unambiguously resolved into individual words. This is achieved through rigid rules governing stress and pausing. For a TTS engine to produce grammatically valid Lojban, its prosody generator must perfectly mirror the language's morphology.

### **The Syllable Parsing Algorithm**

Every vowel, diphthong, and syllabic consonant constitutes exactly one syllable22. The TTS text analyzer must traverse the input string and assign consonant clusters to either the preceding or succeeding syllable (coda vs. onset). Lojban defines exactly 48 permissible initial consonant pairs (e.g., pr, ml, zb, tc)23.  
The algorithmic logic for a Rust-based syllabifier is as follows:

1. **Nucleus Identification:** Scan the token array to locate all valid nuclei.  
2. **Consonant Grouping:** Examine the consonants separating any two nuclei.  
3. **Boundary Resolution:** If the consonant pair is a valid Lojban initial pair, the syllable boundary is placed before the cluster (making it the onset of the second syllable). If the pair is not a valid initial pair (an impermissible initial, but permissible medial pair), the boundary is placed between the consonants22.

### **The Rule of Penultimate Stress**

Lojban dictates that content words (brivla) receive primary stress on their penultimate (second-to-last) syllable5. This stress rule is the primary acoustic cue that allows a listener to identify word boundaries in a rapid speech stream.  
The Rust TTS engine must implement a deterministic stress-assignment routine with the following exclusions:

* **The Schwa Exception:** Syllables containing the vowel y are entirely ignored for stress calculation3. In a word like bisydja, the penultimate syllable is bis, not sy, resulting in the stress pattern BIS.y.dja.  
* **Syllabic Consonants and Buffers:** Similarly, syllabic consonants (l, m, n, r) and epenthetic buffer vowels are excluded from the syllable count22.  
* **Structural Words (cmavo):** Small structural particles do not require specific stress, though they are often penultimately stressed if polysyllabic. However, a cmavo may *never* carry primary stress on a syllable immediately preceding a brivla, as this would violate the brivla's boundary cues22.

When non-standard stress is required—such as in Lojbanized names (cmevla)—it is explicitly marked in the orthography using capital letters (e.g., DJOsefin. or niKOL.)24. The Rust tokenizer must detect uppercase ASCII characters, assign primary stress to that specific nucleus, and override the default penultimate algorithm.  
Acoustically, the TTS engine manifests "stress" by applying a synthesis modifier block to the targeted phoneme frame:

1. **Fundamental Frequency (![][image3]):** A localized positive parabolic pitch contour.  
2. **Amplitude (![][image4]):** A \+2 to \+4 decibel gain increase during the vocalic nucleus.  
3. **Duration:** A temporal dilation of the phoneme by approximately 15% to 20%17.

### **Denpa bu: The Calculus of Word Boundaries**

The period (.) in Lojban is called denpa bu and represents a mandatory pause, physically realized as a glottal stop \[ʔ\] or a distinct cessation of airflow6. Pauses are not merely suggestions; they are the structural pillars of Lojban's unambiguous grammar4.  
The TTS engine must inject a mechanical pause—typically 50 to 100 milliseconds of zero-amplitude data, or a synthetic glottal closure—under absolute rules:

1. **Vowel-Initial Words:** Any word starting with a vowel must be preceded by a pause19. This prevents the preceding word from bleeding its final sound into the vowel, which would trigger a misparse.  
2. **Consonant-Final Words:** Any word ending in a consonant must be followed by a pause19. Because all native Lojban brivla and cmavo end in vowels, a consonant-final word is strictly identifiable as a cmevla (name).  
3. **The "Dotside" Orthography Reform:** Modern Lojban usage adheres to a convention known as "dotside," which mandates that all cmevla be strictly bounded by pauses on both sides, regardless of their internal structure (e.g., .alis.)26. This simplifies older, highly complex rules regarding forbidden sequences like la or doi inside names.  
4. **Stress Collisions:** If a speaker accidentally stresses a syllable immediately prior to a brivla, a pause must be inserted to separate the two words and resolve the stress ambiguity22.

## **Semantic Prosody: The Mathematical Treatment of Attitudinals**

In natural languages, emotion and intent are largely conveyed through unstructured prosodic intonation. Because Lojban relies on lexical precision, it strips away vague intonation and replaces it with a rigorous system of "attitudinal indicators" (cmavo of selma'o UI)30. These words function as spoken emoticons, directly modifying the preceding word or the entire bridi (clause)7.  
While the grammatical presence of words like .ui (happiness), .oi (complaint), .ia (belief), or .u'u (repentance) is sufficient to convey meaning, a sophisticated TTS engine should map these lexemes to macro-prosodic DSP contours7. By applying mathematical envelopes to the fundamental frequency (![][image3]) and execution tempo, the robotic voice gains profound expressive depth while remaining entirely deterministic.

### **Attitudinal Envelope Modeling**

Let ![][image5] be the fundamental frequency at time ![][image6]. The baseline speaking frequency is ![][image7]. The modified frequency can be modeled as:  
![][image8]  
Where ![][image9] represents the micro-prosody (the bumps for penultimate stress and natural sentence declination), and ![][image10] is the attitudinal modifier envelope applied over the scope of the target clause.

| Attitudinal | Semantic Meaning | DSP Contour Execution (Eatt​(t) and Parameters) |
| :---- | :---- | :---- |
| **.ui** | Happiness / Amusement | Upward shift in ![][image7]. Widened dynamic range for stress peaks. Slight increase in playback tempo (+10%). |
| **.oi** | Complaint / Pain | Downward shift in ![][image7]. Terminal pitch drops on syllables. Introduction of low-frequency tremor (manipulating glottal open-quotient to create "creaky voice")18. |
| **.ii** | Fear / Nervousness | High-frequency, low-amplitude sine wave modulation applied to ![][image5] (vibrato). Increased vibrato rate and depth parameters18. |
| **.e'u** | Suggestion / Warning | A rising terminal contour applied to the final syllables of the clause, mimicking an interrogative up-talk. |
| **.o'o** | Patience / Tolerance | Flattening of the ![][image9] curve, resulting in a highly monotone, even-paced delivery. |

Because attitudinals can be modified by suffixes like \-cu'i (neutrality) or \-nai (polar opposite), the TTS engine can scale the amplitude of the ![][image10] envelope proportionally7. This programmatic approach to emotion avoids the unpredictable hallucination artifacts common in large-scale neural network synthesis.

## **Synthesis Paradigms: Architecting the Audio Generation**

To build a Lojban TTS engine in Rust that targets WebAssembly—specifically fulfilling the requirements of operating entirely in-browser without cloud connectivity—the developer must choose between a parameter-driven formant synthesizer and a lightweight neural vocoder approach8. Both paradigms can be executed within the Web Audio API boundary, but they present vastly different trade-offs regarding payload size, linguistic accuracy, and implementation complexity.

### **Paradigm A: Small Neural Approaches and Cross-Lingual Mapping**

Modern offline TTS is dominated by neural architectures like VITS (Variational Inference with adversarial learning for end-to-end Text-to-Speech). Systems like Piper TTS offer highly compressed, efficient models that generate near-human audio11. In the Rust ecosystem, these models can be executed using ONNX runtime bindings (via the ort crate) or heavily optimized machine-learning frameworks like tract, which natively supports WebAssembly compilation10.  
The user request specifies that a small neural approach is acceptable *provided no Lojban-specific audio is required for training*. Because there are no robust Lojban acoustic datasets in existence, deploying a neural model necessitates **cross-lingual phoneme mapping**34. The Rust engine must translate Lojban's IPA phonemes into the input dictionary of a pre-trained model—typically English (ARPABET) or Spanish.

#### **Mapping Lojban to ARPABET (English Models)**

If an English Piper model is loaded via WASM, the text must be translated into ARPABET tokens12. This presents severe linguistic compromises:

| Lojban IPA | Nearest ARPABET | Compromise / Acoustic Artifact |
| :---- | :---- | :---- |
| \[x\] (velar fricative) | HH or K | ARPABET lacks the \[x\] phoneme. The TTS will pronounce xorxes incorrectly as "horhes" or "korkes"5. |
| \[a\] (pure open) | AA / AE | English neural models tend to diphthongize pure vowels, introducing an unwanted off-glide34. |
| \[ə\] (schwa) | AX / AH | Acceptable match, but English models naturally stress these unpredictably based on their own internal English training weights35. |

A preferable neural alternative is to utilize a Spanish or Italian VITS model. These languages share Lojban's pure five-vowel system and heavily trilled/tapped rhotic consonants (r), resulting in significantly less diphthongization. A Spanish model also natively includes the velar fricative \[x\] (the "j" sound), eliminating the ARPABET mapping error5.  
The primary drawback of the neural approach in a WASM context is the binary payload. Even heavily quantized VITS models and their ONNX/Tract runtime binaries require 15MB to 30MB of assets to be downloaded to the browser11.

### **Paradigm B: Parallel-Formant Synthesis (The Klatt Model)**

For a purely mechanical, "robotic" voice that rigidly adheres to Lojban's phonotactics, the parallel-formant synthesizer designed by Dennis Klatt (1980) is mathematically superior17. Formant synthesis models the human vocal tract as a series of digital resonators (biquad filters). It does not require any acoustic training data, relying entirely on programmatic rules17.  
A Rust implementation of this architecture, such as klattsch-rs (a port of the JavaScript klattsch engine), provides a realtime-safe synthesis loop38.  
The synthesizer utilizes:

1. **Voiced Source (Rosenberg Pulse):** A mathematical approximation of the glottal pulse, controlled by fundamental frequency (![][image3]) and amplitude (![][image4]).  
2. **Unvoiced Source (White Noise):** Used to generate fricatives and aspiration, controlled by noise amplitudes (![][image11], ![][image12]).  
3. **Parallel/Cascade Biquads:** A series of bandpass filters where the center frequencies (F1 through F5) define the acoustic shape of the vocal tract17.

By utilizing klattsch-rs, the developer creates a hardcoded matrix of Lojban phonemes mapped directly to precise formant frequencies and bandwidths. The velar fricative /x/ is generated simply by bypassing the glottal source and routing high-amplitude white noise through a high-frequency bandpass filter18.  
Because the entire engine is procedural, it compiles to a WebAssembly binary of just a few hundred kilobytes, entirely eliminating the massive payloads associated with neural models38. The resulting audio possesses the distinct, synthetic aesthetic of 1980s speech synthesizers (e.g., SAM or Votrax), perfectly matching the user's desire to "mechanically produce" Lojban sounds18.

## **Cross-Platform Systems Architecture: Rust, Native, and WASM**

The prompt demands a system that operates across the JS/Web Audio API boundary but can also compile directly to native desktop environments. In the Rust ecosystem, this dual-target requirement is seamlessly fulfilled by cpal (Cross-Platform Audio Library)9.

### **The Unified Audio Backend**

cpal abstracts away the underlying operating system audio drivers.

* **Native Linux:** Targets ALSA or PipeWire9.  
* **Native Windows:** Targets WASAPI or ASIO (for ultra-low latency)9.  
* **Native macOS:** Targets CoreAudio40.  
* **WebAssembly:** Targets the browser's AudioWorklet9.

By writing the TTS engine to push samples into a generic cpal::StreamTrait, the exact same Rust codebase can be executed as a local CLI tool or embedded within a web page42.

### **Traversing the Web Audio API Boundary**

Executing audio synthesis in the browser requires strict adherence to the AudioWorklet architecture. Older methods, such as ScriptProcessorNode, ran on the main JavaScript thread, causing severe audio stuttering during DOM manipulation44. The AudioWorklet offloads DSP calculations to a dedicated, high-priority audio thread41.  
To compile cpal and the TTS engine for the browser, the architecture must fulfill specific constraints:

1. **Compilation Target:** The code is built targeting wasm32-unknown-unknown9.  
2. **Concurrency Features:** The build requires the rustc flags \-C target-feature=+atomics,+bulk-memory,+mutable-globals to enable threading within the WASM module9.  
3. **Cross-Origin Isolation:** Because cpal utilizes a SharedArrayBuffer to move data between the main browser thread and the AudioWorklet thread, the web server hosting the application must emit strict HTTP security headers: Cross-Origin-Opener-Policy: same-origin and Cross-Origin-Embedder-Policy: require-corp9.

### **Real-Time Thread Safety and Memory Constraints**

The callback function executing within the AudioWorklet (or native ASIO thread) operates in a "hard real-time" environment. It is called rapidly (e.g., every 128 sample frames) to fill the output buffer8. To prevent audio dropouts (xruns), the code inside this callback must **never** block. This means no standard mutex locks, no I/O operations, and absolutely no heap allocations (e.g., creating new Vec or String objects)38.  
To pass Lojban text into the audio thread safely, the architecture must utilize a lock-free, Single-Producer Single-Consumer (SPSC) ring buffer, such as the rtrb crate46.  
The full execution pipeline operates as follows:

1. **Main Thread (JS/WASM):** The user inputs a Lojban string. The Rust parser analyzes the text, assigns syllabification, calculates penultimate stress and denpa bu pauses, and extracts attitudinal pitch contours5.  
2. **Main Thread (JS/WASM):** The parser compiles the linguistic data into a schedule of raw DSP parameter updates (e.g., SetTarget { f0: 220.0, transition\_samples: 480 })38.  
3. **Main Thread (JS/WASM):** The schedule is pushed into the producer side of the lock-free ring buffer.  
4. **AudioWorklet Thread (WASM):** The cpal audio callback periodically awakens. It pops parameter updates from the consumer side of the ring buffer38.  
5. **AudioWorklet Thread (WASM):** The Klatt formant synthesizer (e.g., klattsch-core) updates its biquad filters and glottal oscillators without triggering any allocations38. It calculates the raw f32 audio samples and writes them directly into the Web Audio API output buffer, resulting in pristine, mechanical Lojban speech38.

## **Conclusion**

The engineering of a text-to-speech synthesizer for Lojban is fundamentally an exercise in algorithmic precision. Because Lojban was intentionally designed to possess an unambiguous, logical grammar with strict audio-visual isomorphism, its spoken form can be mechanically generated with a degree of accuracy impossible in natural languages.  
A dual-target architecture written in Rust, leveraging the cpal library, successfully bridges the divide between native desktop execution and the in-browser Web Audio API AudioWorklet. While small, WASM-compatible neural models (via ONNX/Tract) offer a viable path, they introduce severe cross-lingual mapping errors and hefty payload sizes due to the lack of native Lojban acoustic data. Conversely, a parallel-formant synthesis engine—such as the Klatt model—offers a mathematically pure, zero-data approach that generates a perfectly mechanical voice in a microscopic binary. By hardcoding Lojban's unique phonology, mandating the strict execution of denpa bu pauses, bypassing epenthetic schwas for penultimate stress, and mathematically translating attitudinal cmavo into prosodic pitch contours, the resulting synthesizer embodies the precise, logical aesthetic that the language was designed to achieve.

#### **Works cited**

1. Lojban Reference Grammar \- The Swiss Bay, [https://theswissbay.ch/pdf/Books/Linguistics/Mega%20linguistics%20pack/Conlangs/Lojban%20Language%2C%20The%20Complete%20%28Cowan%29.pdf](https://theswissbay.ch/pdf/Books/Linguistics/Mega%20linguistics%20pack/Conlangs/Lojban%20Language%2C%20The%20Complete%20%28Cowan%29.pdf)  
2. About Lojban, [https://lojban.io/FAQ](https://lojban.io/FAQ)  
3. Lojban/Sounds and alphabet \- Wikibooks, open books for an open world, [https://en.wikibooks.org/wiki/Lojban/Sounds\_and\_alphabet](https://en.wikibooks.org/wiki/Lojban/Sounds_and_alphabet)  
4. Does Lojban really have “unambiguous resolution of sounds into words”?, [https://linguistics.stackexchange.com/questions/32463/does-lojban-really-have-unambiguous-resolution-of-sounds-into-words](https://linguistics.stackexchange.com/questions/32463/does-lojban-really-have-unambiguous-resolution-of-sounds-into-words)  
5. Lojban grammar \- Wikipedia, [https://en.wikipedia.org/wiki/Lojban\_grammar](https://en.wikipedia.org/wiki/Lojban_grammar)  
6. As Easy As A-B-C? The Lojban Letteral System And Its Uses, [https://lojban.github.io/cll/17/4/](https://lojban.github.io/cll/17/4/)  
7. Lojban/Attitudinals \- Wikibooks, open books for an open world, [https://en.wikibooks.org/wiki/Lojban/Attitudinals](https://en.wikibooks.org/wiki/Lojban/Attitudinals)  
8. A Rust implementation of the Web Audio API, for use in non-browser contexts \- GitHub, [https://github.com/orottier/web-audio-api-rs](https://github.com/orottier/web-audio-api-rs)  
9. RustAudio/cpal: Low-level cross-platform audio I/O library in Rust \- GitHub, [https://github.com/RustAudio/cpal](https://github.com/RustAudio/cpal)  
10. tract — ML/AI/statistics in Rust // Lib.rs, [https://lib.rs/crates/tract](https://lib.rs/crates/tract)  
11. Building a Browser-Based Text-to-Speech System with Piper TTS \- DEV Community, [https://dev.to/linmingren/building-a-browser-based-text-to-speech-system-with-piper-tts-ljh](https://dev.to/linmingren/building-a-browser-based-text-to-speech-system-with-piper-tts-ljh)  
12. wwesantos/arpabet-to-ipa \- GitHub, [https://github.com/wwesantos/arpabet-to-ipa](https://github.com/wwesantos/arpabet-to-ipa)  
13. Lojban phonology and orthography \- Paul Marciano Wiki \- Fandom, [https://paul-marciano.fandom.com/wiki/Lojban\_phonology\_and\_orthography](https://paul-marciano.fandom.com/wiki/Lojban_phonology_and_orthography)  
14. The Complete Lojban Language (2016)/Chapter 3 \- Wikisource, the free online library, [https://en.wikisource.org/wiki/The\_Complete\_Lojban\_Language\_(2016)/Chapter\_3](https://en.wikisource.org/wiki/The_Complete_Lojban_Language_\(2016\)/Chapter_3)  
15. LESSON 1: Sounds, names and a few attitudes \- Lojban, [https://www.lojban.org/static/publications/tutorial/lesson1.html.jbo](https://www.lojban.org/static/publications/tutorial/lesson1.html.jbo)  
16. Phonology and Morphology for a Logical Language, Part I: Critique of Lojban \- Reddit, [https://www.reddit.com/r/conlangs/comments/nwir4b/phonology\_and\_morphology\_for\_a\_logical\_language/](https://www.reddit.com/r/conlangs/comments/nwir4b/phonology_and_morphology_for_a_logical_language/)  
17. crispinprojects/klatt-synthesizer \- GitHub, [https://github.com/crispinprojects/formant-synthesizer](https://github.com/crispinprojects/formant-synthesizer)  
18. GitHub \- tgies/klattsch: primitive parallel-formant speech synth in the browser, [https://github.com/tgies/klattsch](https://github.com/tgies/klattsch)  
19. Page:CLL v1.1.pdf/35 \- Wikisource, the free online library, [https://en.wikisource.org/wiki/Page:CLL\_v1.1.pdf/35](https://en.wikisource.org/wiki/Page:CLL_v1.1.pdf/35)  
20. 3.3. The Special Lojban Characters, [https://lojban.org/publications/cll/cll\_v1.1\_xhtml-section-chunks/section-lojban-characters.html](https://lojban.org/publications/cll/cll_v1.1_xhtml-section-chunks/section-lojban-characters.html)  
21. 4.14. The gismu creation algorithm \- Lojban, [https://lojban.org/publications/cll/cll\_v1.1\_xhtml-section-chunks/section-gismu-making.html](https://lojban.org/publications/cll/cll_v1.1_xhtml-section-chunks/section-gismu-making.html)  
22. 3.9. Syllabication And Stress \- Lojban, [https://lojban.org/publications/cll/cll\_v1.1\_xhtml-section-chunks/section-stress.html](https://lojban.org/publications/cll/cll_v1.1_xhtml-section-chunks/section-stress.html)  
23. The Hills Are Alive With The Sounds Of Lojban \- The Lojban Reference Grammar, [https://lojban.github.io/cll/3/7/](https://lojban.github.io/cll/3/7/)  
24. Lojban Reference Grammar: Chapter 3, [https://lojban.org/misc/lojban-thing-1.0.3/local/hrefgram/chapter3.html](https://lojban.org/misc/lojban-thing-1.0.3/local/hrefgram/chapter3.html)  
25. The Shape Of Words To Come: Lojban Morphology \- The Lojban Reference Grammar, [https://lojban.github.io/cll/4/8/](https://lojban.github.io/cll/4/8/)  
26. 4.9. Rules for inserting pauses \- Lojban, [https://lojban.org/publications/cll/cll\_v1.1\_xhtml-section-chunks/section-pauses.html](https://lojban.org/publications/cll/cll_v1.1_xhtml-section-chunks/section-pauses.html)  
27. Lojban Reference Grammar: Chapter 4, [https://www.lojban.org/publications/reference\_grammar/chapter4.html](https://www.lojban.org/publications/reference_grammar/chapter4.html)  
28. The Lojban I speak \- GitHub, [https://gist.github.com/lynn/453a1ccc62aafbc24d2bfbd29bf5cabf](https://gist.github.com/lynn/453a1ccc62aafbc24d2bfbd29bf5cabf)  
29. The Dot Side \- La Lojban, [https://mw.lojban.org/papri/The\_Dot\_Side](https://mw.lojban.org/papri/The_Dot_Side)  
30. 2.16. Indicators \- Lojban, [https://lojban.org/publications/cll/cll\_v1.1\_xhtml-section-chunks/section-attitudinals.html](https://lojban.org/publications/cll/cll_v1.1_xhtml-section-chunks/section-attitudinals.html)  
31. Text to speech for Rust \- help \- The Rust Programming Language Forum, [https://users.rust-lang.org/t/text-to-speech-for-rust/110824](https://users.rust-lang.org/t/text-to-speech-for-rust/110824)  
32. piper-rs \- Lib.rs, [https://lib.rs/crates/piper-rs](https://lib.rs/crates/piper-rs)  
33. blazen\_audio\_piper \- Rust \- Docs.rs, [https://docs.rs/blazen-audio-piper](https://docs.rs/blazen-audio-piper)  
34. ARPABET \- Wikipedia, [https://en.wikipedia.org/wiki/ARPABET](https://en.wikipedia.org/wiki/ARPABET)  
35. ARPAbet to IPA Mapping Guide | PDF | Phonetics | Speech Synthesis \- Scribd, [https://www.scribd.com/document/907010756/NLSP-4](https://www.scribd.com/document/907010756/NLSP-4)  
36. 3.10. IPA For English Speakers \- Lojban, [https://lojban.org/publications/cll/cll\_v1.1\_xhtml-section-chunks/section-anglophone-phonetics.html](https://lojban.org/publications/cll/cll_v1.1_xhtml-section-chunks/section-anglophone-phonetics.html)  
37. VoiRS is a cutting-edge Text-to-Speech (TTS), Voice Recognition, Sound framework that unifies high-performance crates from the cool-japan ecosystem \- GitHub, [https://github.com/cool-japan/voirs](https://github.com/cool-japan/voirs)  
38. GitHub \- tgies/klattsch-rs: Rust port of klattsch, a parallel-formant speech synthesis engine, [https://github.com/tgies/klattsch-rs](https://github.com/tgies/klattsch-rs)  
39. klattsch-core \- crates.io: Rust Package Registry, [https://crates.io/crates/klattsch-core](https://crates.io/crates/klattsch-core)  
40. cpal \- crates.io: Rust Package Registry, [https://crates.io/crates/cpal/0.13.0](https://crates.io/crates/cpal/0.13.0)  
41. Rust WASM Audio Worklet Example \- GitHub, [https://github.com/PaulBatchelor/rust-wasm-audioworklet](https://github.com/PaulBatchelor/rust-wasm-audioworklet)  
42. cpal \- Rust \- Docs.rs, [https://docs.rs/cpal/latest/cpal/](https://docs.rs/cpal/latest/cpal/)  
43. nannou-org/cpal\_wasm\_example: Example project for compiling CPAL to WASM and publishing on the web \- GitHub, [https://github.com/nannou-org/cpal\_wasm\_example](https://github.com/nannou-org/cpal_wasm_example)  
44. Web Audio Effect Library with Rust and WASM \- Ryosuke, [https://whoisryosuke.com/blog/2025/web-audio-effect-library-with-rust-and-wasm/](https://whoisryosuke.com/blog/2025/web-audio-effect-library-with-rust-and-wasm/)  
45. Wasm Audio Worklet \- The \`wasm-bindgen\` Guide \- Rust and WebAssembly, [https://rustwasm.github.io/docs/wasm-bindgen/examples/wasm-audio-worklet.html](https://rustwasm.github.io/docs/wasm-bindgen/examples/wasm-audio-worklet.html)  
46. klattsch-core — Rust audio library // Lib.rs, [https://lib.rs/crates/klattsch-core](https://lib.rs/crates/klattsch-core)  
47. Creating a DAW in Rust \- Playing Audio \- Ryosuke, [https://whoisryosuke.com/blog/2026/creating-a-daw-in-rust/](https://whoisryosuke.com/blog/2026/creating-a-daw-in-rust/)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEIAAAAaCAYAAAADiYpyAAABUUlEQVR4Xu2XDY0CQQxGRwMW0HAWsIAFLJyFc4AEJOAABzjAAAKOeSFNStPu7oS/HehLmrANW6bffNtZSkmSJLmfvxr/NvmNIEJvQvzUONfY1jiUB6x/VfoTYlGuIiCGsKvxq66b4MZT6U8IRPDWS25jk1OgIDf2JkS0XnJ7mxwDW4m1jsUvPFeGhPDygzBg9OfmAg0wh1pijKjhKB/CXNCDBTtRYMoi5kDUcJQPYTboHWDiekJIYYJjyua1q15J1HCUd/EWLwOTFyuLV5iZwhH2LqKGyeH2Udhx3GAhHwmBcLbplvN6Wa41WmKMISEmnRqeCMDpEVmdR2Ktrr3vvBrvLwGC05/dtBuwPg3Ie4OGAjQrtqJpXYwfFadwr54V74L1sVb9ZokTWpzaDI+NuEA7Yw6wMWwSArChT0WecXuifCU8Nk+1XS9wXCZJkiSfzgXwqnDUOGw7sQAAAABJRU5ErkJggg==>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEMAAAAaCAYAAADsS+FMAAABhklEQVR4Xu2WgU0DMQxFPUNXYAZWYAVWYIWuwAaMwAhswAZswAIdoNwj/VWwklN8CdCqeZLV9nK1k2/HidlkMpmM53mxo394qyDEtYpxWOzTUkL5fPw5HOPBrlcM5v6x2O70m0/WcX9+I8Dekpp/IcarpVijYMGlOW9eCyX2ZB0OghCLmJR0L7U+t2ktKKtyotTCDjp5txTzzg80ov97NomBs/x72MEAEIK4bKEo2t6ecGJxRL8Qb5Yc0JD+ixdLW6h1DrUKCCdWQWVkZk0MBcYQjonnz0bBacDc8qqtUauMkBilQGqitcYmsXIk4iiilaFq9jQniEAE9OiuUROjlAXeRcRe6BuIyuJahYDu06QkBOjMLlUNMMaE863VFHAFYjGfracJ4MPfONfW8Q0ZVHCfTSajHkAF4Fw3OsEYXRofWLhjZyAkPsisjxOFJA67gbaAWF7pUJM6MfoGKhCAk3HUtl2FAL6XqIoi9FbBRVDqM4jx61m4JLT/ZJS5jljZZDKZTG6BL0UOiKcg7RX/AAAAAElFTkSuQmCC>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABUAAAAbCAYAAACTHcTmAAAAyklEQVR4Xu2TYQ3CMBBGTwMW0DALWMACFrCAg0lAAg5wgAMMTAD0ZSncvqTdrfBnyV5y6a63fm3vrmYbq6RP9graYmoLd1aOVamJwl0nIiD4cP4+2dn5i0UPNoqS3wyCXujqvkOwGNGjjRvkTRib0SrP5XeWi31PqfMKKbkl6zSg5KvTNh69OnHyyjgkO03DU6JX9SdHEOEiEVHay/+TC1kkIqoi6n/QStfEVUT9JlRE/Wb8E/6bKBXPbcdzfrrYT9D0tJb29MZaeANxOEYsLJIABQAAAABJRU5ErkJggg==>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABwAAAAaCAYAAACkVDyJAAAA0UlEQVR4Xu2UUQ3CQAyGqwELaMACFrCABSzgAAlImAMc4GAGJgDuy2jSdB3hgfZl9yVNtsut//q3dyKdrXBt8fKLmSBWJniUQsFLi1EKBacWZykSPHwCnlIg+HDPqYL0jf4pg8yCDFAK9I7kGneJBbW3xC1Yty6tEm3SweEC8ERW0/OdX4ygAqrz6FmMBPlBn9y24yuRGDCtaxZh5cm8R3sWYBkb9dxZ9jInRZBhIrmtiKq1cr61vUwBu7UqW2kaOICgn+BUsPvnQfkHHINOJ4c3OLo4743N0lIAAAAASUVORK5CYII=>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACgAAAAWCAYAAACyjt6wAAABZElEQVR4Xu2WYW0DMQxGg6EUhmEURmEURqEUymAQBqEMxmAMRmAA1ry7s2R9teNou0390SdZ6sWX5LNj59ranTu78KEDCeduBx38a166PehgAgLLYJ7a+uJ3t9P2bPbc7W3z8bvis9u7DjqOOtDWgMq1EYBl4KuOAv8oGwSf+VkfoSkzAivYIMqQQWZfdXCD9TPxC5HAL/eb6Ct0vodyGZ0CwkfzryKgJoYTBBOgWP2qKWQ/Gl8gO7oANpM1w9bIyIQZWYAL1AZO37327LFuj+rstwIh9JsYLV69KqhH32Wa3aqGoj2UcL5FrvfQozxrRrnvPDMZ1D2UcP5M6kHfUcEcu75j6P0XZTKtwb0Ephu0tVzoZuBk/PVlXHWxCVPLwOfvMBUICNHSAOYhCn/2tSDDo/1LVFB061efuhGsn4mfgqawzkVItpgGMos23Y9AGM0QHaMxqsUIO/5/hezu+n/wprgAA4SAGyLudhUAAAAASUVORK5CYII=>

[image6]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAcAAAAdCAYAAABmH3YuAAAAb0lEQVR4XmNgGLbADYg/owvCQBcQ70QXhIETQFyJLGAKFbwOxP+hbBAGA5A9IONgkiA2CKMAkGqQJAYA6QZJYHUpyBiQJFaXwoxEcSkMgCSQ7UNxEMwLIKAMxM+Q5FAkQV7KQJIDA1BggFw9CpABAAIRHTXvxtS7AAAAAElFTkSuQmCC>

[image7]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACUAAAAXCAYAAACMLIalAAABPklEQVR4Xu2WbQ3CMBCGqwELaMACFrCABSzgAAlIwAEOcIABBEAf2Lu8uXXASNb9YE9y6ceuvY9em6U0M/OHLLKss1yz3LNsmrFkm+XcfEO3KhhF+nj3bTQ+OUW2qlNyam/9g/WrQO3gkBumP0l2BBnBKVoVuMaTodsVBeecZRiPRunoIB6d9Kqgo+N9ivNxXM2p0q0roWzuUvcYV6kblECXb/HhZZ6HOc4/GeIUrz4GbjbPpqf0Ol5aRGD00rS+5mjz7NnCJhjwoo6F7bCJkGGy5gHFMlB2ySQOAGUgHdb7vq0zUfrwi6DoaD0DrPfj4JsyjGNAn0uEyNGf8CwqOxin7Xv10ZMjHJeyGwMv1tQ3sFApJ3oZJ2r1MUzk0iMTMqhaBPQExR6fnkHoF6eEjJMZjxx9Zcthrm+vmbc8ACARZLzYfcICAAAAAElFTkSuQmCC>

[image8]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAmwAAAAwCAYAAACsRiaAAAAEHUlEQVR4Xu3cgY3jRBQG4NRAC9RAC7RAC7RAC3RACZRAB3RABzRwBcD+unvSu6fnxLubrMjxfZKV2B7bM+NI82ec3csFAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD44WX5aW784tq+/4Pv5wbu6ruX5be58Ytr+wDg6f34svxz+Ry08r7WMwBuPs0Nl6/L/t3e39Ovl8/1qjpmcM76f8Wfl+v1SZ3TN72fbx1zb/NauW89YM/99/DH5ev7Vp+vt5jHJSDnS0L5vb0HgG9KBaG5bbPNIG1lHzHLljr+tWx7lNecOyEkfZNjroWG7ZwJbR8h9erhJlKffq8S4GYfb15T51xjzny95viyfRHIuecXi3ktAPgm9CBUA/ovX16nbaDdQsg2uL5XDxeZtemvj7C160iVvTVj1vfV+0eE281Wr7Pbpu1zcKSHqvqcveW+bfU6uw0Anl4GuL5cs+0/uy0y0F9brul1vFX2Ho7acEsPKF2FuVoyK/eRtvac3Ta9pv/rfuX12uzjLVu9ttnArRwAPL0+wPWBOIFizrTNwfDo90jbtvfq56ywsz3OfY+cr5act68fmSEkddt+55fz1eO6fr4cP/v5mi0MTnk8O+uw9dP2+HArl2v2vsgM6pm+iR6q6pH6z23bWVu9ttnJrRwAPL1tliIDaoWi/nhzDoZZf82gOWfU5nIkoWC7Tmz1v4ejNnQJINtMWY6dIezM+e5p+71adxS0ZrnNtXvVpdyZgFn6tWc95vpRHWY5AHh6R0GoD4ZnBtH5m7VZ7r2OzpewlGvXbFK1p88uJXxW/bIvxxyFle7oml3K9JmmWrJ9Hj/XS59tSj3rfiRwZV9+75VQmuDTy+Z9ytaS9Tmr1s3r1/pbguVRWJq2c/VtaVfNUOacNXPX35d5rlqfv4eb5QDgqVWo2MLFUWCbQSeD7TZ4z8eEb1WPXGuZwbCuXYN2PeLrj/pyjoSS+gvIs/8rbvbJZvbhXBKiUsejfo5qU83UVd0qSKW+6fe0qbZlZi/r2df/srNee7Ar89o5tvqjm+U22z2fZl/UUsfO60afjZszc7NeaWP6bpabnxEA+GbVTFRUCCgztE1bWHiUGsTnYF5qximvqXe1afv3JNMWKB4hdUt9ZghKCK06VJ+nnQltvWxvS8JLhdwtNF+bgYuzYedM/91Sn6u0JW2te5Mg2t+XtG320XSPegHAU8ngPsNa3Br0b+2/p4TDHiAzwGepGbfUvw/yCSS3Bv2P1vurHgdG6prQlW3z32JUeKmyaW+WCjjbfYtbYbsHpEdLG9K+Cmg9tG4BNrZt3VFwBwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJ7Zv0uiLkD99cK5AAAAAElFTkSuQmCC>

[image9]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFYAAAAXCAYAAACRUrg+AAACg0lEQVR4Xu2YgU0DMQxFbwZWYAZWYAVWYAVWYANGYAQ2YAM2YAEGgHtqf2X92kkolXoVeVJEcZwQ/zhOyrJMJpPJZJCbtX24MeF2bZ9uzHhY27cb/yGI+uLGgjs3ZCAq7dU7CtjZ+2W3a4xjY/hd7XFt7/s+fK+Bp2W39gwy9NmNI0gEGpOMojEVrb4tweZ/uTHwtpwQC6ICgvaEcnr+mnvrcPJawvbiPILUjwN+O0HmH4/MaL26NMRAVlbQP3KpHWBArCsqCSP1RJsSxePztWRpxHWAeIJjoxZ3cREkFseid+kgvjZBl9bopmwN1p3dLcSEkBKe1tPl4OiMZm288GLL5uwu5sKw7gp0GC4DSvMKiVSRlQHwEwCjT7gWOh1/gQuqojU3fTwnhyDYTAQhYatMU6D+B7MsbwU0CuvgbXwqrURi7qpPCVTpcERLVIi1NqOX0QIxMj82xIXim4zqHH2x5vkGiuxRn9VBEqmKBbI1gsqdYI0+9wEW7IvJoK4wqQsAo8J6QCwKG3PStEhslBV8+YlgGkeff9UkQE4CccQkwY9Tgy3WRXx7z6ns8vI4K7/yCdFrgkD0fwVadQEKfOLzhN8Zj6DRLoElAD7KakRkXNxgicY4lR/GaR6vyXxufbf3dYoYP/1ltrL7LtpIE273foe+GBBZSDYpax38swDj849x7uM11DO2VQaAsVlGq5Sx4fFkXBxfrC9OYumYSwDsGqvs1RHMLkidIsFnbPgyjo1Ij/Aez/DNo8BEzCLsymYyAjElPELrwpKQ+knpiadAokkY3fK6S9SyEyIY08vqTZHdolmtizd5VrM92/DN/KKt+twibvzkjPjro0WWIJNz8QPSUOse3zl6qAAAAABJRU5ErkJggg==>

[image10]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADoAAAAaCAYAAADmF08eAAAB2ElEQVR4Xu2WC00EQRBEVwMW0IAFLGABC1jAARKQgAMc4AADJwD2JVdJp657dvaYTbhkXjLJXe+nfzU9uyyTyWQgX24o4L4HN94Kz+u6d2PBy7pObuwBB4/r+jkvfsdFEO/h2mi+1/XpxgCJOcT85sZelGgF13qrvodWAZFoFdPV8WwlepVcNkAtWccEna46x15txVvCQz4QoqRa8rqWj6U9WIjpyY1nKMDuRF+Xy5eSGHvzKDQXHM0DXw6yzexNSIqHqBJJyxnSOgrenQWq4ShpaihmZM+XqLK+F5iGRyIVVVSdjNCgqggXZLKFKNu78HsUPYn6zHB2JdpTuT3d5V2xMFVClV1kxXeGJspe2nO0eBcIxm3QSlTbSQWrFMU93WdpK1Ed2PFl2AicapOEzkElxFKlNeR0f6SauhAnPv6yQndPXRxp8iFNTTctzjiu+RGDU7oBPB8TwO7Vj51xqkAZjBqOFCmb/vitnv8zVJECCCofyRz7NI+ghurLSAWvoABZp4eA4xhYdJRJEZtLNkKns/3bQ6a2YdBRBU6AOKNj2o/Y2FPqujre+syjWK6MLYhD2+cwSAYnJEyQSkr/CVp7Ehv3thLJlLBF633/nl4JSzWTyeTG+QWsCZnnzYOvtgAAAABJRU5ErkJggg==>

[image11]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAB4AAAAaCAYAAACgoey0AAAA4ElEQVR4Xu2UUQ1CMQxFqwELaMACFrCABSzgAAlIwAEOcIABBMBOxg1NM8g+WN/PTtKENC+7bW+L2WRSOZZ4xmQGiKYLb20B4UOJuy0g/Cixt2ThzTvgZonC1/A7RRhf8VdcrAqzaEPBW0QUZ/strB1YudzJ5bvwIxZaMP5IWlDYzuqkYr4LOqLbiG75mzDLB3xDAYKCu2iJAtvNo61pgPZBN98NlfGo7tazto9fPExH3kvQ2VGAhJUbBoV58JXiuYShtCbEZOT7EBBp7QXjjgX9DXzWnbIDHjqOFkwmObwAzvY/FXadVPwAAAAASUVORK5CYII=>

[image12]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABwAAAAaCAYAAACkVDyJAAAAx0lEQVR4Xu2UUQ3CQBBERwMW0FALWMACFrCAg0pAAg5wgAMMVADcS9lwLHf0p3s/3EsmaZpmZ/fmtlLnXzglPfzLSDBrZrhTQ8Nj0l0NDaekgxoZDi/BTQ0Mr+451JDcyM+4aDbkAoVAdhQ3nVU33OidsRf7u0h+lIZdnFoBa8hTavADPmA6j+1izdBWx8P0PymZAbeVgqXpIW+G01iEjyhme5ezTRo1F2WSvb4799mFQkN+hULhRPJs7WcRRi331SntX6ezPk/Jzj7pW0n0vAAAAABJRU5ErkJggg==>