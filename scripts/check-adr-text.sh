#!/usr/bin/env bash
#
# Accepted ADR text check.
#
# An accepted ADR changes only in its Status and Date lines outside a normalization: the flip to
# accepted or superseded changes the header line only, and a change of an ADR's text is a
# normalization or a superseding ADR (NUC-decisions-are-proposed-then-accepted, clauses 4 and 5).
# The check reads each commit of the change, base to head, and fails where a commit changes a line
# of a project ADR other than those two lines, or deletes it, where the ADR was accepted at the
# commit's parent or the commit flips it to accepted: a flip changes the header line alone.
#
# Not held:
# - a commit whose message carries a line 'ADR map:', which the normalize skill writes into the
#   commit of a normalization;
# - the NUC- ADRs, which a take-up of a template release replaces and docs/adr/NUC.blobs holds;
# - an ADR proposed at the commit's parent that the commit leaves proposed or rejects;
# - the working tree: only commits are read.
#
# Pure bash + git. Usage:
#     scripts/check-adr-text.sh [--root <dir>] [base [head]]
# Without a base, the merge base of head with origin/main (else main) is used; CI passes the pull
# request's base. Without a main, there is nothing to compare, and the check passes.
# Exit code 0 when the check passes, 1 otherwise, and 2 on an argument it does not read or a base
# or head that is no commit.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/bash.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/bash.sh"

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/check.sh"

# shellcheck source-path=SCRIPTDIR source=lib/git.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/git.sh"

# shellcheck source-path=SCRIPTDIR source=lib/root.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/root.sh"
usage="scripts/check-adr-text.sh [--root <dir>] [base [head]]"
enter_root "${BASH_SOURCE[0]}" "$usage" "$@"
rest_args "${BASH_SOURCE[0]}" "$usage" 2 "${ARGS[@]}"
set -- "${ARGS[@]}"

if ! resolve_change "Accepted ADR text check" "${1:-}" "${2:-}"; then
  echo "Accepted ADR text check passed (nothing to compare) [adr-text/check:passed]"
  exit 0
fi
base="$CHANGE_BASE" head="$CHANGE_HEAD"

# The lines of the header a flip changes: the status and the date of its last change.
FLIP_RE='^- \*\*(Status|Date):\*\*'

# accepted <rev> <path> — whether the ADR at the revision says it is accepted.
accepted() {
  local status
  status="$(git show "$1:$2" 2>/dev/null | grep -m1 -E '^- \*\*Status:\*\*')"
  [[ "$status" == *accepted* ]]
}

# held <path> — whether the path is a project ADR the check holds.
held() {
  [[ "$1" == docs/adr/*.md && "$1" != docs/adr/*/* ]] || return 1
  case "${1#docs/adr/}" in
    README.md | template.md | NUC-*) return 1 ;;
  esac
  return 0
}

# hunks_beyond_flip <commit> <path> — the line in the commit's file, or in the parent's where the
# commit adds none, of each hunk of the commit's change to the file that changes a line other than
# the Status and Date lines.
hunks_beyond_flip() {
  local line at="" beyond=0
  while IFS= read -r line; do
    if [[ "$line" =~ ^@@\ -([0-9]+)(,[0-9]+)?\ \+([0-9]+)(,([0-9]+))? ]]; then
      ((beyond)) && echo "$at"
      at="${BASH_REMATCH[3]}" beyond=0
      [[ "${BASH_REMATCH[5]:-1}" == 0 ]] && at="${BASH_REMATCH[1]}"
    elif [[ "$line" == [-+]* && "$line" != ---* && "$line" != +++* ]]; then
      [[ "${line:1}" =~ $FLIP_RE ]] || beyond=1
    fi
  done < <(git diff -U0 --no-color --no-ext-diff "$1^" "$1" -- "$2")
  ((beyond)) && echo "$at"
  return 0
}

commits=0 held_changes=0
mapfile -t revs < <(git rev-list --reverse --no-merges "$base..$head")
for rev in "${revs[@]}"; do
  commits=$((commits + 1))
  grep -qx 'ADR map:' <<< "$(git log -1 --format=%B "$rev")" && continue
  short="$(git rev-parse --short "$rev")"
  while IFS=$'\t' read -r status path; do
    held "$path" || continue
    accepted "$rev^" "$path" || accepted "$rev" "$path" || continue
    held_changes=$((held_changes + 1))
    if [[ "$status" == D ]]; then
      add_error "$short: $path: an accepted ADR deleted outside a normalization [adr-text/accepted:deleted]"
      continue
    fi
    while read -r n; do
      add_error "$short: $path:$n: a line of an accepted ADR changed outside its Status and Date — supersede it, or leave it to a normalization [adr-text/accepted:changed]"
    done < <(hunks_beyond_flip "$rev" "$path")
  done < <(git diff-tree --no-commit-id --no-renames -r --name-status "$rev")
done

if ((${#errors[@]})); then
  echo "Accepted ADR text check FAILED:" >&2
  printf '  - %s\n' "${errors[@]}" >&2
  exit 1
fi
echo "Accepted ADR text check passed ($commits commits, $held_changes changes of accepted ADRs, each a flip) [adr-text/check:passed]"
exit 0
