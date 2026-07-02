# PHASE 0: Repository scaffolding (paste this as the first Claude Code prompt)

## Context
You are bootstrapping `voksa`, a rule-based Lojban Klatt-style TTS in Rust
targeting WASM (browser AudioWorklet) and native (cpal). This is a greenfield
repo. CLAUDE.md, PLAN.md, docs/phonology.md, docs/formants.md, and three research
reports under docs/research/ are already present — read CLAUDE.md and PLAN.md
first, skim docs/research/03-implementation-playbook.md for the phase plan.
The architecture is settled; do not re-research it.

## Task
Set up the complete development harness. No DSP code in this phase.

1. Cargo workspace:
   - crates/voksa-core  (lib, no_std + alloc; a trivial pub fn + one passing test)
   - crates/voksa-cli   (bin, depends on core; prints version)
   - crates/voksa-web   (cdylib, wasm-bindgen skeleton; compiles for
     wasm32-unknown-unknown)
   - xtask/             (bin; subcommand stubs: wasm-size, oracle,
     listening-battery — each may be a clear TODO returning an error, but the
     `oracle` subcommand must WORK: shell out to `espeak-ng -v jbo -w <out.wav>
     <text>` and write to fixtures/oracle/)
   - Workspace-level release profile for the web crate: opt-level="z", lto=true,
     codegen-units=1, strip=true, panic="abort".
2. Nix flake: dev shell with stable Rust + wasm32-unknown-unknown target,
   wasm-pack, binaryen (wasm-opt), cargo-nextest, cargo-insta, twiggy, espeak-ng.
   Follow the fenix + crane sketch in docs/research/03 §c. `nix develop` must work.
3. CI (GitHub Actions): jobs for fmt --check, clippy -D warnings, nextest,
   wasm-pack build --release, wasm-size budget stub (warn-only until Phase 9).
   Cache with Swatinem/rust-cache; toolchain via dtolnay/rust-toolchain.
4. Claude Code harness:
   - .claude/settings.json hooks: PostToolUse Edit|Write → cargo fmt on the file;
     PreToolUse Bash → block `git push` to main and `rm -rf` outside target/;
     Stop → run `cargo nextest run --workspace`, exit 2 with "tests failing, keep
     working" if red (guard against stop_hook_active loops).
   - .claude/agents/verifier.md: a verification subagent with NO Write/Edit tools.
     Default-FAIL contract: every acceptance criterion starts false and may only
     pass by opening the evidence (test output, rendered files).
   - Skill/command for /phase-start, /verify, /phase-commit per docs/research/03.
5. Git: init if needed, .gitignore (target/, .claude/worktrees/, artifacts/,
   fixtures/oracle/*.wav optionally via LFS or ignored), Conventional Commits
   noted in CONTRIBUTING.md.

## Constraints
- TDD applies even here: the workspace must have at least one real test per crate
  that CI runs. Confirm the trivial test fails first by asserting the wrong value,
  see red, fix, see green (harness smoke test).
- MIT/Apache dependencies only. No nightly.

## Acceptance criteria (show command output as evidence)
- nix develop → cargo nextest run --workspace  (green)
- cargo clippy --workspace --all-targets -- -D warnings  (clean)
- cargo fmt --all --check  (clean)
- cd crates/voksa-web && wasm-pack build --release --target web  (succeeds)
- cargo xtask oracle -- "coi munje"  → fixtures/oracle/coi-munje.wav exists,
  non-empty, RIFF header valid
- CI workflow file passes act/dry-run or is validated by inspection
- Hooks fire: demonstrate the Stop hook blocks on an intentionally broken test,
  then passes after fix

## Verification
Invoke the `verifier` subagent to confirm every criterion against evidence from
a fresh context before declaring done.

## On completion
Update PLAN.md (Phase 0 → [x], date, SHA). Commit as
`chore(repo): scaffold workspace, flake, CI, and agent harness` and tag
`phase0-complete`. Then STOP and summarize what exists for the human.
