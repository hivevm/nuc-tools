#!/usr/bin/env bash
#
# Gates check for this repository (ADR-0018).
#
# This repository builds the tool that derived projects run as their gates; its own gates never
# run the 'nuc' built from the change they judge. This check reads the gates and what they run —
# scripts/check-all.sh, every hook under .githooks/, .github/workflows/checks.yml, and every
# script under scripts/ — and fails on each line that would run a build of the working tree:
#
#   1. 'cargo run', or its alias 'cargo r', which builds and runs the working tree.
#   2. 'cargo install' with '--path', which installs it.
#   3. A path under 'target/', or a 'debug/nuc' or 'release/nuc' under another target directory,
#      where cargo puts what it builds.
#   4. 'nuc' run as a command: at the start of a line or a 'run:' value; after ';', '&', '|',
#      '(', '$(', or a backquote; or after 'env' with its options and assignments, 'exec',
#      'command', 'time', or 'xargs'. A released 'nuc' pinned as a dependency is a later decision
#      of its own, and this check changes with it.
#
# Four scripts are not read. scripts/check-build.sh and scripts/check-parity.sh are the tests of
# the build and run it; they judge nothing but the build. This check and its self-test write the
# patterns above into fixtures and run none of them. Full-line comments are dropped, so a gate may
# explain itself; a trailing comment is read as content. The files are read line by line, so a
# command assembled from variables is not seen; that is review.
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

NOT_READ=(scripts/check-build.sh scripts/check-parity.sh scripts/check-gates.sh scripts/test-check-gates.sh)

gates=()
[[ -f "$ROOT/scripts/check-all.sh" ]] && gates+=("$ROOT/scripts/check-all.sh")
shopt -s nullglob
for hook in "$ROOT/.githooks"/*; do [[ -f "$hook" ]] && gates+=("$hook"); done
shopt -u nullglob
[[ -f "$ROOT/.github/workflows/checks.yml" ]] && gates+=("$ROOT/.github/workflows/checks.yml")
if [[ -d "$ROOT/scripts" ]]; then
  while IFS= read -r -d '' script; do
    rel="${script#"$ROOT"/}"
    [[ "$rel" == scripts/check-all.sh ]] || _contains "$rel" "${NOT_READ[@]}" || gates+=("$script")
  done < <(find "$ROOT/scripts" -type f -name '*.sh' -print0 | sort -z)
fi

CARGO='(^|[^A-Za-z0-9_-])cargo([[:space:]]+[+-][^[:space:]]*)*[[:space:]]+'
CARGO_RUN="${CARGO}(run|r)([[:space:]]|\$)"
CARGO_INSTALL="${CARGO}install([[:space:]].*)?[[:space:]]--path([[:space:]=]|\$)"
TARGET='(^|[^A-Za-z0-9_.-])(\./)?target/|(^|[/"])(debug|release)/nuc(\.exe)?([^A-Za-z0-9_.-]|$)'
LAUNCHER='(env([[:space:]]+(-[^[:space:]]*|[A-Za-z_][A-Za-z0-9_]*=[^[:space:]]*))*|exec|command|time|xargs)'
NUC="(^|run:|[;&|(\`]|(^|[[:space:]])${LAUNCHER})[[:space:]]*(\\\$\\([[:space:]]*)?([^[:space:]]*/)?nuc(\\.exe)?([[:space:]]|\$)"

for gate in "${gates[@]}"; do
  rel="${gate#"$ROOT"/}"
  while IFS= read -r hit; do
    n="${hit%%:*}" text="${hit#*:}"
    [[ "$text" =~ $CARGO_RUN ]] && add_error "$rel:$n: 'cargo run' runs the working tree's build"
    [[ "$text" =~ $CARGO_INSTALL ]] && add_error "$rel:$n: 'cargo install --path' installs the working tree's build"
    [[ "$text" =~ $TARGET ]] && add_error "$rel:$n: a path under a target directory is the working tree's build"
    [[ "$text" =~ $NUC ]] && add_error "$rel:$n: 'nuc' run by a gate, which runs no unreleased build"
  done < <(grep -nvE '^[[:space:]]*#' "$gate")
done

if ((${#errors[@]})); then
  echo "Gates check: ${#errors[@]} finding(s) — the gates run no build of the working tree:" >&2
  printf '  - %s\n' "${errors[@]}" >&2
  exit 1
fi
echo "Gates check passed (${#gates[@]} gates, none runs a build of the working tree)."
