#!/usr/bin/env bash
#
# Self-test of scripts/check-compose.sh.
#
# Each case builds a fixture under a temporary directory — a .devcontainer/compose.yaml whose one
# service publishes one port on a variable with a loopback default — runs the check against it,
# and asserts the exit code and, for a failing case, the key and the data of the finding. The
# cases stand in scripts/fixtures/check-compose.txt, which says its format, so that a port of the
# check is held to them: the baseline and this repository's own stack pass, and each form of a port
# that can land on every interface of the host fails. The table is read twice, the second time
# against a copy of the check that rewords its messages.
#
# Inputs: scripts/check-compose.sh scripts/lib/ scripts/fixtures/check-compose.txt
#         .devcontainer/compose.yaml
#
# Usage:
#     scripts/test-check-compose.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
CHECK="$ROOT/scripts/check-compose.sh"
TABLE="$ROOT/scripts/fixtures/check-compose.txt"

# repo <name> — a fixture that passes the check, printed as its path.
repo() {
  local dir="$work/$1"
  mkdir -p "$dir/.devcontainer"
  # shellcheck disable=SC2016 # the variable is Compose's, not the shell's
  printf '%s\n' 'services:' '  collector:' '    image: example' '    ports:' \
    '      - "${OBSERVABILITY_BIND:-127.0.0.1}:4318:4318" # OTLP/HTTP' \
    > "$dir/.devcontainer/compose.yaml"
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
        rm) rm -r "${d:?}/$arg" ;;
        port) printf '      - %s\n' "$arg" >> "$d/.devcontainer/compose.yaml" ;;
        line) printf '        %s\n' "$arg" >> "$d/.devcontainer/compose.yaml" ;;
        copy) cp "$ROOT/$arg" "$d/$arg" ;;
        write)
          path="${arg%% *}" text="${s#write "${arg%% *}" }"
          text="${text//\\n/$'\n'}"
          mkdir -p "$(dirname "$d/$path")" && printf '%s' "${text//\\r/$'\r'}" > "$d/$path" ;;
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
sed -E '/\[compose\//s/ (a|is|no|not|the|on) / reworded /g' "$CHECK" > "$copy/check-compose.sh"
keyed="$(grep -c '\[compose/' "$CHECK")"
check "a copy of the check rewords each message with a key" \
  test "$keyed" -gt 0 -a "$keyed" -eq "$(grep -c ' reworded .*\[compose/' "$copy/check-compose.sh")"
CHECK="$copy/check-compose.sh"
table_cases reworded
CHECK="$ROOT/scripts/check-compose.sh"

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

summary "Compose self-test"
