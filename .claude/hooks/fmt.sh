#!/bin/sh
# PostToolUse(Edit|Write): format Rust code after edits.
# Reads the hook JSON on stdin; only acts when a .rs file_path is involved.
# `cargo fmt --all` rather than per-file: exact path extraction needs a JSON
# parser (jq is not guaranteed on Git Bash), and the workspace is small.
. "$(dirname "$0")/lib.sh"

input=$(cat)
case "$input" in
    *'.rs"'*)
        run_linux_helper .claude/hooks/linux/fmt.sh >/dev/null 2>&1
        ;;
esac
exit 0
