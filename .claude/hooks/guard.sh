#!/bin/sh
# PreToolUse(Bash): guard rail, not a security boundary.
# Blocks (exit 2, stderr shown to Claude):
#   - force pushes (commits auto-push fast-forward; history rewrites on a
#     pushed main stay a deliberate human act)
#   - recursive rm that is not confined to target/
# Reads the hook JSON on stdin. The command value is inspected as the slice of
# JSON after "command":" — a JSON-escape-aware parser is overkill here and a
# rare false positive only produces a visible, recoverable block.

input=$(cat)
cmd=${input#*'"command":"'}
[ "$cmd" = "$input" ] && exit 0 # no Bash command field found

if printf '%s' "$cmd" | grep -Eq 'git[[:space:]]+push'; then
    if printf '%s' "$cmd" | grep -Eq -- '--force|[[:space:]]-f([[:space:]]|\\?"|$)'; then
        echo "Blocked: force pushes are not allowed — rewrite history only as a deliberate human act." >&2
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
