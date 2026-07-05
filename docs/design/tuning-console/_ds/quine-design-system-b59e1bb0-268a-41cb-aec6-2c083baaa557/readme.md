# dhilipsiva — Design System

> *Build systems that outlive their meaning. One must imagine the compiler happy.*

This is the personal design system of **dhilipsiva** — an **optimistic-nihilist hacker-scientist** equally at home in Rust, Python, FOSS, WebAssembly, WebRTC, Web3, distributed systems, and symbolic reasoning. The thesis: *the universe is indifferent, so author your own fixed point.*

It is a **terminal-first, scientific** identity: a deep "void" canvas, a signature **rust-ember** accent (a deliberate homage to Rust), a **phosphor** mint for the "alive / node" signal, and real mathematical & logic glyphs (`∀ ∃ λ ⊢ ⊥ ∴`) used as iconography. Monospace is the default voice, not a code-block special case. The recurring conceptual motif is the **fixed point** — `f(x) = x`, the quine, distributed consensus — meaning that bootstraps itself from nothing.

### Sources
This system was authored from a written brand persona and the brand name **dhilipsiva** — **no external codebase, Figma, or asset library was provided.** Every token, component, and screen here is original. The UI kits showcase a sample open-source runtime called **fixpoint** as dhilipsiva's demonstration project. *(See "Open questions" at the bottom — a real product surface, **nibli / the Transparency Triad**, is pending materials.)*

---

## CONTENT FUNDAMENTALS — the voice

Dry, precise, quietly hopeful. State facts, then imply the optimism rather than performing it. Lab notebook crossed with a stoic aphorism.

- **Casing:** Mono UI labels are `lowercase` or `UPPERCASE` with wide tracking (eyebrows, status chips, table headers). Display headlines are sentence case. Never Title Case Everything.
- **Person:** Imperative and impersonal. "Build anyway." "Quorum lost. Retrying."
- **Numbers are the heroes.** Lead with measured quantities in mono — `38ms`, `5/5 ack`, `12kB`, `term 7`. Precision is the flex.
- **The `//` comment prefix** is a connective device for eyebrows and asides — `// why it exists`, `// fixed point of eval`.
- **Symbolic flourishes** appear where they're *true*, not decorative: `∀ peers: agree`, `eval(q) = q`, `∴ leader`.
- **Optimistic-nihilist register** for the big moments, borrowed from Camus, sincerely: *"One must imagine the cluster happy."* Use sparingly, once per surface.
- **No emoji.** Ever. The glyph vocabulary is mathematical Unicode, not 🚀😢.
- **Errors are deadpan, never cute.** "Partition detected — quorum lost on shard 3." Not "Oops!".

**Say this** → `Nothing is owed. Build anyway.` · `5 peers. 38ms. Committed.`
**Not this** → ~~`Unlock your potential today! 🚀`~~ · ~~`Oops! Something went wrong 😢`~~

---

## VISUAL FOUNDATIONS

**Color.** Dark-first. The background is the **void** (`#0B0B10`), an almost-black with a faint blue cast; surfaces stack upward through `void-800 → 700 → 600`. Foregrounds are **warm off-whites**, never pure `#FFF` (`--paper-100 #F4F3EF`). The single primary accent is **ember** (`#F2542D`) — action, focus, links, the "leader/active" marker. **Phosphor** mint (`#38E3A6`) is the life signal: healthy nodes, success, online. **Quanta** violet (`#8B7CF6`) is reserved and flat — symbolic/Wasm/Web3 nods only, never gradient slop. Status uses amber/sky/crimson. A `[data-theme="light"]` ("daylight nihilism") scope inverts to paper-and-ink while ember keeps burning.

**Type.** Mono-forward. **IBM Plex Mono** is the signature workhorse (UI default, data, code, labels). **IBM Plex Sans** carries longer reading (body, forms). **Space Grotesk** is the display face for loud headlines (tracking −3%). **IBM Plex Serif**, italic, is for philosophical pull-quotes. Scale is a 1.25 major third, 11 → 88px.

**Spacing & layout.** A strict **4px grid**. Containers cap at 1120–1200px. Controls are 36px default (28 sm / 44 lg). Layouts favor CSS grid with explicit `gap`. Fixed elements: a sticky glass top-nav on marketing; a fixed left sidebar in the console.

**Backgrounds.** No photography, no gradients-as-decoration. The recurring motif is the **blueprint grid** (`--bg-grid`, a 24px hairline lattice), radial-masked behind heroes/CTAs and painted solid behind "engineered" panels and the topology map. Node dots glow against it.

**Borders & corners.** Hairline `1px` borders in `void-500`; bold `2px` for emphasis. Radii are **tight and engineered** — 2–6px on controls, 10px on cards, 16px max. The "active/selected" affordance is a **2px ember top-rule** on a card, not a fill.

**Elevation.** Shadows are restrained and cool (`shadow-1..3`). The *signature* lift is not a drop shadow but a **colored glow** — `--glow-ember` / `--glow-phosphor` — a 1px ring + soft colored bloom. Used on hovered primary buttons, leader nodes, alive indicators.

**Transparency & blur.** Glass (`backdrop-filter: var(--blur-glass)`) appears on the sticky nav only. Otherwise surfaces are solid.

