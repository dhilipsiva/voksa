# voksa handoff bundle

Files to seed the new `voksa` repository before the first Claude Code session.

## Layout in the new repo

```
voksa/
├── CLAUDE.md                  ← from this bundle (root, not docs/)
├── PLAN.md                    ← from this bundle (root)
└── docs/
    ├── phonology.md           ← from this bundle
    ├── formants.md            ← from this bundle
    ├── phase0-prompt.md       ← from this bundle
    └── research/
        ├── 01-architecture-v1.md          ← report 1 (first research round)
        ├── 02-architecture-v2.md          ← report 2 (verified v2 — wins conflicts)
        └── 03-implementation-playbook.md  ← report 3 (12-phase Claude Code plan)
```

Copy the three research reports out of the Claude chat artifacts into
docs/research/ with those names — CLAUDE.md and the phase prompts reference
them by path.

## First session

1. `git init voksa && cd voksa`, place the files as above, initial commit.
2. Open Claude Code (Fable 5, effort high) in the repo.
3. Paste docs/phase0-prompt.md as the first prompt.
4. Subsequent phases: use the reusable phase-prompt template in
   docs/research/03-implementation-playbook.md §e, filling in the per-phase
   acceptance criteria from its phase table (§d). Phases 2 and 7 already have
   fully-written prompts in that report.

## Name

voksa — Lojban gismu, x1 is the voice/speech sounds of x2. Penultimate stress:
VOK.sa. Companion to nibli (NI.bli): nibli reasons, voksa speaks.
Verify github.com/dhilipsiva/voksa and crates.io/crates/voksa are free before
creating (unverified as of this handoff).
