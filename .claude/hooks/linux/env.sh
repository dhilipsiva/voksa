#!/bin/sh
# Put cargo and nix on PATH for non-interactive shells (hooks, wsl.exe calls).
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
[ -f /etc/profile.d/nix.sh ] && . /etc/profile.d/nix.sh
[ -f /etc/profile.d/nix-daemon.sh ] && . /etc/profile.d/nix-daemon.sh
PATH="$HOME/.cargo/bin:$PATH"
export PATH
