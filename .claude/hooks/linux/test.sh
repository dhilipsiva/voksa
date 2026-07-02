#!/bin/sh
. "$(dirname "$0")/env.sh"
if command -v cargo-nextest >/dev/null 2>&1; then
    cargo nextest run --workspace
elif command -v nix >/dev/null 2>&1; then
    nix develop --command cargo nextest run --workspace
else
    cargo test --workspace
fi
