#!/usr/bin/env bash
#
# Self-test of tests/bench/hook-latency.sh, the benchmark of Q-5.
#
# Each case runs the benchmark over a few calls with a stand-in for nuc behind the reference shim:
# one that answers at once passes a generous limit, one that sleeps a tenth of a second fails the
# limit of Q-5, one that refuses fails as a benchmark that cannot run, and an unknown argument or
# a count that is no number is refused. Timing alone decides no case: the margins are a hundred
# times the noise of a runner.
#
# Verifies: Q-5
# Inputs: tests/bench/hook-latency.sh shims/reference-transaction scripts/lib/
#
# Usage:
#     scripts/test-bench-hook-latency.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
BENCH="$ROOT/tests/bench/hook-latency.sh"

# stand_in <name> <shell code> — a nuc that runs the code, printed as its path.
stand_in() {
  printf '#!/bin/sh\n%s\n' "$2" > "$work/$1"
  chmod +x "$work/$1"
  echo "$work/$1"
}

# expect <exit code> <name> <needle> <args...> — run the benchmark; it exits as told, with the
# needle in its output.
expect() {
  local want="$1" name="$2" needle="$3" out rc
  shift 3
  out="$(bash "$BENCH" "$@" 2>&1)"; rc=$?
  if ((rc == want)) && [[ "$out" == *"$needle"* ]]; then
    passed=$((passed + 1))
    return
  fi
  failed=$((failed + 1))
  echo "FAIL: $name (expected exit $want with \"$needle\", exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
}

expect 0 "a nuc that answers at once passes" "Hook latency passed" \
  --nuc "$(stand_in fast 'exit 0')" --calls 10 --limit-ms 50
expect 1 "a nuc slower than the limit fails" "more than 2 ms (Q-5)" \
  --nuc "$(stand_in slow 'sleep 0.1; exit 0')" --calls 5
expect 1 "a nuc that refuses the update fails" "the shim refused" \
  --nuc "$(stand_in refusing 'exit 1')" --calls 3
expect 2 "an unknown argument is refused" "unknown argument" --nuc "$work/fast" --fast
expect 2 "a count that is no number is refused" "positive number" --nuc "$work/fast" --calls x

summary "Hook latency benchmark self-test"
