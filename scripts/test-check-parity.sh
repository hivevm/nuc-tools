#!/usr/bin/env bash
#
# Self-test of scripts/check-parity.sh. Verifies: ADR-0022 Q-2
#
# The comparison cases feed the harness pairs of outputs through '--compare': pairs that differ in
# one finding, in one exit status, by one bash message the table does not know, or by a line nuc
# writes that is no finding fail; pairs that agree pass, a check not ported yet included.
#
# The run cases build a corpus from fixture repositories, fetched through file://, whose
# check-docs.sh is a stub that reports the missing release line as check 14 does, beside a
# check-traceability.sh that passes, and run the harness with a stand-in for nuc. Agreeing trees
# and variants pass and leave the cache as pinned; differing ones, a cache that cannot be filled,
# and a malformed corpus fail, and so do a tree whose check-traceability.sh fails without a
# finding while its check-docs.sh passes and a tree without check-traceability.sh. A comparison
# case reads the outputs of both scripts as one. The floor cases
# refuse a tree whose release line names a release before v0.5.0, a pre-release, 'unreleased',
# or none, read as check 14 reads it, and compare a later one. The variant cases fail a variant
# that does not apply to one tree, one that does not make its rule fire beyond the tree's own
# findings, a ported rule without one, a path outside the tree, a sed expression that runs a
# command or writes a file, a name used twice, and a malformed or unmapped row. No network and no
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
# The texts nuc words as the bash does: the message after its location.
MISSING_TEXT="${MISSING#README.md: }" MALFORMED_TEXT="${MALFORMED#README.md: }"

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
  compare "$(bash_out same "$MALFORMED")" 1 "$(nuc_out same "README.md:5: docs/template-release: $MALFORMED_TEXT")" 1

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

# A table whose rows place a bash location at the file nuc names, by path and by prefix.
printf '%s\n' $'^ADR index: lists \tdocs/x\t-\t=docs/adr/README.md' $'^[^ :]+:[0-9]+: a field\tdocs/y\t-\t+docs/adr/' \
  > "$work/places.tsv"
# shellcheck disable=SC2317,SC2329 # run through expect
placed() { env PARITY_TABLE="$work/places.tsv" bash "$HARNESS" --compare "$@"; }
expect pass "a place '=' names the file nuc names" "" \
  placed "$(bash_out place-eq "ADR index: lists 'a.md' twice")" 1 "$(nuc_out place-eq "docs/adr/README.md:4: docs/x: ADR index: lists 'a.md' twice")" 1
expect pass "a place '+' prefixes the bash location, its line left out under '-'" "" \
  placed "$(bash_out place-plus "0001-a.md:3: a field")" 1 "$(nuc_out place-plus "docs/adr/0001-a.md:3: docs/y: a field")" 1
expect fail "a place that does not name nuc's file is a difference" "only bash reports: docs/x docs/adr/README.md" \
  placed "$(bash_out place-other "ADR index: lists 'a.md' twice")" 1 "$(nuc_out place-other "README.md: docs/x: lists")" 1
expect fail "the same file and rule with another text is a difference" "only nuc reports: docs/x docs/adr/README.md" \
  placed "$(bash_out place-text "ADR index: lists 'a.md' twice")" 1 \
  "$(nuc_out place-text "docs/adr/README.md: docs/x: ADR index: lists 'b.md' twice")" 1
printf '%s\n' $'^x\tdocs/x\t-\tdocs/adr/' > "$work/bad-place.tsv"
expect fail "a place that is neither '=' nor '+' is refused" "is not 'pattern<TAB>rule ID" \
  env PARITY_TABLE="$work/bad-place.tsv" bash "$HARNESS" --compare "$(bash_out bad-place)" 0 "$(nuc_out bad-place)" 0
printf '%s\n' $'^y\tdocs/y\t-\t@adrx' > "$work/bad-adr.tsv"
expect fail "a place that only begins as '@adr' is refused" "is not 'pattern<TAB>rule ID" \
  env PARITY_TABLE="$work/bad-adr.tsv" bash "$HARNESS" --compare "$(bash_out bad-adr)" 0 "$(nuc_out bad-adr)" 0

