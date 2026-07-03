# voksa — Lojban Phonology Rules (single source of truth)

Derived from The Complete Lojban Language (CLL), chapters 3–4. All rules here were
verified in docs/research/02-architecture-v2.md. On conflict, CLL > this file >
research reports. Cite section numbers when implementing.

## 1. Inventory

### Consonants (17)
b c d f g j k l m n p r s t v x z

| Letter | IPA | Manner / notes |
|--------|-----|----------------|
| b | [b]  | voiced bilabial stop |
| c | [ʃ]  | unvoiced coronal sibilant |
| d | [d]  | voiced dental/alveolar stop |
| f | [f]  | unvoiced labial fricative |
| g | [ɡ]  | voiced velar stop |
| j | [ʒ]  | voiced coronal sibilant |
| k | [k]  | unvoiced velar stop |
| l | [l]  | lateral approximant; may be syllabic [l̩] |
| m | [m]  | bilabial nasal; may be syllabic [m̩] |
| n | [n]  | dental/alveolar nasal ([ŋ] variant); may be syllabic [n̩] |
| p | [p]  | unvoiced bilabial stop |
| r | [r]  | any rhotic; may be syllabic [r̩] |
| s | [s]  | unvoiced alveolar sibilant |
| t | [t]  | unvoiced dental/alveolar stop |
| v | [v]  | voiced labial fricative |
| x | [x]  | unvoiced velar fricative (no voiced counterpart) |
| z | [z]  | voiced alveolar sibilant |

tc [tʃ], dj [dʒ], ts [ts], dz [dz] are CLUSTERS, not phonemes — synthesize as
stop + fricative sequences.

### Vowels (6)
a [a], e [ɛ], i [i], o [o], u [u], y [ə].
y appears only in: cmevla, lujvo glue position, letter names, hesitation. y is
NEVER stressed and NEVER counted for stress.

### Diphthongs (exactly 16 — NO triphthongs)
Falling (free use): ai [aj], ei [ɛj], oi [oj], au [aw]
Rising (words/names/borrowings): ia ie ii io iu, ua ue ui uo uu
Names only: iy [jə], uy [wə]
A diphthong is always exactly one syllable. Three-vowel sequences are diphthong +
vowel across syllables — never a single unit. (CLL §3.4)

### Special characters
- ' (apostrophe) = [h]. Intervocalic only. Realized as aspiration noise shaped by
  the following vowel's formants.
- . (period, denpa bu) = mandatory pause; shortest realization is glottal stop [ʔ].
- , (comma, slaka bu) = syllable separator. NEVER a pause. May realize as a glide.
- Capital letters in cmevla = non-standard stress on that syllable (e.g. DJOsefin.).

## 2. Permissible clusters

### 48 initial pairs (valid word-initially AND as onset in syllabification)
bl br cf ck cl cm cn cp cr ct dj dr dz fl fr gl gr jb jd jg jm jv kl kr ml mr
pl pr sf sk sl sm sn sp sr st tc tr ts vl vr xl xr zb zd zg zm zv

### Medial pair rules (CLL §3.6)
A CC pair is permissible medially iff:
1. Not doubled (no bb, cc, …)
2. Not voiced+unvoiced mix — EXEMPT: l m n r (they pair with anything legal)
3. Not both from {c j s z}
4. Not one of the specifically forbidden: cx kx xc xk mz

CCC medial: C1C2 must be a permissible pair AND C2C3 must be a permissible
INITIAL pair. Additionally, the four triples ndj ndz ntc nts are FORBIDDEN
(CLL §3.7; live example §4.7: lerldjamo takes an l-hyphen because *lerndjamo
would contain ndj).

## 3. Syllabification (CLL §3.9)
- Every vowel, diphthong, or syllabic consonant is exactly one nucleus.
- Vowel runs pair from the LEFT into nuclei (CLL §3.5: meiin. → mei,in.);
  written commas override the default pairing.
