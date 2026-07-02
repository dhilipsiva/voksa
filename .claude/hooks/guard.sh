#!/bin/sh
# PreToolUse(Bash): guard rail, not a security boundary.
# Blocks (exit 2, stderr shown to Claude):
#   - `git push` that targets main, or a bare `git push` (could target main)
#   - recursive rm that is not confined to target/
# Reads the hook JSON on stdin. The command value is inspected as the slice of
# JSON after "command":" — a JSON-escape-aware parser is overkill here and a
# rare false positive only produces a visible, recoverable block.

input=$(cat)
cmd=${input#*'"command":"'}
[ "$cmd" = "$input" ] && exit 0 # no Bash command field found

if printf '%s' "$cmd" | grep -Eq 'git[[:space:]]+push'; then
    if printf '%s' "$cmd" | grep -q 'main'; then
        echo "Blocked: pushing to main is not allowed — main only moves via CI-green, human-reviewed phase merges." >&2
        exit 2
    fi
    if printf '%s' "$cmd" | grep -Eq 'git[[:space:]]+push[[:space:]]*(\\?"|$)'; then
        echo "Blocked: bare 'git push' may target main; name an explicit non-main branch." >&2
        exit 2
    fi
fi

if printf '%s' "$cmd" | grep -Eq '(^|[;&| ])rm[[:space:]]+-[a-zA-Z]*[rR]'; then
    if ! printf '%s' "$cmd" | grep -Eq 'rm[[:space:]]+-[a-zA-Z]+[[:space:]]+(\./)?target(/|[[:space:]]|\\?"|$)'; then
        echo "Blocked: recursive rm outside target/ is not allowed. Delete specific files, or rm -rf target/ only." >&2
        exit 2
    fi
fi

exit 0
