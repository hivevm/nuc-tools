#!/usr/bin/env bash
#
# Benchmark of Q-5: what 'nuc hook reference-transaction' adds per call over a no-op hook, as the
# median over a number of calls, and a failure where it adds more than the limit. Run by
# .github/workflows/benchmark.yml on Linux x64, on a schedule, and never by scripts/check-all.sh or
# a hook: the timing of a shared runner moves without a change (ADR-0008). It measures and gates
# nothing, so it stands with the tests and not with the checks under scripts/.
#
# Both hooks run as git runs them: in a fixture repository whose origin/main is set, as the
# executable git would start, with the 'prepared' state as their argument and one update on their
# standard input, the commit of a branch, the update almost every git command makes. The reference
# shim under shims/ hands it to nuc; the no-op hook is a POSIX sh script that exits 0, so the two
# differ by nuc alone. The calls alternate, so that a drift of the machine meets both alike, and
# each is timed by bash's EPOCHREALTIME, which costs no process. A second set, an update of main
# to where it already is, as pack-refs makes, is reported and not gated.
#
# nuc is built from the working tree by 'cargo build --release --locked', the build the release
# ships, unless --nuc names a binary.
#
# Pure bash 5 + coreutils/git, and cargo for the build. Usage:
#     tests/bench/hook-latency.sh [--nuc <binary>] [--calls <n>] [--limit-ms <ms>]
# Defaults: 1,000 calls, a limit of 2 ms (Q-5).
# Exit code 0 when the median adds no more than the limit, 1 when it adds more or the benchmark
# cannot run, 2 on an unknown argument.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

die() { echo "Hook latency: $1" >&2; exit 2; }

nuc="" calls=1000 limit_ms=2
while (($#)); do
  case "$1" in
    --nuc) nuc="${2:-}"; shift 2 || die "--nuc takes a binary" ;;
    --calls) calls="${2:-}"; shift 2 || die "--calls takes a number" ;;
    --limit-ms) limit_ms="${2:-}"; shift 2 || die "--limit-ms takes a number" ;;
    *) die "unknown argument '$1'" ;;
  esac
done
[[ "$calls" =~ ^[1-9][0-9]*$ ]] || die "--calls takes a positive number, not '$calls'"
[[ "$limit_ms" =~ ^[0-9]+(\.[0-9]+)?$ ]] || die "--limit-ms takes a number of milliseconds, not '$limit_ms'"
[[ -n "${EPOCHREALTIME:-}" ]] || { echo "Hook latency: bash 5 or later is needed for EPOCHREALTIME" >&2; exit 1; }

# shellcheck disable=SC2046 # one variable name per word
unset $(git rev-parse --local-env-vars 2>/dev/null)

if [[ -z "$nuc" ]]; then
  (cd "$ROOT" && cargo build --release --locked -q) || { echo "Hook latency: nuc could not be built" >&2; exit 1; }
  nuc="${CARGO_TARGET_DIR:-$ROOT/target}/release/nuc"
fi
[[ -x "$nuc" ]] || { echo "Hook latency: '$nuc' is no executable" >&2; exit 1; }

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/bin" "$work/repo"
ln -s "$(realpath "$nuc")" "$work/bin/nuc"
printf '#!/bin/sh\nexit 0\n' > "$work/noop"
chmod +x "$work/noop"
repo="$work/repo"
git -C "$repo" init -q -b main
git -C "$repo" -c user.name=b -c user.email=b@example.invalid -c core.hooksPath=/dev/null commit -q --allow-empty -m init
head="$(git -C "$repo" rev-parse HEAD)"
git -C "$repo" update-ref refs/remotes/origin/main "$head"
zero="$(printf '0%.0s' {1..40})"

# median_us <micros...> — the median of the times, in microseconds.
median_us() { printf '%s\n' "$@" | sort -n | awk '{ a[NR] = $1 } END { print (NR % 2) ? a[(NR + 1) / 2] : int((a[NR / 2] + a[NR / 2 + 1]) / 2) }'; }

# elapsed_us <hook> <input> — one call of the hook as git makes it, timed; fails with the hook.
elapsed_us() {
  local start end
  start="$EPOCHREALTIME"
  (cd "$repo" && env -i PATH="$work/bin:$PATH" HOME="$HOME" "$1" prepared <<< "$2" > /dev/null 2>&1) || return 1
  end="$EPOCHREALTIME"
  echo $(( ${end/./} - ${start/./} ))
}

# measure <label> <input> — print the medians of both hooks and their difference in µs, and leave
# the difference in $added.
measure() {
  local label="$1" input="$2" i t
  local -a noop=() shim=()
  for ((i = 0; i < calls; i++)); do
    t="$(elapsed_us "$work/noop" "$input")" || { echo "Hook latency: the no-op hook failed" >&2; exit 1; }
    noop+=("$t")
    t="$(elapsed_us "$ROOT/shims/reference-transaction" "$input")" \
      || { echo "Hook latency: the shim refused the $label update" >&2; exit 1; }
    shim+=("$t")
  done
  local n s
  n="$(median_us "${noop[@]}")" s="$(median_us "${shim[@]}")"
  added=$((s - n))
  echo "Hook latency: $label — no-op hook ${n} µs, nuc ${s} µs, added ${added} µs (median of $calls calls)"
}

measure "a branch" "$zero $head refs/heads/feature"
gated="$added"
measure "main where it is (reported, not gated)" "$head $head refs/heads/main"

limit_us="$(awk -v ms="$limit_ms" 'BEGIN { printf "%d", ms * 1000 }')"
if ((gated > limit_us)); then
  echo "Hook latency FAILED: nuc adds ${gated} µs per call over a no-op hook, more than ${limit_ms} ms (Q-5)." >&2
  exit 1
fi
echo "Hook latency passed: nuc adds ${gated} µs per call, within ${limit_ms} ms (Q-5)."
