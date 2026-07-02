---
name: phase-start
description: Start a voksa implementation phase - restate goal, acceptance criteria, and constraints, then plan.
argument-hint: [phase-number]
disable-model-invocation: true
---

Start Phase $0 of voksa. Do not write any code during this command.

1. Read PLAN.md and run `git log --oneline -5` + `git tag`. Confirm Phase $0 is the next unstarted phase (all prior phases tagged `phaseN-complete`). If not, stop and report the discrepancy.
2. Read the Phase $0 row in the phase table of docs/research/03-implementation-playbook.md §d, and the fully-written phase prompt if one exists (§e; docs/phase0-prompt.md for Phase 0).
3. Restate for the user: the phase goal, key deliverables, every acceptance criterion, the constraints that apply (TDD red-first, schedule determinism, MIT/Apache deps only, no nightly, metric units), and whether this phase ends in a listening checkpoint (Phases 7, 10, 11 do).
4. Enter plan mode and produce an implementation plan before touching any code.
