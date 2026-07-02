#!/bin/sh
# Shared helpers for voksa Claude Code hooks. POSIX sh, no jq.
#
# Sessions run either inside WSL/Linux or Windows-side (Git Bash) against
# \\wsl.localhost — cargo/nix commands must always execute inside Linux, so
# helpers dispatch through wsl.exe when not on Linux.

WSL_DISTRO="${VOKSA_WSL_DISTRO:-Ubuntu}"
WSL_REPO="${VOKSA_WSL_PATH:-/home/dhilipsiva/projects/dhilipsiva/voksa}"

# Run a repo-relative helper script (POSIX path, no leading slash) inside the
# Linux environment, from the repo root.
run_linux_helper() {
    if [ "$(uname -s)" = "Linux" ]; then
        sh "$1"
    else
        # Git Bash rewrites /foo args into Windows paths; disable for wsl.exe.
        MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*' \
            wsl.exe -d "$WSL_DISTRO" --cd "$WSL_REPO" -- bash -lc "sh $1"
    fi
}
