#!/usr/bin/env bash
#
# Self-test of scripts/check-parity.sh. Verifies: ADR-draft-parity-harness-with-variants
#
# The comparison cases feed the harness pairs of outputs through '--compare': pairs that differ in
# one finding, in one exit status, by one bash message the table does not know, or by a line nuc
# writes that is no finding fail; pairs that agree pass, a check not ported yet included. The run
# cases build a corpus from a fixture repository, fetched through file://, whose check-docs.sh is a
# stub, and run the harness with a stand-in for nuc: agreeing outputs pass, differing ones fail, a
# corpus whose cache cannot be filled fails, and a malformed corpus is refused. The floor cases
# refuse a tree of a release before v0.5.0 or of none, and compare a later one. The variant cases
# use a stub that reports the missing release line as check 14 does: a variant that fits is
# compared, one that fits no tree, one on which bash and nuc differ, one that does not make its
# rule fire, and a malformed or unmapped row each fail. No network and no build.
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
  printf '# Fixture\n\n- **Template release:** %s\n' "${RELEASE:-v0.5.0}" > "$dir/README.md"
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
  { echo '#!/usr/bin/env bash'; (($#)) && printf 'printf "%%s\\n" %q\n' "$@"; (($#)) && echo 'exit 1'; echo 'exit 0'; } > "$f"
  chmod +x "$f"
  echo "$f"
}

: > "$work/no-variants.tsv"
# run <corpus> <nuc> [variants] — the harness on the corpus, with no variants unless given.
# shellcheck disable=SC2317,SC2329 # run through expect
run() { bash "$HARNESS" --corpus "$1" --cache "$work/cache" --nuc "$2" --variants "${3:-$work/no-variants.tsv}"; }

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

# --- cases: the release floor -------------------------------------------------------------

for release in v0.4.9 "unreleased"; do
  src="$(RELEASE="$release" source_repo "old-${release//[^a-z0-9]/}")"
  c="$(corpus "$work/old.toml" old "file://$src" "$(cat "$work/src-old-${release//[^a-z0-9]/}.commit")")"
  expect fail "a tree of the release '$release', before v0.5.0, is refused" "not v0.5.0 or later" \
    run "$c" "$(stand_in "old-${release//[^a-z0-9]/}")"
done

src="$(RELEASE=v0.6.0 source_repo later)"
c="$(corpus "$work/later.toml" later "file://$src" "$(cat "$work/src-later.commit")")"
expect pass "a tree of a later release is compared" "" run "$c" "$(stand_in later)"

# --- cases: variants ----------------------------------------------------------------------

# reactive_repo <name> — a repository whose check-docs.sh reports the missing release line when
# the README lacks it, as check 14 does, printed as its path.
reactive_repo() {
  local dir
  dir="$(source_repo "$1")"
  # shellcheck disable=SC2016 # the script is the stub's
  printf '%s\n' '#!/usr/bin/env bash' \
    'if grep -q "^- \*\*Template release:\*\* v" README.md; then echo "Documentation checks passed."; exit 0; fi' \
    "printf 'Documentation checks FAILED:\\n\\n  - %s\\n\\n1 problem(s) found.\\n' \"$MISSING\"" \
    'exit 1' > "$dir/scripts/check-docs.sh"
  git -C "$dir" add -A
  git -C "$dir" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -q -m reactive
  git -C "$dir" rev-parse HEAD > "$work/src-$1.commit"
  echo "$dir"
}

# reactive_nuc — a nuc that reports the missing release line of the tree given by --repo.
reactive_nuc() {
  local f="$work/nuc-reactive"
  # shellcheck disable=SC2016 # the script is the stand-in's
  printf '%s\n' '#!/usr/bin/env bash' \
    'grep -q "^- \*\*Template release:\*\* v" "$3/README.md" && exit 0' \
    'echo "README.md: docs/template-release: no line"; exit 1' > "$f"
  chmod +x "$f"
  echo "$f"
}

# variants <file> <rows...> — a variants file with the rows, printed as its path.
variants() { local f="$1"; shift; printf '%s\n' "$@" > "$f"; echo "$f"; }

src="$(reactive_repo react)"
c="$(corpus "$work/react.toml" react "file://$src" "$(cat "$work/src-react.commit")")"
DELETE=$'deleted\tdocs/template-release\tREADME.md\tsed\t/Template release:/d'
v="$(variants "$work/v-applies.tsv" "# A comment." "$DELETE")"
expect pass "a variant that fits is compared, and both report its finding" "" run "$c" "$(reactive_nuc)" "$v"
check "the run names the variant it compared" \
  bash -c "bash \"$HARNESS\" --corpus \"$c\" --cache \"$work/cache\" --nuc \"$(reactive_nuc)\" --variants \"$v\" | grep -q 'variant deleted — 1 finding'"

v="$(variants "$work/v-nowhere.tsv" $'nowhere\tdocs/template-release\tREADME.md\tsed\t/No such line/d')"
expect fail "a variant that fits no tree fails" "variant nowhere applies to no tree" run "$c" "$(reactive_nuc)" "$v"

v="$(variants "$work/v-missing-file.tsv" $'gone\tdocs/template-release\tNO-SUCH.md\tdelete\t-')"
expect fail "a variant of a file no tree has fails" "variant gone applies to no tree" run "$c" "$(reactive_nuc)" "$v"

v="$(variants "$work/v-differ.tsv" "$DELETE")"
expect fail "a variant on which bash and nuc differ fails" "variant deleted — 2 difference" run "$c" "$(stand_in silent)" "$v"

v="$(variants "$work/v-silent.tsv" $'silent\tdocs/template-release\tREADME.md\tsed\ts/Fixture/Changed/')"
expect fail "a variant that does not make its rule fire fails" "does not make its rule fire" run "$c" "$(reactive_nuc)" "$v"

v="$(variants "$work/v-malformed.tsv" $'bad name\tdocs/template-release\tREADME.md\tsed')"
expect fail "a malformed variant row is refused" "is not 'name<TAB>rule" run "$c" "$(reactive_nuc)" "$v"

v="$(variants "$work/v-unmapped.tsv" $'odd\tdocs/no-such-rule\tREADME.md\tdelete\t-')"
expect fail "a variant of a rule the table maps nothing to is refused" "maps no message to" run "$c" "$(reactive_nuc)" "$v"

summary "Parity harness self-test"