- Single C between nuclei → onset of the FOLLOWING syllable.
- CC between nuclei → split C.C, UNLESS the pair is one of the 48 initial pairs,
  in which case both go to the following syllable (.CC).
- CCC → split after the first consonant (C.CC), consistent with the
  medial-triple rule above.
- Apostrophe and comma force a syllable boundary.

### voksa's deterministic realization (chosen among CLL-valid variants)
CLL leaves two choices to the speaker; voksa fixes them deterministically:
1. **Onset maximization**: each inter-nucleus consonant run gives its maximal
   suffix that is a legal onset (every adjacent pair one of the 48 — CLL §4.7's
   fu'ivla-initial rule) to the following syllable; the rest is coda. This
   reproduces C.C / .CC / C.CC exactly for all standard words and degrades
   gracefully for cmevla clusters where CLL has "no definitive rules" (§3.9).
2. **Syllabic sonorants only where forced**: CLL §3.4/§3.9 make syllabicity the
   speaker's choice ("any l m n r may be pronounced syllabically") and
   guarantee it never affects stress. voksa makes a sonorant syllabic ONLY in
   a region with no vowel nucleus (a vowel-less comma segment: kat,r,in.; a
   vowel-less word: rl.; or a word-initial residue beyond the legal onset:
   brlgan. → b‑r̩,l̩,gan). Codas next to vowels are never split (ARM keeps rm).
   A vowel-less region with no sonorant is a typed error (no nucleus possible).

## 4. Stress (CLL §3.9 — the corrected rule)
Primary stress falls on the PENULTIMATE COUNTABLE syllable of brivla and
(by default) cmevla.

COUNTABLE excludes ALL THREE of:
1. syllables whose nucleus is y
2. syllables whose nucleus is a syllabic consonant (l m n r)
3. syllables created solely by an epenthetic buffer vowel

Verbatim CLL §3.9: "In counting syllables, however, syllables whose vowel is y or
which contain a syllabic consonant (l, m, n, or r) are never counted... Similarly,
syllables created solely by adding a buffer vowel, such as [ɪ], are not counted."
"Weak stress is required for syllables containing y, a syllabic consonant, or a
buffer vowel."

- gismu are CVCCV or CCVCV → always first-syllable stress.
- cmavo: unstressed by default. A stressed syllable immediately before a brivla
  REQUIRES a pause after it (else the words merge).
- cmevla: stressed anywhere; non-penultimate stress must be marked by capitals.
  Default (no capitals) = penultimate countable, same as brivla (CLL §4.8).
- The iy/uy diphthongs (cmevla-only) are NOT countable and never stressed:
  CLL §3.4 defines their vowel as [ə] and §3.9 requires weak stress for
  "syllables containing y".
- Capital marking: ANY capitalized character marks its containing syllable
  ("it is sufficient to capitalize the vowel letter", CLL §3.1). Cmevla:
  normative. Cmavo: legal spoken-stress annotation (CLL's own e'U examples).
  Brivla: capitals off the computed penultimate are an ERROR (§3.9 forbids a
  false primary stress in brivla). Capitals on uncountable syllables or across
  two syllables are errors.
- Note CLL §4.1: syllabic l m n r count as CONSONANTS for morphology/word-shape,
  while being excluded from STRESS counting. Two different rules; don't conflate.

Test vectors: .erNAce, VEcnu, POFygau, BRIvla, DI,ky,jvo (dikyjvo),
.ARMstrong. (armstrong), DJOsefin.

## 5. Pauses (CLL §3.3, §4.2, §4.9, §17.2)
MANDATORY:
1. Before any word beginning with a vowel (y counts as a vowel: .ybu, .y'y).
2. After any word ending in a consonant (i.e., after every cmevla).
3. Before and after non-Lojban text (zoi / la'o quotes — the pauses bracket
   the delimiter words and the quoted text, CLL §19.10).
4. BEFORE every cmevla, unless the immediately preceding word is one of the
   cmavo la, lai, la'i, doi (CLL §4.9 rule 4 — which is why those strings are
   forbidden inside cmevla).
5. Between a finally-stressed syllable and a following brivla (the
   stress-collision rule, CLL §4.9 rule 5 / §3.9). Generalized by §4.2: a
   finally-stressed word before a stressed-first-syllable word also pauses
   (le RE. NANmu). Only capital-marked cmavo can fire these in synthesis.
6. After any y-final cmavo unless another y-final cmavo follows (CLL §17.2's
   safe generalization of §4.9 rule 6's Cy rule; covers cy, .y'y, .ybu).
7. Before AND after the hesitation cmavo .y. (CLL §4.2 — the trailing pause
   is not derivable from any other rule).
OPTIONAL: anywhere else between words. The comma is NEVER a pause (§3.3).
voksa honors EVERY written period as a pause (a period marks a legal pause
wherever it appears, §3.3; writer-marked pauses merge with rule-mandated ones).
Pauses at one boundary MERGE into one (§4.2: "the pause after cy. merges with
the pause before .ibu"). A glottal stop is the minimal pause (§3.3).

### Dotside flag (--dotside, default OFF)
Community convention (not BPFK-mandated): drop the la/lai/la'i/doi exemption
in rule 4 (every cmevla gets a leading pause unconditionally) and drop the
la/doi forbidden-substring validation inside cmevla. That exemption removal is
the ONLY audio-level delta versus classic rules.

## 6. Buffer vowels (--buffer flag, default OFF)
An [ɪ]-like vowel (seeds 400/1900/2600 Hz, 35 ms, weak), acoustically distinct
from all 6 phonemic vowels, never stressed, excluded from stress counting
(CLL §3.8: "the buffering syllables are never stressed, and are not counted").
This is a SPEAKER-side accommodation; a synthesizer renders all legal clusters
exactly, so default is OFF.

voksa's deterministic realization when ON — CLL §3.8/§3.9 sanction any
per-pair choice ("partial buffering is also acceptable"); voksa implements the
"fully-buffered dialect" minus one option:
- Insert between EVERY word-internal pair of adjacent onset/coda consonants
  (including across syllable boundaries, per CLL's xapcke example).
- NO trailing buffer after a word-final consonant (CLL's full-buffer exemplars
  include one — [gɪʔ] — but partial buffering is explicitly permitted).
- Never adjacent to [h], pauses, or syllabic-consonant nuclei (those are not
  cluster members).
- Stress is resolved BEFORE insertion ("the stress remains in the same
  place"); buffer syllable spans are marked uncountable.

## 7. Word classification (minimal classifier — sufficient for stress + pauses)
- cmevla: ends in a consonant.
- ends in y → cmavo (no brivla may end in y, CLL §4.1/§4.7 — this also keeps
  solid Cy sequences from exposing a false pair).
- brivla: contains A consonant pair — permissibility NOT required (CLL §4.3's
  own qualifying example is the impermissible "sc" in bisycla) — within the
  first five letters counted after deleting y and apostrophe ("bisycla" has
  the pair via "syc"; "ro'inre'o" has "nr"), and ends in a vowel.
- cmavo: everything else (V, VV, CV, CVV, CV'V forms).
Rigorous fallback for dense text: BPFK PEG morphology (camxes); camxes-rs exists
but verify license/completeness before linking — porting the needed PEG
productions is acceptable.

## 8. Normalization
- Digits → PA cmavo: 0 no, 1 pa, 2 re, 3 ci, 4 vo, 5 mu, 6 xa, 7 ze, 8 bi, 9 so.
  Read digit-by-digit as SEPARATE words (10 = pa no; CLL §18.2 style — §4.2
  makes compound/separate identical in speech). Decimal point = pi (§18.3).
  Thousands separator = ki'o: CLL permits short-group elision (left-zero-padded,
  §18.3) but voksa always EMITS full three-digit groups (1,000 = pa ki'o no no
  no); non-canonical figure grouping (1,00) is a typed error. Mixed-base
  separator = pi'e — ":" between digits maps to it (clock times, §18.10).
  Hex A–F = dau fei gai jau rei vai (§18.10) — vocabulary only in v1, NOT
  auto-detected (letters inside figures are errors; 0x gating = future work).
- Lerfu (letterals, CLL §17.2/§17.4/§17.5): consonants = C+y (by cy dy fy gy jy
  ky ly my ny py ry sy ty vy xy zy); vowels = V+bu (.abu .ebu .ibu .obu .ubu
  .ybu); apostrophe = .y'y; h = .y'y.bu, q = ky.bu, w = vy.bu (§17.5);
  period = denpa bu; comma = slaka bu (no pause needed before bu). The written
  dots in .y'y.bu / ky.bu are exactly the vowel-initial and y-final pause
  rules firing — emit the words separately and §5 does the rest. Case shifts
  (ga'e/to'a/tau) change letter meaning, not sound: spelling is
  case-insensitive, shifts never emitted.
- zoi / la'o quoted foreign text: bracket with mandatory pauses; policy for the
  content itself = spell out via lerfu (v1 default) or pass through literally.

## 9. Prosody defaults
- Base F0 ≈ 110–120 Hz, robotic/monotone baseline.
- Declination: linear F0 fall ≈ 120 → 95 Hz across a bridi/utterance.
- Stress realization: ~1.5× duration, +10–30 Hz F0 excursion, small amplitude
  boost on the stressed syllable.
- Pause: 50–150 ms silence; word-initial-vowel pause minimally a glottal stop.
- Optional xu terminal rise (non-canonical nicety; flag-gated).

### 9.1 Pinned constants (Phase 7 implementation, voksa-core prosody.rs)
These are voksa's chosen values within the bands above (CLL mandates no
prosody; this is a documented convention):
- `DECLINATION_START_HZ = 120`, `DECLINATION_END_HZ = 95` — linear over the
  post-stretch utterance, applied ADDITIVELY (`f0 += baseline(t) − 120`) so
  the Phase-10 attitudinal overlay composes on top.
- `STRESS_DURATION_FACTOR = 1.5` — a stressed syllable's RHYME (nucleus
  onward) stretches by 1.5×; its onset consonants keep unit rate. The stretch
  window opens at `start_ms + nucleus_off_ms` (the `SyllableSpan` records the
  nucleus offset: onset consonants + [h] + any onset-side epenthetic buffer).
  Everything later shifts by the added rhyme time. (Phase-7.1 CP1 fix — see
  below.)
- `STRESS_F0_EXCURSION_HZ = +20` (middle of the 10–30 band), applied above
  the declination baseline, over the WHOLE stressed span (onset included).
- `STRESS_AMP_FACTOR = ×1.2` on formant amplitudes, whole stressed span.
- `XU_RISE_HZ = +25` — one rise event at 25% into the final syllable, ramped
  across the span remainder; later in-span events carry the rise too (else a
  following vowel event would re-set F0 down).
- Composition order: rhyme stretch → declination → stress excursion → xu rise.

Phase-7.1 CP1 fix (resolved): the original v1 stretched the WHOLE stressed
syllable, lengthening onset consonant clusters — the CP1 rater heard this as
"CC strong + long" on gismu-initial stressed syllables (pre/kla/zga/dja/DJO)
and preferred the flat baseline 8/10 in ABX. The stretch now scopes to the
rhyme only, so onsets stay crisp; excursion + amplitude stay whole-span.

Remaining v1 caveats:
- A voiceless final segment makes the xu rise inaudible (nothing voiced to
  carry it); Lojban questions ending in vowels — the normal case — are fine.
- Attitudinal overlay: see docs/research/02-architecture-v2.md §11 table
  (F0 mean/range in semitones, rate multipliers, voice quality via OQ TL FL DI AH AV).
  This is an INVENTED, documented, non-normative convention — CLL mandates none.