# The two scripts' outputs read as one: a finding of check-traceability.sh beside a passing
# check-docs.sh is understood and compared.
UNCITED="ADR-0001 is accepted but no test cites it — add a 'Verifies' marker"
UNCITED_ID="${UNCITED%% *}"
UNCITED_KEY="${UNCITED_ID#ADR-}"
printf '%s\n' "Documentation checks passed." "Traceability check FAILED:" "  - $UNCITED" > "$work/joined.bash"
expect pass "the outputs of both scripts are understood" "" \
  compare "$work/joined.bash" 1 "$(nuc_out joined "docs/adr/${UNCITED_KEY}-a.md: traceability/coverage: $UNCITED")" 1

# A place '@adr' names the file of the ADR the message opens with, compared by its key. The
# identifiers are taken from the message and assembled, so that this file cites no ADR of its own.
expect fail "an ADR's finding at the file of another ADR is a difference" "only bash reports: traceability/coverage docs/adr/$UNCITED_KEY" \
  compare "$work/joined.bash" 1 "$(nuc_out adr-other "docs/adr/9999-b.md: traceability/coverage: $UNCITED")" 1
expect fail "an ADR's finding at the directory is a difference" "only nuc reports: traceability/coverage docs/adr" \
  compare "$work/joined.bash" 1 "$(nuc_out adr-dir "docs/adr: traceability/coverage: $UNCITED")" 1
DRAFT="$(printf '%s-%s' ADR draft-a-b)"
printf '%s\n' "Traceability check FAILED:" "  - ${UNCITED/"$UNCITED_ID"/$DRAFT}" > "$work/draft.bash"
expect pass "a draft's finding names the draft's file" "" \
  compare "$work/draft.bash" 1 "$(nuc_out adr-draft "docs/adr/draft-a-b.md: traceability/coverage: ${UNCITED/"$UNCITED_ID"/$DRAFT}")" 1
# A slug the citations cannot name, with capitals or '_', is compared whole.
ODD="$(printf '%s-%s' ADR draft-Odd_b)"
printf '%s\n' "Traceability check FAILED:" "  - ${UNCITED/"$UNCITED_ID"/$ODD}" > "$work/odd.bash"
expect pass "a draft's slug with capitals and '_' names its file" "" \
  compare "$work/odd.bash" 1 "$(nuc_out adr-odd "docs/adr/draft-Odd_b.md: traceability/coverage: ${UNCITED/"$UNCITED_ID"/$ODD}")" 1
expect fail "a draft's finding at a shorter slug is a difference" "only bash reports: traceability/coverage docs/adr/draft-Odd_b" \
  compare "$work/odd.bash" 1 "$(nuc_out adr-short "docs/adr/draft-Odd.md: traceability/coverage: ${UNCITED/"$UNCITED_ID"/$ODD}")" 1
