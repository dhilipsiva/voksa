#!/bin/sh
# Stop hook: refuse to end the turn while workspace tests are red.
# Exit 2 => Claude must keep working (stderr becomes the instruction).
# Loop guard: a consecutive-failure counter (reset on green) allows the stop
# after 5 blocks so a genuinely stuck session can end — `stop_hook_active`
# alone cannot distinguish "still fixing" from "hopeless".
. "$(dirname "$0")/lib.sh"

cat >/dev/null # consume hook JSON; the counter replaces stop_hook_active logic

count_file="$(dirname "$0")/.stop-gate-count"
log_file="${TMPDIR:-/tmp}/voksa-stop-gate.log"

if run_linux_helper .claude/hooks/linux/test.sh >"$log_file" 2>&1; then
    rm -f "$count_file"
    exit 0
fi

n=$(cat "$count_file" 2>/dev/null || echo 0)
n=$((n + 1))
printf '%s' "$n" >"$count_file"

if [ "$n" -ge 5 ]; then
    rm -f "$count_file"
    echo "stop-gate: tests still red after $n consecutive blocks; allowing stop. Fix tests before any commit."
    exit 0
fi

{
    echo "tests failing, keep working (stop-gate block $n/5). Tail of test output:"
    tail -20 "$log_file"
} >&2
exit 2
