#!/usr/bin/env bash
#
# Self-test of scripts/check-macos-floor.sh.
#
# Each case builds a fixture under a temporary directory — the floor, the self-test library, a
# script without a library and one behind the floor, and a workflow with a macOS job that runs the
# first and a Linux job that runs the second — runs the check against it, and asserts the exit code
# and, for a failing case, the key and the data of the finding. The cases stand in
# scripts/fixtures/check-macos-floor.txt, which says its format, so that a port of the check is
# held to them. The table is read twice, the second time against a copy of the check that rewords
# its messages.
#
# Inputs: scripts/check-macos-floor.sh scripts/lib/ scripts/fixtures/check-macos-floor.txt .github/workflows/ scripts/check-build.sh scripts/test-release.sh
#
# Usage:
#     scripts/test-check-macos-floor.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
CHECK="$ROOT/scripts/check-macos-floor.sh"
TABLE="$ROOT/scripts/fixtures/check-macos-floor.txt"

# repo <name> — a fixture that passes the check, printed as its path.
repo() {
  local dir="$work/$1"
  mkdir -p "$dir/scripts/lib" "$dir/.github/workflows"
  echo '# the floor' > "$dir/scripts/lib/bash.sh"
  # shellcheck disable=SC2016 # the line is the library's, not this shell's
  printf '%s\n' 'source "$(dirname "${BASH_SOURCE[0]}")/bash.sh"' > "$dir/scripts/lib/selftest.sh"
  printf '#!/usr/bin/env bash\necho plain\n' > "$dir/scripts/plain.sh"
  # shellcheck disable=SC2016 # the line is the script's, not this shell's
  printf '%s\n' '#!/usr/bin/env bash' 'source "$(dirname "${BASH_SOURCE[0]}")/lib/bash.sh"' \
    > "$dir/scripts/floored.sh"
  printf '%s\n' 'jobs:' '  mac:' '    runs-on: macos-latest' '    steps:' \
    '      - run: bash scripts/plain.sh' '  linux:' '    runs-on: ubuntu-latest' '    steps:' \
    '      - run: bash scripts/floored.sh' > "$dir/.github/workflows/ci.yml"
  echo "$dir"
}

# table_cases <prefix> — run each case of the table against $CHECK.
table_cases() {
  local n=0 line want needle name tree s op arg d path text out
  while IFS= read -r line; do
    [[ "$line" =~ ^(pass|fail) ]] || continue
    n=$((n + 1))
    IFS=$'\x1f' read -r want needle name tree <<< "${line// | /$'\x1f'}"
    d="$(repo "$1-$n")"
    [[ "$tree" != - ]] || tree=""
    while IFS= read -r s; do
      [[ -n "$s" ]] || continue
      read -r op arg <<< "$s"
      case "$op" in
        rm) rm -f "${d:?}/$arg" ;;
        copy) mkdir -p "$(dirname "$d/$arg")" && cp "$ROOT/$arg" "$d/$arg" ;;
        wf | write)
          if [[ "$op" == wf ]]; then
            path=.github/workflows/ci.yml text="${s#wf }"
          else
            path="${arg%% *}" text="${s#write "${arg%% *}" }"
          fi
          mkdir -p "$(dirname "$d/$path")" && printf '%s' "${text//\\n/$'\n'}" > "$d/$path" ;;
        *) echo "fixture table: unknown step '$op'" >&2; exit 1 ;;
      esac
    done <<< "${tree//; /$'\n'}"
    out="$(bash "$CHECK" --root "$d" 2>&1)"
    table_case "$name" "$want" "$needle" "$?" "$out"
  done < "$TABLE"
  ((n > 0)) || { echo "FAIL: no case read from $TABLE" >&2; failed=$((failed + 1)); }
}
table_cases table

# --- the key decides, not the wording ------------------------------------------------------

copy="$work/reworded/scripts"
mkdir -p "$copy"
cp -R "$ROOT/scripts/lib" "$copy/"
sed -E '/\[macos\//s/ (a|is|no|not|the|of|on) / reworded /g' "$CHECK" > "$copy/check-macos-floor.sh"
keyed="$(grep -c '\[macos/' "$CHECK")"
check "a copy of the check rewords each message with a key" \
  test "$keyed" -gt 0 -a "$keyed" -eq "$(grep -c ' reworded .*\[macos/' "$copy/check-macos-floor.sh")"
CHECK="$copy/check-macos-floor.sh"
table_cases reworded
CHECK="$ROOT/scripts/check-macos-floor.sh"

# --- arguments -----------------------------------------------------------------------------

# refused <argument...> — the check stops with exit 2 and names the first argument.
# shellcheck disable=SC2317,SC2329 # run by check
refused() {
  local out rc
  out="$(bash "$CHECK" "$@" 2>&1)"; rc=$?
  ((rc == 2)) && [[ "$out" == *"unknown argument '$1'"* ]]
}
check "a root given as a first argument is refused" refused "$work"
check "an argument the check does not read is refused" refused --verbose

summary "macOS floor self-test"
