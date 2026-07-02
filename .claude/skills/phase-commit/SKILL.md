---
name: phase-commit
description: Finalize a voksa phase - verify, update PLAN.md, conventional commit, tag the milestone.
argument-hint: [phase-number]
disable-model-invocation: true
---

Finalize Phase $0 of voksa.

1. Run the /verify battery first. Abort immediately if any criterion fails.
2. Listening checkpoints: if Phase $0 is 7, 10, or 11, STOP after rendering the battery — a human must A/B the WAVs against the eSpeak-jbo oracle and record MOS/ABX results in docs/listening/phase$0.md before anything is tagged. The human tags checkpoint milestones, not you.
3. Update PLAN.md: mark Phase $0 `[x]`, record date, commit SHA, and any deviations from the playbook; append a session-log line (date, phase, sessions used, notes).
4. Commit everything with a Conventional Commit message scoped to the phase (e.g. `feat(core): ...`, `chore(repo): ...`).
5. Create an annotated tag `phase$0-complete`. Do not push unless the user asks.
