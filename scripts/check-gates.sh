#!/usr/bin/env bash
#
# Gates check for this repository (ADR-draft-gates-never-run-the-build).
#
# This repository builds the tool that derived projects run as their gates; its own gates never
# run the 'nuc' built from the change they judge. This check reads the gates —
# scripts/check-all.sh, every hook under .githooks/, and .github/workflows/checks.yml — and fails
# on each line that would run a build of the working tree:
#
#   1. 'cargo run', which builds and runs the working tree.
#   2. A path under 'target/', where cargo puts what it builds.
#   3. 'nuc' run as a command: at the start of a line or a 'run:' value, or after ';', '&', '|',
#      '(', '$(', or a backquote. A released 'nuc' pinned as a dependency is a later decision of
#      its own, and this check changes with it.
#
# scripts/check-build.sh runs the build's own tests and is not read: the gates name it, and it
# judges nothing but the build. Full-line comments are dropped, so a gate may explain itself; a
# trailing comment is read as content. The files are read line by line, so a command assembled
# from variables is not seen; that is review.
#
# Pure bash + coreutils/grep. Usage:
#     scripts/check-gates.sh [repository root]
# The root defaults to this repository; scripts/test-check-gates.sh passes fixtures.
# Exit code 0 when the check passes, 1 otherwise, 2 when the root does not exist.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/check.sh"

ROOT="$(cd "${1:-$(dirname "${BASH_SOURCE[0]}")/..}" 2>/dev/null && pwd)" \
  || { echo "Gates check: '${1:-}' is not a directory." >&2; exit 2; }

gates=()
[[ -f "$ROOT/scripts/check-all.sh" ]] && gates+=("$ROOT/scripts/check-all.sh")
shopt -s nullglob
for hook in "$ROOT/.githooks"/*; do [[ -f "$hook" ]] && gates+=("$hook"); done
shopt -u nullglob
[[ -f "$ROOT/.github/workflows/checks.yml" ]] && gates+=("$ROOT/.github/workflows/checks.yml")

CARGO_RUN='(^|[^A-Za-z0-9_-])cargo([[:space:]]+[+-][^[:space:]]*)*[[:space:]]+run([[:space:]]|$)'
TARGET='(^|[^A-Za-z0-9_.-])(\./)?target/'
NUC='(^|run:|[;&|(`])[[:space:]]*(\$\([[:space:]]*)?([^[:space:]]*/)?nuc(\.exe)?([[:space:]]|$)'

for gate in "${gates[@]}"; do
  rel="${gate#"$ROOT"/}"
  while IFS= read -r hit; do
    n="${hit%%:*}" text="${hit#*:}"
    [[ "$text" =~ $CARGO_RUN ]] && add_error "$rel:$n: 'cargo run' runs the working tree's build"
    [[ "$text" =~ $TARGET ]] && add_error "$rel:$n: a path under 'target/' is the working tree's build"
    [[ "$text" =~ $NUC ]] && add_error "$rel:$n: 'nuc' run by a gate, which runs no unreleased build"
  done < <(grep -nvE '^[[:space:]]*#' "$gate")
done

if ((${#errors[@]})); then
  echo "Gates check: ${#errors[@]} finding(s) — the gates run no build of the working tree:" >&2
  printf '  - %s\n' "${errors[@]}" >&2
  exit 1
fi
echo "Gates check passed (${#gates[@]} gates, none runs a build of the working tree)."