**Motion.** Crisp and deterministic. Default easing `--ease-out (cubic-bezier(.2,.8,.2,1))`; durations 80/140/220/360ms. One **optimistic spring** (`--ease-spring`, slight overshoot) is reserved for confirmations — toggle thumbs, checkbox ticks. Decorative loops avoided except the **blinking cursor** and the **scanning spinner**. All motion collapses under `prefers-reduced-motion`.

**Hover / press.** Hover = lighter surface (`--surface-hover`) or border shifting to ember; primary buttons gain their glow. Press = a 1px `translateY` nudge (physical, not a color flip). Focus is always a 2px ember ring offset from the background.

**Imagery vibe.** Essentially no photography. Where imagery exists it is data: node graphs, terminal output, code. Cool, high-contrast, phosphor-on-void. Any real imagery should be cool/desaturated with optional grain — never warm stock photos.

**Cards.** Solid `--surface-card` (`#181822`), 1px subtle border, 10px radius, clipped. Optional blueprint grid, ember top-rule (active), or ember glow on hover. No drop-shadow by default — the border does the work.

---

## ICONOGRAPHY

A deliberately **two-layer** system:

1. **Mathematical Unicode glyphs in IBM Plex Mono** are the *primary* icon language — `∀ ∃ λ ⊢ ⊥ ⊤ ∴ ≡ ⇒ → ⟂ ∘ ⊗ ∇ ∑ ⎇`. They render in the brand font, scale with type, recolor with `currentColor`, and are authentic to a symbolic-reasoning audience. Use for callout markers (`∴` theorem, `⊥` halt), feature glyphs, and conceptual nav.
2. **[Lucide](https://lucide.dev)** is the **adopted** line-icon set for conventional UI affordances (search, settings, git-branch, terminal, play, lock, chevrons…). It's FOSS, geometric, and ships a ~1.75px stroke that matches the engineered feel. Load via CDN: `https://unpkg.com/lucide@0.456.0/dist/umd/lucide.min.js`, place `<i data-lucide="search"></i>`, then call `lucide.createIcons()`. Icons inherit `currentColor` and size via CSS. See the **Iconography** brand card for both layers side by side.

> If you'd prefer **Tabler** or **Phosphor** instead of Lucide, it's a one-line CDN swap — say the word. **No emoji** is used anywhere.

### Assets in `assets/`
- `mark.svg` — the logomark: two chevrons converging on a phosphor fixed point (consensus / recursion / `f(x)=x`), ember strokes in a void tile.
- `mark-mono.svg` — single-color mark using `currentColor` (light backgrounds, favicons, inline).
- `lockup.svg` — horizontal mark + "dhilipsiva" wordmark + tagline.

---

## FONTS — what's shipped & what to do

**Fonts load from the Google Fonts CDN** (`tokens/fonts.css`, via `@import`). All four families (IBM Plex Mono / Sans / Serif, Space Grotesk) are OFL / open-source, matching the FOSS ethos. **No local `.woff2` binaries are shipped**, so the compiler reports "Fonts: none" (it counts literal `@font-face` rules, not remote imports).

- **For prototyping / online use: do nothing.** The CDN import works as-is (you can see it rendering in every card).
- **For production / offline / self-hosting:** drop the `.woff2` files into `assets/fonts/` (use the **Import** menu to upload them — I can't fetch font binaries from here), and I'll replace the `@import` line with local `@font-face` rules. Just confirm and attach.

---

## INDEX — what's in this project

**Global entry**
- `styles.css` — the one file consumers link. An `@import` manifest only.

**Tokens** (`tokens/`, all reached from `styles.css`)
- `fonts.css` · `colors.css` · `typography.css` · `spacing.css` · `effects.css` · `motion.css` · `base.css` (reset + utilities `.q-label`, `.q-blueprint`, `.q-comment`, `.q-cursor`)

**Components** (`components/`) — bundled to `window.QUINEDesignSystem_b59e1b` (internal namespace id; brand-agnostic)
- `core/` — Button, IconButton, Badge, Card, Callout, Avatar
- `forms/` — Input, Select, Checkbox (+radio), Switch
- `navigation/` — Tabs
- `feedback/` — Spinner, Tooltip
- Each has `<Name>.jsx`, `<Name>.d.ts`, `<Name>.prompt.md`; each directory has a `*.card.html` thumbnail.

**UI kits** (`ui_kits/`)
- `landing/` — `dhilipsiva.dev` landing page for the sample **fixpoint** runtime (`LandingPage`).
- `console/` — cluster monitoring app (`ConsoleApp`).

**Guidelines** (`guidelines/`) — Design System tab specimen cards: Type (5), Colors (5), Spacing (3), Brand (5, incl. Logo & Iconography).

**`q-loader.js`** — preview-time loader. The compiler emits `_ds_bundle.js` for *consuming* projects, but that virtual artifact isn't served inside this authoring project's file-preview, so cards/screens would render blank. `q-loader.js` fetches the real component source, transpiles it in-browser with Babel, and populates the namespace — so every card/screen renders here too. Consuming projects ignore it and use the real bundle.

**`SKILL.md`** — makes this usable as a downloadable Claude Agent Skill.

---

## OPEN QUESTIONS / TODO

- **nibli — the Transparency Triad** (real product surface): I have no access to prior conversations, so I can't reconstruct it. Send the codebase, Figma, screenshots, or a written description and I'll build a third UI kit that recreates it faithfully.
- **Fonts:** confirm CDN is fine, or upload `.woff2` to self-host.
- **Icons:** Lucide adopted — confirm, or swap to Tabler/Phosphor.

*The void, organized.*
