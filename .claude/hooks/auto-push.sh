#!/bin/sh
# PostToolUse(Bash): after a successful `git commit` or `git tag`, push the
# current branch plus reachable annotated tags to origin. A command that merely
# mentions git commit only causes a harmless up-to-date push. Push failures
# exit 2 so the session sees them instead of silently diverging from origin.
. "$(dirname "$0")/lib.sh"

input=$(cat)
case "$input" in
    *'git commit'* | *'git tag'*) ;;
    *) exit 0 ;;
esac

log="${TMPDIR:-/tmp}/voksa-auto-push.log"
if run_linux_helper .claude/hooks/linux/push.sh >"$log" 2>&1; then
    exit 0
fi
{
    echo "auto-push failed — origin is now behind; push manually or fix auth:"
    tail -5 "$log"
} >&2
exit 2
