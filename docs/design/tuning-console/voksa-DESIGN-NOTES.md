# voksa redesign — working plan (resume point)

## EMBED CONTRACT (user requirement — carry into Component Spec + all handoff docs)
dhilipsiva.dev already has a site navbar. The console ships NO navbar / brand
chrome of its own: no logo, no "voksa" wordmark header, nothing sticky at
top:0. Top element is a slim non-sticky page-title row: display-font "voksa"
(22px, the page h1 — bigger than the label) + q-label
"tuning console" + version chip + Δ chip + demo/about/theme buttons; it
sits under the site nav; rack-nav stickiness assumes it may scroll under the
HOST navbar (offsets are page-relative: left col top:12px, rack nav top:8px).
Theme should follow the site (initialTheme prop / inherit data-theme);
in-console ◐ toggle is a preview convenience, droppable in the port.
Load/export live ONLY in the share card (topbar duplicates removed).

Repo explored (local folder `voksa/`). All real data extracted into
`voksa-engine-data.json` (449-float layout, descriptors, presets, 18 sentences,
demo config w/ out-of-range a.f1_hz=1400 to demo range-widening, capabilities,
sizes, links) + `voksa-help-text.json` (skeleton registry, user fills via
Claude Code; UI falls back to "// help pending — <key>"). DS mark copied to
assets/voksa-mark.svg.

## User answers
- 1 voice-table pattern, fully worked: phoneme grid navigator + per-phoneme editor
- Silent prototype (visual states only; waveform = deterministic mock PCM)
- English primary labels, Lojban subtext (only REAL Lojban from repo: karsna,
  zunsna, cnima'o, "lo voksa selci"); help "?" on every control = skeleton only
- One responsive DC + phone shown in device frame alongside
- Representative-subset data OK (I have full real data anyway)
- Wants an "about voksa" surface: capabilities + gated sizes (42/43 kB gate)
- deliverable_shape/prototype_depth/key_states: decide → interactive workbench
  + working state model (dirty tracking, delta export, replace-on-load,
  range-widening) + demo-state menu (fresh/dirty/loaded/flat/wasm-fail)

## Files plan (all at project ROOT so _ds/ links resolve)
1. `Voksa Workbench.dc.html` — main deliverable. NEXT UP (nothing written yet).
2. `Voksa IA & Flows.dc.html` — personas (listener/tuner/reporter), IA map, contracts.
3. `Voksa Phone Preview.dc.html` — ios_frame + iframe of workbench @390.
4. `Voksa Component Spec.dc.html` — Dioxus handoff: ParamSlider/FlagChip/
   PhonemeCell/TranscriptLine/Rack/etc, states (default/modified/disabled/
   widened), tokens, behavior notes.

## Workbench design (decided)
- Topbar sticky glass 52px: mark + "voksa" + q-label "tuning console", Badge
  `v0.1.0 · wasm 42 kB`, status; right: demo-states menu (ghost), about λ,
  theme ◐ toggle, Load/Export buttons.
- Grid ≥1080: left sticky source col 384px / right tuning col. ≤720: single
  col + fixed bottom dock (▶ speak + status), racks stack.
- Left col cards: (1) utterance: big mono text input, sentence Select+⟳ next
  +`NN/18`+gloss, 4 flag chips (flat/xu/dotside/buffer; custom FlagChip);
  (2) transcript card: token spans (stress=strong, ‖=ember, (ɪ)=phosphor,
  dots=faint) + legend "?" popover; (3) transport: Speak (primary lg), Switch
  "speak on change", waveform canvas (phosphor on void-800+blueprint), status
  line (Spinner when speaking), download WAV ghost; (4) share: delta Badge
  `Δ n`, notes textarea, Export/Load (drop zone), CTA github issue + mailto,
  loaded-config Badge alive.
- Right col: sticky rack-nav (anchor chips + preset Select + reset all).
  Racks: prosody (7 rows), naturalness (explainer + A/B segmented
  current↔off listening compare, 9 rows), attitudinals (∴ theorem Callout
  "invented, non-normative", 7 chips w/ mod dots → editor 8 rows + try-it +
  per-emotion reset), voice table (grid navigator by class + changed-only
  Switch + editor: formant matrix 3×3 mini-cells + voicing/asp/duration rows;
  stops = closure block + burst block + timing; dur-kind = single row + note;
  per-phoneme reset + reset-all-voice).
- ParamSlider: grid 150px|1fr|84px; custom track (default tick, ember delta
  fill default→thumb, 13px thumb) + native range opacity-0 on top (a11y);
  value = editable input (drafts state, Enter commits, can exceed range →
  widen + step 'any' + ⤢ marker); mod dot + hover ↺ reset; dblclick reset;
  disabled via flat. Helmet CSS only for: range pseudo resets, :focus-within
  ring, hover scale, media queries, scrollbars, @keyframes.
- Logic: state {booted,failed,text,flags,knobs{16},atts{7×[8]},voice{27
  items:flat arrays},ranges{path:{min,max,step}},notes,preset,selEmotion,
  selPhoneme,vtChangedOnly,abOff,autoSpeak,speaking,status,lastRender,
  loadedName,theme,aboutOpen,demoOpen,legendOpen,help{key,x,y},drafts}.
  paths: `k.rate` / `a.ui.3` / `v.a.5`. fround-compare for mod. Export =
  reference schema exactly (flat knobs, delta atts/phonemes w/ closure/burst
  nesting, phonetics, notes, sampleRate, voksaVersion). applyConfig REPLACE
  semantics + widening (setSliderExact port). Presets = reset-all + knob
  overrides (reference behavior). Boot: fetch 2 JSONs in componentDidMount,
  brief 'compiling voksa_web_bg.wasm…' state; fail → error Callout + retry
  (speak disabled, sliders alive).
- Mock transcriber (approx of voksa_transcribe, documented): syllabify,
  penultimate stress CAPS (y excluded, input CAPS force stress, ','
  syllable breaks, digits→PA cmavo incl pi/ki'o), cmevla→‖, buffer→(ɪ);
  emits tokens + seg list reused by mock PCM render (durations/amps read
  LIVE voice-table + knob values so waveform responds; seeded PRNG; 8 kHz;
  WAV download works, labeled mock).
- Auto-speak: 400ms debounce on every mutator when enabled.
- data-screen-label on zones; help "?" everywhere reading HELP registry.
- Root props (tweaks): density cozy|compact, lojbanSubs bool, defaultTheme.

## QUINE usage notes
x-import global `QUINEDesignSystem_b59e1b.<Name>`; keep x-imports OUT of
sc-for loops. Button(variant primary/secondary/ghost/danger, size sm/lg,
block), IconButton(label), Badge(tone neutral/ember/alive/danger, dot),
Callout(kind note/ok/warn/error/theorem, title), Switch(label,checked,
onChange), Select(options[{value,label}]|strings, value, onChange), Input,
Checkbox, Spinner(tone alive,label), Tabs. Tokens as in _ds/tokens. Glass
only on sticky topbar. Focus ring var(--ring). No emoji; glyphs ∀∃λ⊢⊥∴≡⎇.
Helmet: 8 token/style links + _ds_bundle.js (root-relative "_ds/...").
