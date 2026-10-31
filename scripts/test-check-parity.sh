#!/usr/bin/env bash
#
# Self-test of scripts/check-parity.sh. Verifies: ADR-draft-parity-harness-with-variants
#
# The comparison cases feed the harness pairs of outputs through '--compare': pairs that differ in
# one finding, in one exit status, by one bash message the table does not know, or by a line nuc
# writes that is no finding fail; pairs that agree pass, a check not ported yet included. The run
# cases build a corpus from a fixture repository, fetched through file://, whose check-docs.sh is a
# stub, and run the harness with a stand-in for nuc: agreeing outputs pass, differing ones fail, a
# corpus whose cache cannot be filled fails, and a malformed corpus is refused. No network and no
# build.
#
# Inputs: scripts/check-parity.sh tests/parity/ scripts/lib/
#
# Usage:
#     scripts/test-check-parity.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
HARNESS="$ROOT/scripts/check-parity.sh"

MISSING="README.md: no '**Template release:**' line — the Template section names the release of the template this repository carries, or 'unreleased' (ADR-0005)"
MALFORMED="README.md: the Template release line names neither a 'vX.Y.Z' tag nor 'unreleased' — a release moves it to the tag it cuts (ADR-0005)"
UNPORTED="docs/x.md:3: a line of prose is 104 columns, over 100"

