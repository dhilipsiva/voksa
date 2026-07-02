# Contributing to voksa

voksa is built in 12 sequential, verifiable phases (see PLAN.md — the live
status table — and docs/research/03-implementation-playbook.md for the full
plan). Read CLAUDE.md first; it holds the project map and the exact commands.

## Ground rules

- **Conventional Commits.** `feat(core): ...`, `test(front-end): ...`,
  `chore(repo): ...`, `docs: ...`. Each phase ends in one milestone commit
  tagged `phaseN-complete`.
- **main is always green.** Phases are strictly sequential; do not start
  Phase N+1 before Phase N is tagged. Greenness is enforced by the Stop
  test-gate hook, the /verify battery, and CI.
- **Commits auto-push.** Every commit is pushed to origin automatically
  (hook-enforced). Force pushes are blocked — history rewrites on a pushed
  main are a deliberate human act.
- **TDD, red first.** Write failing tests, run them, see red, commit the
  failing tests, then implement until green. Never modify a test to make it
  pass.
- **Determinism.** The schedule compiler must be deterministic; snapshot
  schedules with insta. Never bit-compare rendered WAVs — acoustic assertions
  use tolerance bands.
- **Licensing.** MIT/Apache-2.0 dependencies only. eSpeak NG (GPLv3) is an
  out-of-process regression oracle; never copy its code, phoneme tables, or
  data. All linguistic rules come from the CLL.
- **Listening checkpoints.** Phases 7, 10, and 11 end with a human listening
  review (`cargo xtask listening-battery`, results logged to docs/listening/).
  A human signs off and tags those milestones.

## Dev environment

`nix develop` provides the pinned toolchain (stable Rust + wasm32 target,
wasm-pack, binaryen, cargo-nextest, cargo-insta, twiggy, espeak-ng).
