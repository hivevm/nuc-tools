#!/usr/bin/env bash
#
# Self-test of scripts/check-adr-text.sh. Verifies: NUC-decisions-are-proposed-then-accepted
#
# Each case builds a git repository under a temporary directory: main with a proposed, an accepted,
# and a NUC- ADR, and a branch with the commits of the case. It runs the check with the branch's
# base and asserts the exit code and, for a failing case, the commit, the ADR, the line, and the
# key of the finding.
#
# Inputs: scripts/check-adr-text.sh scripts/lib/
#
# Usage:
#     scripts/test-check-adr-text.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
CHECK="$ROOT/scripts/check-adr-text.sh"

# adr <status> — an ADR with that status, its body two lines of prose.
adr() {
  printf '# X-a: A decision\n\n- **Status:** %s\n- **Date:** 2026-01-01\n\n## Decision\n\nWe will do it.\nAnd so on.\n' "$1"
}

# repo <name> — a fixture on a branch 'work' off main, printed as its path.
repo() {
  local dir="$work/$1"
  mkdir -p "$dir/docs/adr"
  git init -q -b main "$dir"
  adr '🟡 proposed' > "$dir/docs/adr/API-proposed.md"
  adr '🟢 accepted' > "$dir/docs/adr/API-accepted.md"
  adr '🟢 accepted' > "$dir/docs/adr/NUC-base.md"
  g "$dir" add -A
  g "$dir" commit -q -m "Initial commit"
  g "$dir" switch -q -c work
  echo "$dir"
}

# g <dir> <git args...> — git in the fixture with an identity.
g() {
  local dir="$1"; shift
  fixture "$dir"
  git -C "$dir" -c user.name=t -c user.email=t@example.invalid "$@"
}

# edit <file> <sed script> — the file rewritten by the script, without sed's in-place flag, which
# BSD sed reads otherwise.
edit() { sed "$2" "$1" > "$1.new" && mv "$1.new" "$1"; }

# commit <dir> <message> — stage everything and commit it.
commit() { g "$1" add -A; g "$1" commit -q -m "$2"; }

# run_check <dir> — the check on the fixture, against main; sets out and code.
run_check() {
  out="$(bash "$CHECK" --root "$1" main 2>&1)"
  code=$?
}

# expect <name> <code> [needle...] — count one case: the exit code, and each needle in the output.
expect() {
  local name="$1" want="$2" needle ok=1
  shift 2
  [[ "$code" == "$want" ]] || ok=0
  for needle in "$@"; do output_holds "$out" "$needle" || ok=0; done
  check "$name" test "$ok" -eq 1
  ((ok)) || printf '    exit %s, output:\n%s\n' "$code" "$out" >&2
}

d="$(repo body)"
edit "$d/docs/adr/API-accepted.md" 's/We will do it./We will do it otherwise./'
commit "$d" "Reword an accepted ADR"
run_check "$d"
expect "a line of an accepted ADR changed" 1 \
  "$(g "$d" rev-parse --short HEAD): docs/adr/API-accepted.md:8:" "[adr-text/accepted:changed]"

d="$(repo flip)"
edit "$d/docs/adr/API-proposed.md" 's/🟡 proposed/🟢 accepted/; s/2026-01-01/2026-02-02/'
commit "$d" "Accept an ADR"
run_check "$d"
expect "a flip of the Status and Date lines alone" 0 "[adr-text/check:passed]"

d="$(repo flip-and-body)"
edit "$d/docs/adr/API-proposed.md" 's/🟡 proposed/🟢 accepted/; s/And so on./And so forth./'
commit "$d" "Accept an ADR and reword it"
run_check "$d"
expect "a flip to accepted that rewords the ADR" 1 "docs/adr/API-proposed.md:9:" \
  "[adr-text/accepted:changed]"

d="$(repo supersede)"
edit "$d/docs/adr/API-accepted.md" 's/🟢 accepted/⚪ superseded by API-other/; s/2026-01-01/2026-03-03/'
commit "$d" "Supersede an ADR"
run_check "$d"
expect "a flip of an accepted ADR to superseded" 0 "[adr-text/check:passed]"

d="$(repo normalization)"
edit "$d/docs/adr/API-accepted.md" 's/We will do it./We will do it, restated./'
g "$d" add -A
g "$d" commit -q -m "Restate the record" -m "ADR map:" -m "API-accepted API-accepted"
run_check "$d"
expect "a normalization's commit" 0 "[adr-text/check:passed]"

d="$(repo nuc)"
edit "$d/docs/adr/NUC-base.md" 's/We will do it./We will do it as the template says./'
commit "$d" "Take up a template release"
run_check "$d"
expect "a NUC- ADR" 0 "[adr-text/check:passed]"

d="$(repo proposed)"
edit "$d/docs/adr/API-proposed.md" 's/We will do it./We will do it better./'
commit "$d" "Revise a proposed ADR"
run_check "$d"
expect "a proposed ADR left proposed" 0 "[adr-text/check:passed]"

d="$(repo deleted)"
rm "$d/docs/adr/API-accepted.md"
commit "$d" "Delete an accepted ADR"
run_check "$d"
expect "an accepted ADR deleted" 1 "docs/adr/API-accepted.md: an accepted ADR deleted" \
  "[adr-text/accepted:deleted]"

d="$(repo later)"
edit "$d/docs/adr/API-accepted.md" 's/And so on./And so forth./'
commit "$d" "Reword an accepted ADR"
edit "$d/docs/adr/API-accepted.md" 's/And so forth./And so on./'
commit "$d" "Take the rewording back"
run_check "$d"
expect "each commit read alone, a rewording taken back included" 1 \
  "$(g "$d" rev-parse --short HEAD~1): docs/adr/API-accepted.md:9:" \
  "$(g "$d" rev-parse --short HEAD): docs/adr/API-accepted.md:9:"

d="$(repo rule)"
edit "$d/docs/adr/API-accepted.md" 's/^And so on.$/---/'
commit "$d" "Turn a line of an accepted ADR into a rule"
run_check "$d"
expect "a line that reads as a diff's header, inside a hunk" 1 "docs/adr/API-accepted.md:9:"

d="$(repo unicode)"
edit "$d/docs/adr/API-accepted.md" 's/We will do it./We will do it otherwise./'
mv "$d/docs/adr/API-accepted.md" "$d/docs/adr/API-ä b.md"
commit "$d" "Rename an accepted ADR beyond ASCII and reword it"
run_check "$d"
expect "a path beyond ASCII" 1 "docs/adr/API-ä b.md:1:" "[adr-text/accepted:deleted]"

d="$work/no-main"
mkdir -p "$d"
git init -q -b other "$d"
g "$d" commit -q --allow-empty -m "Initial commit"
out="$(bash "$CHECK" --root "$d" 2>&1)"
code=$?
expect "no main to compare with" 0 "nothing to compare"

out="$(bash "$CHECK" --root "$work/body" main HEAD extra 2>&1)"
code=$?
expect "a third argument" 2 "unknown argument"

summary "Accepted ADR text check self-test"