# Another finding of the rule, whose text the row does not match, keeps its whole file.
EXEMPT="docs/adr/${UNCITED_KEY}-a.md: 'Not mechanically decidable' declared without a reason — the paragraph has to say why no test can decide it"
printf '%s\n' "Traceability check FAILED:" "  - $EXEMPT" > "$work/exempt.bash"
expect pass "a finding the '@adr' row does not match keeps its file" "" \
  compare "$work/exempt.bash" 1 "$(nuc_out adr-exempt "docs/adr/${UNCITED_KEY}-a.md: traceability/coverage: ${EXEMPT#*: }")" 1

# --- cases: a run over a corpus -----------------------------------------------------------

# The check-traceability.sh of a passing tree.
TRACE_PASSES='echo "Traceability check passed (0 accepted ADRs, 0 criteria, 0 citations)."'

# source_repo <name> [README line] [traceability] — a repository whose check-docs.sh reports the
# missing release line as check 14 does whenever the README lacks a line
# '- **Template release:** v…', printed as its path; its commit is in $work/src-<name>.commit. The
# README carries the line given, by default one naming v0.5.0. Its check-traceability.sh runs the
# shell code given, by default a pass; 'none' leaves the script out.
source_repo() {
  local dir="$work/src-$1" line="${2:-- **Template release:** v0.5.0}"
  local trace="${3:-$TRACE_PASSES}"
  mkdir -p "$dir/scripts"
  [[ "$trace" == none ]] || printf '%s\n' '#!/usr/bin/env bash' "$trace" > "$dir/scripts/check-traceability.sh"
  git -C "$dir" init -q
  git -C "$dir" config uploadpack.allowAnySHA1InWant true
  # shellcheck disable=SC2016 # the script is the stub's
  printf '%s\n' '#!/usr/bin/env bash' \
    'if grep -q "^- \*\*Template release:\*\* v" README.md; then echo "Documentation checks passed."; exit 0; fi' \
    "printf 'Documentation checks FAILED:\\n\\n  - %s\\n\\n1 problem(s) found.\\n' \"$MISSING\"" \
    'exit 1' > "$dir/scripts/check-docs.sh"
  printf '# Fixture\n\n%s\n' "$line" > "$dir/README.md"
  git -C "$dir" add -A
  git -C "$dir" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -q -m fixture
  git -C "$dir" rev-parse HEAD > "$work/src-$1.commit"
  echo "$dir"
}

# corpus <file> <name>... — a corpus with the source repositories of the names, through file://.
corpus() {
  local f="$1" n; shift
  for n in "$@"; do
    printf '[[repository]]\nname = "%s"\nurl = "file://%s"\ncommit = "%s"\n' \
      "$n" "$work/src-$n" "$(cat "$work/src-$n.commit")"
  done > "$f"
  echo "$f"
}

# reactive_nuc — a nuc that reports the missing release line of the tree given by --repo, as the
# stub's check-docs.sh does.
reactive_nuc() {
  local f="$work/nuc-reactive"
  # shellcheck disable=SC2016 # the script is the stand-in's
  printf '%s\n' '#!/usr/bin/env bash' \
    'grep -q "^- \*\*Template release:\*\* v" "$3/README.md" && exit 0' \
    "echo $(printf %q "README.md: docs/template-release: $MISSING_TEXT"); exit 1" > "$f"
  chmod +x "$f"
  echo "$f"
}

# stand_in <name> <lines...> — a nuc that prints the lines whatever the tree, and exits 1 when
# there are any.
stand_in() {
  local f="$work/nuc-$1"; shift
  { echo '#!/usr/bin/env bash'; (($#)) && printf 'printf "%%s\\n" %q\n' "$@"; (($#)) && echo 'exit 1'; echo 'exit 0'; } > "$f"
  chmod +x "$f"
  echo "$f"
}

# variants <file> <rows...> — a variants file with the rows, printed as its path.
variants() { local f="$1"; shift; printf '%s\n' "$@" > "$f"; echo "$f"; }

DELETE=$'deleted\tdocs/template-release\tREADME.md\tsed\t/Template release:/d'
DEFAULT="$(variants "$work/default.tsv" "# The variant of the one ported rule." "$DELETE")"

# The table of the repository with the rows of docs/template-release alone, the one rule the
# fixtures carry.
grep -E $'^#|\tdocs/template-release\t' "$ROOT/tests/parity/normalize.tsv" > "$work/one-rule.tsv"

# run <corpus> <nuc> [variants] — the harness on the corpus, with the default variant unless given.
# shellcheck disable=SC2317,SC2329 # run through expect
run() {
  env PARITY_TABLE="$work/one-rule.tsv" bash "$HARNESS" --corpus "$1" --cache "$work/cache" --nuc "$2" \
    --variants "${3:-$DEFAULT}"
}

source_repo agree > /dev/null
c="$(corpus "$work/agree.toml" agree)"
expect pass "a corpus on which bash and nuc agree, on the tree and on its variant" "" run "$c" "$(reactive_nuc)"
expect pass "the filled cache is used again, the tree reset" "" run "$c" "$(reactive_nuc)"
check "the run names the variant it compared" \
  bash -c "PARITY_TABLE=\"$work/one-rule.tsv\" bash \"$HARNESS\" --corpus \"$c\" --cache \"$work/cache\" --nuc \"$(reactive_nuc)\" --variants \"$DEFAULT\" | grep -q 'variant deleted — 1 finding'"
check "the variant leaves the cached tree as pinned" \
  bash -c "test -z \"\$(git -C \"$work/cache/agree\" status --porcelain --untracked-files=no)\""

expect fail "a corpus on which they differ" "variant deleted — 2 difference" run "$c" "$(stand_in silent)"

source_repo trace-fails "" 'echo "Traceability check FAILED:"; exit 1' > /dev/null
expect fail "a script that fails without a finding fails the harness, though the other passes" \
  "bash: scripts/check-traceability.sh exited 1 without a finding" \
  run "$(corpus "$work/trace-fails.toml" trace-fails)" "$(reactive_nuc)"

source_repo no-trace "" none > /dev/null
expect fail "a tree without check-traceability.sh fails" "the tree has no scripts/check-traceability.sh" \
  run "$(corpus "$work/no-trace.toml" no-trace)" "$(reactive_nuc)"

printf '[[repository]]\nname = "gone"\nurl = "file://%s"\ncommit = "%s"\n' "$work/no-such-repository" \
  "$(cat "$work/src-agree.commit")" > "$work/gone.toml"
expect fail "a cache that cannot be filled fails" "the cache could not be filled" run "$work/gone.toml" "$(reactive_nuc)"

printf '[[repository]]\nname = "x"\nurl = "file:///x"\ncommit = "abc"\n' > "$work/short.toml"
expect fail "a commit that is no full SHA is refused" "a 40-hex commit" run "$work/short.toml" "$(reactive_nuc)"

printf '[[repository]]\nname = "x"\nbranch = "main"\n' > "$work/key.toml"
expect fail "a key the corpus does not have is refused" "a 40-hex commit" run "$work/key.toml" "$(reactive_nuc)"

# --- cases: the release floor -------------------------------------------------------------

n=0
for line in "- **Template release:** v0.4.9" "- **Template release:** unreleased" \
            "- **Template release:** v0.5.0-rc.1" "- **Template release:** unreleased (next after v0.5.0)" \
            "Created from the template."; do
  n=$((n + 1))
  source_repo "floor-$n" "$line" > /dev/null
  expect fail "a tree whose release line reads '$line' is refused" "not v0.5.0 or later" \
    run "$(corpus "$work/floor-$n.toml" "floor-$n")" "$(reactive_nuc)"
done

source_repo later "- **Template release:** v0.6.0." > /dev/null
expect pass "a tree of a later release, ending a sentence, is compared" "" \
  run "$(corpus "$work/later.toml" later)" "$(reactive_nuc)"

# --- cases: variants ----------------------------------------------------------------------

source_repo other "- **Template release:** v0.5.0" > /dev/null
printf 'Other\n' >> "$work/src-other/README.md"
git -C "$work/src-other" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qam other
git -C "$work/src-other" rev-parse HEAD > "$work/src-other.commit"
two="$(corpus "$work/two.toml" agree other)"
v="$(variants "$work/v-one.tsv" "$DELETE" $'other-only\tdocs/template-release\tREADME.md\tsed\t/^Other$/d')"
expect fail "a variant that does not apply to one tree of the corpus fails and names it" \
  "agree, variant other-only — its edit of README.md does not apply" run "$two" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-named.tsv" "$DELETE" $'other-only\tdocs/template-release\tREADME.md\tsed\t/Template release:/{N;/\\nOther$/d}\tother')"
expect pass "a variant that names its trees is for those alone" "" run "$two" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-named-unfit.tsv" "$DELETE" $'other-only\tdocs/template-release\tREADME.md\tsed\t/Template release:/{N;/\\nOther$/d}\tagree,other')"
expect fail "a variant that does not apply to a tree it names fails" \
  "agree, variant other-only — its edit of README.md does not apply" run "$two" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-lacks.tsv" "$DELETE" $'other-only\tdocs/template-release\tREADME.md\tsed\t/Template release:/{N;/\\nOther$/d}\tother,gone')"
expect fail "a variant that names a tree the corpus lacks fails" "names the tree 'gone', which the corpus lacks" \
  run "$two" "$(reactive_nuc)" "$v"

v="$(variants "$work/v-missing-file.tsv" "$DELETE" $'gone\tdocs/template-release\tNO-SUCH.md\tdelete\t-')"
expect fail "a variant of a file the tree lacks fails" "variant gone — its edit of NO-SUCH.md does not apply" \
  run "$c" "$(reactive_nuc)" "$v"

v="$(variants "$work/v-silent.tsv" $'silent\tdocs/template-release\tREADME.md\tsed\ts/Fixture/Changed/')"
expect fail "a variant that does not make its rule fire fails" "does not make its rule fire" run "$c" "$(reactive_nuc)" "$v"

source_repo broken "- **Template release:** v0.5.0" > /dev/null
printf '%s\n' '#!/usr/bin/env bash' \
  "printf 'Documentation checks FAILED:\\n\\n  - %s\\n\\n1 problem(s) found.\\n' \"$MISSING\"" 'exit 1' \
  > "$work/src-broken/scripts/check-docs.sh"
git -C "$work/src-broken" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qam always
git -C "$work/src-broken" rev-parse HEAD > "$work/src-broken.commit"
v="$(variants "$work/v-already.tsv" $'retitled\tdocs/template-release\tREADME.md\tsed\ts/Fixture/Changed/')"
expect fail "a finding the tree already has does not make a variant fire" "does not make its rule fire" \
  run "$(corpus "$work/broken.toml" broken)" "$(stand_in always "README.md: docs/template-release: $MISSING_TEXT")" "$v"

: > "$work/v-empty.tsv"
expect fail "a ported rule without a variant is refused" "brings no variant" run "$c" "$(reactive_nuc)" "$work/v-empty.tsv"
expect fail "an empty --variants is refused" "--variants takes a file" \
  bash "$HARNESS" --corpus "$c" --cache "$work/cache" --nuc "$(reactive_nuc)" --variants ""
printf '%s' "$DELETE" > "$work/v-no-newline.tsv"
expect pass "a last row without a newline is read" "" run "$c" "$(reactive_nuc)" "$work/v-no-newline.tsv"

source_repo linked > /dev/null
mkdir -p "$work/src-linked/docs"
git -C "$work/src-linked" mv README.md docs/real.md
ln -s docs/real.md "$work/src-linked/README.md"
stub="$work/src-linked/scripts/check-docs.sh"
sed "s/grep -q /grep -qs /" "$stub" > "$stub.tmp" && cat "$stub.tmp" > "$stub" && rm "$stub.tmp"
git -C "$work/src-linked" add -A
git -C "$work/src-linked" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qm linked
git -C "$work/src-linked" rev-parse HEAD > "$work/src-linked.commit"
v="$(variants "$work/v-link.tsv" $'unlinked\tdocs/template-release\tREADME.md\tdelete\t-')"
expect pass "a variant deletes a symbolic link itself" "" run "$(corpus "$work/linked.toml" linked)" "$(reactive_nuc)" "$v"
check "the target of the deleted link stays" test -f "$work/src-linked/docs/real.md"

v="$(variants "$work/v-new.tsv" "$DELETE" $'written\tdocs/template-release\tNEW.md\twrite\tA new file')"
expect fail "a variant that writes a new file applies, and is compared" "variant written — bash reports no finding" \
  run "$c" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-overwrite.tsv" "$DELETE" $'overwrite\tdocs/template-release\tREADME.md\twrite\tGone')"
expect fail "a variant that would write over a file does not apply" "variant overwrite — its edit of README.md does not apply" \
  run "$c" "$(reactive_nuc)" "$v"

v="$(variants "$work/v-git.tsv" "$DELETE" $'git\tdocs/template-release\t.git/commondir\twrite\t/elsewhere')"
expect fail "a variant that edits the repository's .git is refused" "no path inside the tree" run "$c" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-no-parent.tsv" "$DELETE" $'orphan\tdocs/template-release\tno-such-dir/NEW.md\twrite\tx')"
expect fail "a variant that would write into a directory the tree lacks does not apply" \
  "variant orphan — its edit of no-such-dir/NEW.md does not apply" run "$c" "$(reactive_nuc)" "$v"

v="$(variants "$work/v-escape.tsv" "$DELETE" $'escape\tdocs/template-release\t../agree/README.md\tdelete\t-')"
expect fail "a variant whose path leaves the tree is refused" "no path inside the tree" run "$c" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-absolute.tsv" "$DELETE" $'absolute\tdocs/template-release\t/etc/hostname\tdelete\t-')"
expect fail "a variant with an absolute path is refused" "no path inside the tree" run "$c" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-command.tsv" "$DELETE" "command	docs/template-release	README.md	sed	s|Fixture|touch $work/PWNED|e")"
expect fail "a sed expression that runs a command does not apply" "variant command — its edit of README.md does not apply" \
  run "$c" "$(reactive_nuc)" "$v"
check "the command of the refused expression did not run" test ! -e "$work/PWNED"
v="$(variants "$work/v-write.tsv" "$DELETE" "write	docs/template-release	README.md	sed	w $work/WRITTEN")"
expect fail "a sed expression that writes another file does not apply" "variant write — its edit of README.md does not apply" \
  run "$c" "$(reactive_nuc)" "$v"
check "the file of the refused expression was not written" test ! -e "$work/WRITTEN"

v="$(variants "$work/v-twice.tsv" "$DELETE" "$DELETE")"
expect fail "a variant named twice is refused" "is named twice" run "$c" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-malformed.tsv" $'bad name\tdocs/template-release\tREADME.md\tsed')"
expect fail "a malformed variant row is refused" "is not 'name<TAB>rule" run "$c" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-unmapped.tsv" "$DELETE" $'odd\tdocs/no-such-rule\tREADME.md\tdelete\t-')"
expect fail "a variant of a rule the table maps nothing to is refused" "maps no message to" run "$c" "$(reactive_nuc)" "$v"

summary "Parity harness self-test"