# bash_out <name> [messages...] — the output of check-docs.sh with the messages as its findings.
bash_out() {
  local f="$work/$1.bash"; shift
  if (($#)); then
    { echo "Documentation checks FAILED:"; echo; printf '  - %s\n' "$@"; echo; echo "$# problem(s) found."; } > "$f"
  else
    echo "Documentation checks passed." > "$f"
  fi
  echo "$f"
}

# nuc_out <name> [lines...] — the standard output of nuc with the lines.
nuc_out() {
  local f="$work/$1.nuc"; shift
  printf '%s\n' "$@" | sed '/^$/d' > "$f"
  echo "$f"
}

# expect <pass|fail> <name> <needle> <command...> — run it; for a failure, its output contains the
# needle.
expect() {
  local want="$1" name="$2" needle="$3" out rc
  shift 3
  out="$("$@" 2>&1)"; rc=$?
  if [[ ( "$want" == pass && $rc -eq 0 ) || ( "$want" == fail && $rc -ne 0 && "$out" == *"$needle"* ) ]]; then
    passed=$((passed + 1))
    return
  fi
  failed=$((failed + 1))
  echo "FAIL: $name (expected $want${needle:+ with \"$needle\"}, exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
}

# shellcheck disable=SC2317,SC2329 # run through expect
compare() { bash "$HARNESS" --compare "$@"; }

# --- cases: one pair ----------------------------------------------------------------------

expect pass "both pass" "" compare "$(bash_out pass)" 0 "$(nuc_out pass)" 0

expect pass "the same finding, bash without a line, nuc with one" "" \
  compare "$(bash_out same "$MALFORMED")" 1 "$(nuc_out same "README.md:5: docs/template-release: $MALFORMED")" 1

# The table of the repository and a row for a check not ported yet.
{ cat "$ROOT/tests/parity/normalize.tsv"; printf '^docs/x\\.md:[0-9]+: a line of prose\t-\t-\n'; } > "$work/unported.tsv"
expect pass "a check not ported yet is left out, its exit status with it" "" \
  env PARITY_TABLE="$work/unported.tsv" bash "$HARNESS" --compare "$(bash_out unported "$UNPORTED")" 1 "$(nuc_out unported)" 0

expect fail "a finding only bash reports" "only bash reports: docs/template-release README.md" \
  compare "$(bash_out only-bash "$MISSING")" 1 "$(nuc_out only-bash)" 0

expect fail "a finding only nuc reports" "only nuc reports" \
  compare "$(bash_out only-nuc)" 0 "$(nuc_out only-nuc "README.md: docs/template-release: no line")" 1

expect fail "the same findings, a different exit status" "nuc exited 0" \
  compare "$(bash_out status "$MISSING")" 1 "$(nuc_out status "README.md: docs/template-release: x")" 0

expect fail "nuc could not decide" "nuc could not decide (exit 2)" \
  compare "$(bash_out undecided)" 0 "$(nuc_out undecided)" 2

expect fail "a bash message the table does not know" "bash message the table does not know" \
  compare "$(bash_out unknown "README.md: something new")" 1 "$(nuc_out unknown)" 0

expect fail "bash failing without a finding" "bash exited 2 without a finding" \
  compare "$(bash_out crashed)" 2 "$(nuc_out crashed)" 0

expect fail "a line of nuc that is no finding" "nuc output not understood" \
  compare "$(bash_out stray)" 0 "$(nuc_out stray "warning: something")" 0

expect fail "a rule of nuc the table maps nothing to" "which the table maps no message to" \
  compare "$(bash_out unmapped)" 0 "$(nuc_out unmapped "README.md:1: docs/other: x")" 1

# --- cases: a run over a corpus -----------------------------------------------------------

# source_repo <name> <findings...> — a repository whose check-docs.sh prints the findings, printed
# as its path; its commit is in $work/<name>.commit.
source_repo() {
  local dir="$work/src-$1"; shift
  mkdir -p "$dir/scripts"
  git -C "$dir" init -q
  git -C "$dir" config uploadpack.allowAnySHA1InWant true
  bash_out "stub-$(basename "$dir")" "$@" > /dev/null
  printf '#!/usr/bin/env bash\ncat %q\n[[ -s %q ]] && grep -q "^  - " %q && exit 1\nexit 0\n' \
    "$work/stub-$(basename "$dir").bash" "$work/stub-$(basename "$dir").bash" "$work/stub-$(basename "$dir").bash" \
    > "$dir/scripts/check-docs.sh"
  echo "# Fixture" > "$dir/README.md"
  git -C "$dir" add -A
  git -C "$dir" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -q -m fixture
  git -C "$dir" rev-parse HEAD > "$work/$(basename "$dir").commit"
  echo "$dir"
}

# corpus <file> <name> <url> <commit> — a corpus with one repository.
corpus() {
  printf '# A fixture corpus.\n[[repository]]\nname = "%s"\nurl = "%s"\ncommit = "%s"\n' "$2" "$3" "$4" > "$1"
  echo "$1"
}

# stand_in <name> <lines...> — a nuc that prints the lines and exits 1 when there are any.
stand_in() {
  local f="$work/nuc-$1"; shift
  { echo '#!/usr/bin/env bash'; printf 'printf "%%s\\n" %q\n' "$@"; (($#)) && echo 'exit 1'; echo 'exit 0'; } > "$f"
  chmod +x "$f"
  echo "$f"
}

# shellcheck disable=SC2317,SC2329 # run through expect
run() { bash "$HARNESS" --corpus "$1" --cache "$work/cache" --nuc "$2"; }

src="$(source_repo agree "$MISSING")"
c="$(corpus "$work/agree.toml" agree "file://$src" "$(cat "$work/src-agree.commit")")"
expect pass "a corpus on which bash and nuc agree" "" \
  run "$c" "$(stand_in agree "README.md: docs/template-release: no line")"
expect pass "the filled cache is used again, the tree reset" "" \
  run "$c" "$(stand_in again "README.md: docs/template-release: no line")"

expect fail "a corpus on which they differ" "only bash reports" run "$c" "$(stand_in differ)"

c="$(corpus "$work/gone.toml" gone "file://$work/no-such-repository" "$(cat "$work/src-agree.commit")")"
expect fail "a cache that cannot be filled fails" "the cache could not be filled" run "$c" "$(stand_in gone)"

printf '[[repository]]\nname = "x"\nurl = "file:///x"\ncommit = "abc"\n' > "$work/short.toml"
expect fail "a commit that is no full SHA is refused" "a 40-hex commit" run "$work/short.toml" "$(stand_in short)"

printf '[[repository]]\nname = "x"\nbranch = "main"\n' > "$work/key.toml"
expect fail "a key the corpus does not have is refused" "a 40-hex commit" run "$work/key.toml" "$(stand_in key)"

summary "Parity harness self-test"
