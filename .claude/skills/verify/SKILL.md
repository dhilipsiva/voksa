---
name: verify
description: Run the voksa acceptance battery and have the verifier subagent grade it from a fresh context.
disable-model-invocation: true
---

Run the full acceptance battery, showing each command's real output as evidence. All commands run inside the nix dev shell in WSL; from a Windows-side session wrap each as:
`wsl.exe -d Ubuntu --cd /home/dhilipsiva/projects/dhilipsiva/voksa -- bash -lc "nix develop --command <cmd>"`

1. `cargo nextest run --workspace`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo fmt --all --check`
4. `wasm-pack build --release --target web crates/voksa-web`
5. Every phase-specific criterion from the current phase's prompt (FFT/LPC formant assertions, insta snapshots, proptest, oracle render, wasm-size budget, listening-battery render — whichever apply).

Then invoke the `verifier` subagent, passing it the current phase's full acceptance-criteria list. Every criterion starts FAIL; only the verifier's own evidence flips one to PASS. Relay its PASS/FAIL table verbatim.

Do not declare the phase done unless the verifier's verdict is "all criteria PASS". If anything fails, fix and re-run /verify — never weaken a test or a criterion to make it pass.
