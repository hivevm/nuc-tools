#!/usr/bin/env bash
#
# Self-test of scripts/check-parity.sh.
# Verifies: TEST-parity-harness TEST-hook-scenarios-on-bash-hooks Q-1 Q-2
# Verifies: API-sensor-cores-ask-the-shell
#
# The comparison cases feed the harness pairs of outputs through '--compare': pairs that differ in
# one finding, in one exit status, by one bash message the table does not know, or by a line nuc
# writes that is no finding fail; pairs that agree pass, a check not ported yet included.
#
# The run cases build a corpus from fixture repositories, fetched through file://, whose
# check-docs.sh is a stub that reports the missing release line as check 14 does, beside a
# check-traceability.sh that passes, and run the harness with a stand-in for nuc. Agreeing trees and
# variants pass and leave the cache as pinned; differing ones, a cache that cannot be filled, and a
# malformed corpus fail, and so do a tree whose check-traceability.sh fails without a finding while
# its check-docs.sh passes and a tree without check-traceability.sh. A comparison case reads the
# outputs of both scripts as one. The floor cases refuse a tree whose release line names a release
# before v0.5.0, a pre-release, 'unreleased', or none, read as check 14 reads it, and compare a
# later one. The variant cases fail a variant that does not apply to one tree, one that does not
# make its rule fire beyond the tree's own findings, a ported rule without one, a path outside the
# tree, a sed expression that runs a command or writes a file, a name used twice, and a malformed or
# unmapped row; a 'chomp' applies and is compared, does not apply to a file without a last newline,
# and takes '-' alone; a 'chmod' applies and is compared, does not apply to a mode left as it was or
# to a file git does not track, and takes +x or -x alone. The shell cases compare the lines of
# check-shell.sh keyed shell/ alone, map a rule nuc leaves to the bash to none, and run no
# check-shell.sh without the key of shell/mode. The hooks cases read the '  - ' lines of
# check-hooks.sh as findings, fail one that fails without a finding, and hand it the harness's
# nuc.toml with the tree's minimum and hook/ rules. The Dev Container cases read the '  - ' lines of
# check-devcontainer.sh as findings, fail one that fails without a finding, and compare its wording.
# The sensor cases compare a sensor's report byte for byte, fail an exit that differs where
# tests/parity/sensor-exits.tsv lists no row for the sensor, and pass one it lists; a tree that
# names no release has its tree rows compared with the sensor of the newest template tree, one that
# names its release with its own, and both sides run in the tree. The pin cases pass a pin of a
# copied fixture table that the tag of a template tree bears out, and fail one the tag does not, one
# whose tag cannot be read, and one without a template tree, and exit 2 on a malformed or missing
# list of pins. The shard cases cut the comparisons, tree by tree in the order of the corpus, into
# contiguous slices of about equal cost by the costs of the corpus, a tree without one taking the
# largest, the last slice ending with the cost of the whole-corpus parts; they report in the shards
# together each comparison and each finding of a tree as pinned once, as one run does, measure a
# variant against its tree as pinned in a shard that does not report that tree, fail a shard on its
# own comparisons alone, keep a tree that cannot be fetched in the deal, report an unfit variant in
# its own shard alone, read a decimal cost, fail a variant whose tree as pinned gave no whole bash
# output, run the pins, the releases, and the hook scenarios in the last shard alone, and exit 2 on
# a shard that is not <k>/<n> with 1 <= k <= n, on a cost that is no number of seconds above 0, on
# two costs in one entry, on a cost of the whole-corpus parts that is none, and on a deal that does
# not come out whole. No network and no build.
#
# The hook cases run the scenarios alone, through '--hooks', on a tree of the template whose bash
# pre-commit refuses a commit on main, with the reference shim and a stand-in nuc: an agreeing nuc
# passes; one whose exit status differs, one whose refusal maps to another rule ID or cites another
# release, one that differs only with nuc.toml, an unknown line of the bash hook, an uncovered
# self-test case, a listed case the self-test lacks, a scenario that names a case no release has, a
# scenario whose bash side does not end as its case expects, a notice whose field differs, a record
# of green trees that differs, a merge driver that writes another index, no tree at all, and a
# malformed table each fail (TEST-hook-scenarios-on-bash-hooks). A case is held only on a tree whose
# self-test names it. A tree whose .githooks/nuc.blobs pins its hooks, nuc's shims, is left out of
# the scenarios and their coverage, and a run with no other tree fails. The bound of the hooks nuc
# ports is v0.11.1 by default, and the scenario of the defaults runs on the newest tree left in.
#
# Inputs: scripts/check-parity.sh tests/parity/ scripts/lib/ shims/
#
# Usage:
#     scripts/test-check-parity.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
HARNESS="$ROOT/scripts/check-parity.sh"
# A run over a corpus replays no take-up of this repository's: the tables are empty, and the cases
# of the replay name their own (API-upgrade-by-blob-toml-optional).
echo '# None.' > "$work/take-ups-empty.tsv"
export PARITY_TAKE_UPS="$work/take-ups-empty.tsv" PARITY_TAKE_UP_DIFFERENCES="$work/take-ups-empty.tsv"

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

# keyed — each finding line of the standard input ending in a key of its rule, as nuc writes it
# (API-command-line-with-sense), unless it ends in one already; the rule is the first
# '<area>/<name>' between ': ' and ': '.
keyed() {
  awk '!/ \[[a-z]+\/[a-z0-9-]+:[a-z]+\]$/ && match($0, /: [a-z]+\/[a-z0-9-]+: /) {
    $0 = $0 " [" substr($0, RSTART + 2, RLENGTH - 4) ":x]"
  } { print }'
}

# nuc_out <name> [lines...] — the standard output of nuc with the lines, each keyed.
nuc_out() {
  local f="$work/$1.nuc"; shift
  printf '%s\n' "$@" | sed '/^$/d' | keyed > "$f"
  echo "$f"
}

# raw_nuc_out <name> [lines...] — the standard output of nuc with the lines as given.
raw_nuc_out() {
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

# --- cases: the arguments ----------------------------------------------------------------

# A root given as a first argument is refused with exit 2, not read as a corpus or a tree
# (NUC-checks-are-bash §9); expect reads only that the harness failed, so the exit is asked here.
out="$(bash "$HARNESS" "$work" 2>&1)"; rc=$?
if ((rc == 2)) && [[ "$out" == *"unknown argument '$work'"* ]]; then
  passed=$((passed + 1))
else
  failed=$((failed + 1))
  echo "FAIL: a root given as a first argument (expected exit 2 naming it, exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
fi

# --- cases: one pair ----------------------------------------------------------------------

expect pass "both pass" "" compare "$(bash_out pass)" 0 "$(nuc_out pass)" 0

expect pass "the same finding, bash without a line, nuc with one" "" \
  compare "$(bash_out same "$MALFORMED")" 1 "$(nuc_out same "README.md:5: docs/template-release: $MALFORMED_TEXT")" 1

expect fail "a finding of nuc without its key" "without its key" \
  compare "$(bash_out unkeyed "$MALFORMED")" 1 "$(raw_nuc_out unkeyed "README.md:5: docs/template-release: $MALFORMED_TEXT")" 1

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

# From v0.9.1 on the bash ends a message in its key too, whose rule ID is the one of its release.
# Its reason is compared with nuc's; its rule ID, which an earlier release names otherwise, is not.
expect pass "a bash message's own key is no part of its wording" "" \
  compare "$(bash_out keyed-bash "$MALFORMED [docs/template-release:version]")" 1 \
  "$(nuc_out keyed-bash "README.md:5: docs/template-release: $MALFORMED_TEXT [docs/template-release:version]")" 1
expect pass "a bash key of another rule ID, as an earlier release names it, is left out as well" "" \
  compare "$(bash_out keyed-old "$MALFORMED [docs/template-version:version]")" 1 \
  "$(nuc_out keyed-old "README.md:5: docs/template-release: $MALFORMED_TEXT [docs/template-release:version]")" 1
# A message the bash writes without its key, as v0.9.1 writes one of the traceability, has no
# reason to compare.
expect pass "a bash message without a key among keyed ones compares no reason" "" \
  compare "$(bash_out keyed-some "$MALFORMED [docs/template-release:version]" "$MISSING")" 1 \
  "$(nuc_out keyed-some "README.md:5: docs/template-release: $MALFORMED_TEXT [docs/template-release:version]" \
    "README.md: docs/template-release: $MISSING_TEXT [docs/template-release:line]")" 1
expect fail "a reason of nuc other than the bash's is a difference" "nuc gives the reason 'line' where bash gives 'version'" \
  compare "$(bash_out keyed-other "$MALFORMED [docs/template-release:version]")" 1 \
  "$(nuc_out keyed-other "README.md:5: docs/template-release: $MALFORMED_TEXT [docs/template-release:line]")" 1

# The sensors (API-sensor-cores-ask-the-shell): the standard output byte for byte, and an exit
# that differs only where tests/parity/sensor-exits.tsv lists it.
# shellcheck disable=SC2317,SC2329 # run through expect
sense() { bash "$HARNESS" --sense-compare "$@"; }
printf 'Design revision sensor: 1 of 2 changes outside docs/ since the first commit — not yet due.\n' > "$work/sense.a"
cp "$work/sense.a" "$work/sense.same"
sed 's/1 of 2/1 of 3/' "$work/sense.a" > "$work/sense.byte"
: > "$work/sense.empty"
expect pass "a sensor's report the same from both, exits alike, passes" "" \
  sense revision-due v0.12.0 "$work/sense.a" 0 "$work/sense.same" 0
expect fail "a sensor's report that differs in one byte is a difference" "the standard output differs" \
  sense revision-due v0.12.0 "$work/sense.a" 0 "$work/sense.byte" 0
expect fail "a sensor's exit that differs where no row lists it is a difference" "which no row of" \
  sense revision-due v0.12.0 "$work/sense.a" 0 "$work/sense.same" 2
expect pass "a sensor's exit that differs where a row lists it passes" "" \
  sense revision-due v0.12.0 "$work/sense.empty" 1 "$work/sense.empty" 2
expect fail "a listed exit of one sensor does not hold for another" "which no row of" \
  sense backlog v0.12.0 "$work/sense.empty" 1 "$work/sense.empty" 2

# The tree rows of a sensor: a tree whose README names no release is compared with the bash sensor
# of the newest template tree, SENSOR_NEWEST_TREE, as nuc sense reports there as the newest
# release it knows, also where the tree itself has no such sensor; a tree that names its release
# with its own.
# sensor_stub <dir> <report> — a tree whose sensor x writes the report.
sensor_stub() {
  mkdir -p "$1/scripts"
  printf '#!/usr/bin/env bash\necho %s\n' "$2" > "$1/scripts/sensor-x.sh"
}
sensor_stub "$work/st-old" old
sensor_stub "$work/st-newest" new
mkdir -p "$work/st-none"
printf '# T\n\n- **Template release:** v0.6.2\n' > "$work/st-old/README.md"
printf '# T\n\n- **Template release:** v0.12.0\n' > "$work/st-newest/README.md"
printf '# T\n' > "$work/st-none/README.md"
cp -R "$work/st-old" "$work/st-unnamed" && printf '# T\n' > "$work/st-unnamed/README.md"
printf '#!/usr/bin/env bash\necho new\n' > "$work/st-nuc" && chmod +x "$work/st-nuc"
printf 'x\ttree\t-\n' > "$work/st-table.tsv"
: > "$work/st-scenarios.sh"
# shellcheck disable=SC2317,SC2329 # run through expect
tree_rows() {
  (
    eval "$(sed -n '/^release_of() {/,/^}/p' "$HARNESS")"
    export PARITY_SENSOR_TABLE="$work/st-table.tsv" PARITY_SENSOR_SCENARIOS="$work/st-scenarios.sh"
    # shellcheck source=/dev/null # the library, read as the harness reads it
    source "$ROOT/scripts/lib/sensor-parity.sh"
    load_sensor_tables || exit 2
    # shellcheck disable=SC2034 # read by sensor_tree of the library
    SENSOR_NEWEST_TREE="$1"
    out="$work/st-out-${2##*/}"
    sensor_tree "${3:-$work/st-nuc}" "${2##*/}" "$2" "$out" || exit 1
    [[ -f "$out.sensor-0.sb" ]] || { echo "no sensor ran"; exit 1; }
  )
}
expect pass "a tree that names no release is compared with the newest tree's sensor" "" \
  tree_rows "$work/st-newest" "$work/st-unnamed"
expect fail "without the newest tree, the tree's own sensor differs" "the standard output differs" \
  tree_rows "" "$work/st-unnamed"
expect pass "a tree that names no release and lacks the sensor still compares it" "" \
  tree_rows "$work/st-newest" "$work/st-none"
expect fail "a tree that names its release is compared with its own sensor" "the standard output differs" \
  tree_rows "$work/st-newest" "$work/st-old"

# The rows of other states run SENSOR_JOBS at once, and report in the order of the trees and the
# rows, whatever order they end in: the first row sleeps, so that with eight jobs it ends last.
printf '%s\n' 'st_slow() { sleep 0.5; }' 'st_built() { :; }' 'st_broken() { return 1; }' \
  > "$work/st-scen.sh"
# scenarios_run <name> <jobs> <table> <tree>... — the rows of the table on the trees, with the
# jobs; the output and the exit into st-scen-<name>.out.
# shellcheck disable=SC2317,SC2329 # run through check
scenarios_run() {
  local name="$1" jobs="$2" table="$3"
  shift 3
  (
    eval "$(sed -n '/^release_of() {/,/^}/p' "$HARNESS")"
    # shellcheck source=/dev/null # the library, read as the harness reads it
    source "$ROOT/scripts/lib/sensor-parity.sh"
    # The library's own names of its tables, set after it read the defaults.
    # shellcheck disable=SC2034 # read by load_sensor_tables of the library
    SENSOR_TABLE="$table" SENSOR_SCENARIOS="$work/st-scen.sh"
    load_sensor_tables || exit 2
    mkdir -p "$work/st-scen-work-$name"
    SENSOR_JOBS="$jobs" sensor_scenarios "$work/st-nuc" "$work/st-scen-work-$name" "$@"
  ) > "$work/st-scen-$name.out" 2>&1
  echo "exit $?" >> "$work/st-scen-$name.out"
}
# Two rows that differ on the old tree, and one whose fixture cannot be built, on both trees.
printf 'x\tst_slow\t-\nx\tst_broken\t-\nx\tst_built\t-\n' > "$work/st-scen-mixed.tsv"
scenarios_run mixed-1 1 "$work/st-scen-mixed.tsv" "$work/st-old" "$work/st-newest"
scenarios_run mixed-8 8 "$work/st-scen-mixed.tsv" "$work/st-old" "$work/st-newest"
check "one scenario job and eight report the same, in the same order" \
  cmp -s "$work/st-scen-mixed-1.out" "$work/st-scen-mixed-8.out"
check "the scenarios fail where one differs or cannot be built" \
  grep -qx 'exit 1' "$work/st-scen-mixed-8.out"
# scenarios_ordered — whether, with eight jobs, the old tree's slow row comes before its last row,
# and both before the newest tree's broken state.
# shellcheck disable=SC2317,SC2329 # run through check
scenarios_ordered() {
  local out="$work/st-scen-mixed-8.out" a b c
  a="$(grep -n 'v0.6.2, state st_slow' "$out" | head -n 1 | cut -d: -f1)"
  b="$(grep -n 'v0.6.2, state st_built' "$out" | head -n 1 | cut -d: -f1)"
  c="$(grep -n 'v0.12.0, sensor x — the state st_broken could not be built' "$out" | cut -d: -f1)"
  [[ -n "$a" && -n "$b" && -n "$c" ]] && ((a < b && b < c))
}
check "the scenarios report in the order of the trees and the rows" scenarios_ordered
check "the scenarios leave no fixture behind" \
  bash -c "[[ -z \"\$(ls -A \"$work/st-scen-work-mixed-8/sensor-scenarios\")\" ]]"
# A fixture that cannot be built fails the run by itself, beside a row that agrees.
printf 'x\tst_built\t-\nx\tst_broken\t-\n' > "$work/st-scen-broken.tsv"
scenarios_run broken 8 "$work/st-scen-broken.tsv" "$work/st-newest"
check "a fixture that cannot be built fails the scenarios by itself" \
  grep -qx 'exit 1' "$work/st-scen-broken.out"
check "the run names the state that could not be built" \
  grep -q 'the state st_broken could not be built' "$work/st-scen-broken.out"
# Rows that agree pass, and the run counts them.
printf 'x\tst_slow\t-\nx\tst_built\t-\n' > "$work/st-scen-agree.tsv"
scenarios_run agree 8 "$work/st-scen-agree.tsv" "$work/st-newest"
check "scenarios that agree pass" grep -qx 'exit 0' "$work/st-scen-agree.out"
check "the run counts the scenarios it compared" \
  grep -qx 'Parity check: the sensor scenarios — 2 run(s), the same from bash and nuc.' \
  "$work/st-scen-agree.out"

# Both sides run in the tree, so that a path an argument names from where the sensor runs, as
# --hand-off's, names the same file there: each side prints where it runs, the other where it must.
mkdir -p "$work/st-cwd-bash/scripts" "$work/st-cwd-nuc/scripts"
for side in bash nuc; do
  printf '# T\n\n- **Template release:** v0.12.0\n' > "$work/st-cwd-$side/README.md"
done
printf '#!/usr/bin/env bash\npwd\n' > "$work/st-cwd-bash/scripts/sensor-x.sh"
printf '#!/usr/bin/env bash\necho %q\n' "$(cd "$work/st-cwd-bash" && pwd)" > "$work/st-nuc-cwd-bash"
printf '#!/usr/bin/env bash\necho %q\n' "$(cd "$work/st-cwd-nuc" && pwd)" \
  > "$work/st-cwd-nuc/scripts/sensor-x.sh"
printf '#!/usr/bin/env bash\npwd\n' > "$work/st-nuc-cwd-nuc"
chmod +x "$work/st-nuc-cwd-bash" "$work/st-nuc-cwd-nuc"
expect pass "the bash sensor runs in the tree" "" \
  tree_rows "" "$work/st-cwd-bash" "$work/st-nuc-cwd-bash"
expect pass "nuc sense runs in the tree" "" \
  tree_rows "" "$work/st-cwd-nuc" "$work/st-nuc-cwd-nuc"

# A record named by category: the ADR's file is its identifier.
NAMED="$(printf '%s-%s' SEC a-b)"
printf '%s\n' "Traceability check FAILED:" "  - ${UNCITED/"$UNCITED_ID"/$NAMED}" > "$work/named.bash"
expect pass "an ADR named by category is placed at its file" "" \
  compare "$work/named.bash" 1 "$(nuc_out adr-named "docs/adr/$NAMED.md: traceability/coverage: ${UNCITED/"$UNCITED_ID"/$NAMED}")" 1
expect fail "an ADR named by category at another file is a difference" "only bash reports: traceability/coverage docs/adr/$NAMED" \
  compare "$work/named.bash" 1 "$(nuc_out adr-named-other "docs/adr/$NAMED-c.md: traceability/coverage: ${UNCITED/"$UNCITED_ID"/$NAMED}")" 1

# A place '@word' names the path the message opens with before its first space.
CHECKS="$(printf '%s-%s' NUC checks-are-bash)"
SENSOR="has no self-test scripts/test-sensor-x.sh — each sensor runs after one that cites $CHECKS"
printf '%s\n' "Traceability check FAILED:" "  - scripts/sensor-x.sh $SENSOR" > "$work/word.bash"
expect pass "a message that opens with its path and a space is placed at that path" "" \
  compare "$work/word.bash" 1 "$(nuc_out word "scripts/sensor-x.sh: traceability/sensor-tests: $SENSOR")" 1
expect fail "a finding of such a message at another path is a difference" "only bash reports: traceability/sensor-tests scripts/sensor-x.sh" \
  compare "$work/word.bash" 1 "$(nuc_out word-other "scripts/sensor-y.sh: traceability/sensor-tests: $SENSOR")" 1

# A path holding ': ' is placed whole on both sides, at the file nuc names.
COLON="docs/a: b.md: broken relative link -> 'x.md'"
expect pass "a path holding ': ' is placed at the same file" "" \
  compare "$(bash_out colon "$COLON")" 1 "$(nuc_out colon "docs/a: b.md: docs/links: broken relative link -> 'x.md'")" 1
expect fail "a path holding ': ' that nuc names otherwise is a difference" "only nuc reports: docs/links docs/a: c.md" \
  compare "$(bash_out colon-other "$COLON")" 1 "$(nuc_out colon-other "docs/a: c.md: docs/links: broken relative link -> 'x.md'")" 1

# A path holding ': ' at a line, and a message that quotes ': <rule ID>: ', are placed whole.
CITED="parity: variant.txt:1: cites 'Q-999', which is not a criterion of docs/SPECIFICATION.md"
printf '%s\n' "Traceability check FAILED:" "  - $CITED" > "$work/colon-line.bash"
expect pass "a path holding ': ' at a line is placed at the same file and line" "" \
  compare "$work/colon-line.bash" 1 "$(nuc_out colon-line "parity: variant.txt:1: traceability/citations: ${CITED#*:1: }")" 1
QUOTED="README.md: broken relative link -> 'q: docs/links: z'"
expect pass "a message quoting ': <rule ID>: ' keeps its text" "" \
  compare "$(bash_out quoted "$QUOTED")" 1 "$(nuc_out quoted "README.md: docs/links: ${QUOTED#README.md: }")" 1

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

# corpus <file> <name>[:<cost>]... — a corpus with the source repositories of the names, through
# file://, each with the cost given.
corpus() {
  local f="$1" n; shift
  for n in "$@"; do
    printf '[[repository]]\nname = "%s"\nurl = "file://%s"\ncommit = "%s"\n' \
      "${n%%:*}" "$work/src-${n%%:*}" "$(cat "$work/src-${n%%:*}.commit")"
    [[ "$n" == *:* ]] && printf 'cost = %s\n' "${n#*:}"
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
    "echo $(printf %q "README.md: docs/template-release: $MISSING_TEXT [docs/template-release:line]"); exit 1" > "$f"
  chmod +x "$f"
  echo "$f"
}

# stand_in <name> <lines...> — a nuc that prints the lines whatever the tree, and exits 1 when
# there are any.
stand_in() {
  local f="$work/nuc-$1"; shift
  local lines=()
  (($#)) && mapfile -t lines < <(printf '%s\n' "$@" | keyed)
  { echo '#!/usr/bin/env bash'; ((${#lines[@]})) && printf 'printf "%%s\\n" %q\n' "${lines[@]}"; ((${#lines[@]})) && echo 'exit 1'; echo 'exit 0'; } > "$f"
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
# No pin of a copied fixture table, but where a case gives one: the fixtures' corpora hold no tag of
# a template release.
: > "$work/no-blobs"
export PARITY_TABLE_BLOBS="$work/no-blobs"

# run <corpus> <nuc> [variants] — the harness on the corpus, with the default variant unless given.
# shellcheck disable=SC2317,SC2329 # run through expect
run() {
  env PARITY_TABLE="$work/one-rule.tsv" bash "$HARNESS" --no-hooks --corpus "$1" --cache "$work/cache" --nuc "$2" \
    --variants "${3:-$DEFAULT}"
}

source_repo agree > /dev/null
c="$(corpus "$work/agree.toml" agree)"
expect pass "a corpus on which bash and nuc agree, on the tree and on its variant" "" run "$c" "$(reactive_nuc)"
expect pass "the filled cache is used again, the tree reset" "" run "$c" "$(reactive_nuc)"
check "the run names the variant it compared" \
  bash -c "PARITY_TABLE=\"$work/one-rule.tsv\" bash \"$HARNESS\" --no-hooks --corpus \"$c\" --cache \"$work/cache\" --nuc \"$(reactive_nuc)\" --variants \"$DEFAULT\" | grep -c 'variant deleted — 1 finding' > /dev/null"
check "the variant leaves the cached tree as pinned" \
  bash -c "test -z \"\$(git -C \"$work/cache/agree\" status --porcelain --untracked-files=no)\""

expect fail "a corpus on which they differ" "variant deleted — 2 difference" run "$c" "$(stand_in silent)"

# check-shell.sh of a tree that has the rule shell/mode: only its lines keyed shell/ are findings,
# a rule nuc leaves to the bash maps to none, and a tree whose script lacks the rule's key is not
# run. shelled <name> <has key: 1|0> <lines...> — a source repository whose check-shell.sh prints
# a line of ShellCheck and the lines, and exits 1.
ROOT_FINDING="scripts/x.sh: calls no enter_root on a line of its own — x [shell/root:unrooted]"
MODE_FINDING="scripts/x.sh: is tracked with mode 100644, not 100755 (NUC-checks-are-bash §9), so a direct call is refused — x [shell/mode:exec]"
shelled() {
  local src key="$2" name="$1"; shift 2
  src="$(source_repo "$name")"
  { echo '#!/usr/bin/env bash'
    ((key)) && echo '# [shell/mode:exec]'
    echo 'echo "In scripts/x.sh line 1: SC2034"'
    printf 'echo %q\n' "$@"
    echo 'exit 1'; } > "$src/scripts/check-shell.sh"
  git -C "$src" add -A
  git -C "$src" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -q -m shelled
  git -C "$src" rev-parse HEAD > "$work/src-$name.commit"
  corpus "$work/$name.toml" "$name"
}
# The table of the run cases holds the rows of the rules nuc leaves to the bash, not shell/mode's,
# which would bring a variant; the case after them compares a finding of shell/mode by its row.
grep -E $'^#|\tdocs/template-release\t|^\\^\\[\\^ \\]\\.\\*: .*\t-\t-$' "$ROOT/tests/parity/normalize.tsv" > "$work/shell.tsv"
# shellcheck disable=SC2317,SC2329 # run through expect
shell_run() {
  env PARITY_TABLE="$work/shell.tsv" bash "$HARNESS" --no-hooks --corpus "$1" --cache "$work/cache" --nuc "$2" \
    --variants "$DEFAULT"
}
expect pass "a finding of check-shell.sh of a rule nuc leaves to the bash is no difference, nor its other lines" "" \
  shell_run "$(shelled shell-root 1 "$ROOT_FINDING")" "$(reactive_nuc)"
expect fail "a line of check-shell.sh keyed shell/mode is read as a finding" "bash message the table does not know" \
  shell_run "$(shelled shell-mode 1 "$MODE_FINDING")" "$(reactive_nuc)"
expect pass "a check-shell.sh without the key of shell/mode is not run" "" \
  shell_run "$(shelled shell-none 0 "$MODE_FINDING")" "$(reactive_nuc)"
MODE_TEXT="${MODE_FINDING#scripts/x.sh: }"
expect pass "a finding of shell/mode maps to the rule by its row" "" \
  compare "$(bash_out shell-mode "$MODE_FINDING")" 1 "$(raw_nuc_out shell-mode "scripts/x.sh: shell/mode: $MODE_TEXT")" 1
expect fail "a finding of shell/mode nuc does not report is a difference" "only bash reports: shell/mode scripts/x.sh" \
  compare "$(bash_out shell-mode-only "$MODE_FINDING")" 1 "$(nuc_out shell-mode-only)" 0

# check-hooks.sh of a tree that has it: its '  - ' lines are findings, and it fails only with one.
# The harness's nuc.toml keeps the tree's minimum and its hook/ rules, read as check-hooks.sh reads
# them, a rule in a comment left out, and adds no other rule of the tree's.
# hooked <name> <script body> — a source repository whose check-hooks.sh runs the body, and whose
# nuc.toml names the minimum 0.2.2, a hook/ rule, one in a comment, and a rule of docs/.
TOML_FINDING="nuc.toml: its rules do not name 'hook/x' — with a nuc.toml its list governs which rules run, and no project removes a hook/ rule (NUC-hooks-run-nuc §3) [hooks/toml:rule]"
hooked() {
  local src name="$1"
  src="$(source_repo "$name")"
  printf '%s\n' '#!/usr/bin/env bash' "$2" > "$src/scripts/check-hooks.sh"
  printf '%s\n' '# The nuc of the hooks.' 'minimum = "0.2.2"' 'rules = [' '  "hook/push-checks", # "hook/x",' \
    '  "docs/x",' ']' > "$src/nuc.toml"
  git -C "$src" add -A
  git -C "$src" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -q -m hooked
  git -C "$src" rev-parse HEAD > "$work/src-$name.commit"
  corpus "$work/$name.toml" "$name"
}
# The harness's nuc.toml as the tree's check-hooks.sh sees it: a finding unless it is the one the
# tree's own minimum and hook/ rules make.
# shellcheck disable=SC2016 # the body is the stub's
KEPT='grep -qx "minimum = \"0.2.2\"" nuc.toml && grep -q ",\"hook/push-checks\"\]$" nuc.toml && ! grep -q "docs/x\|hook/x" nuc.toml && exit 0
printf "Hooks pin check FAILED:\n  - %s\n" "$(cat nuc.toml)"; exit 1'
expect pass "the harness's nuc.toml keeps the tree's minimum and hook/ rules, and nothing in a comment" "" \
  run "$(hooked hooks-kept "$KEPT")" "$(reactive_nuc)"
expect fail "a line of check-hooks.sh is read as a finding" "bash message the table does not know" \
  run "$(hooked hooks-finding "printf '  - %s\n' $(printf %q "$TOML_FINDING"); exit 1")" "$(reactive_nuc)"
expect fail "a check-hooks.sh that fails without a finding fails the harness" "check-hooks.sh exited 1 without a finding" \
  run "$(hooked hooks-silent 'exit 1')" "$(reactive_nuc)"

# check-devcontainer.sh of a tree that has it, every release from v0.5.0 on: its '  - ' lines are
# findings, and it fails only with one; its finding maps to the rule by its row, the wording of
# each release compared.
# devcontained <name> <script body> — a source repository whose check-devcontainer.sh runs the
# body.
devcontained() {
  local src name="$1"
  src="$(source_repo "$name")"
  printf '%s\n' '#!/usr/bin/env bash' "$2" > "$src/scripts/check-devcontainer.sh"
  git -C "$src" add -A
  git -C "$src" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -q -m devcontained
  git -C "$src" rev-parse HEAD > "$work/src-$name.commit"
  corpus "$work/$name.toml" "$name"
}
LOCK_TEXT="not found — the lock is committed so that the same commit yields the same Features"
LOCK_FINDING=".devcontainer/devcontainer-lock.json: $LOCK_TEXT (NUC-container-hides-host-daemon §3) [devcontainer/lock:absent]"
expect pass "a check-devcontainer.sh that passes adds no finding" "" \
  run "$(devcontained devcontainer-passes 'echo "Dev Container check passed."')" "$(reactive_nuc)"
expect fail "a line of check-devcontainer.sh is read as a finding" "bash message the table does not know" \
  run "$(devcontained devcontainer-finding "printf 'Dev Container check FAILED:\n  - %s\n' $(printf %q "$LOCK_FINDING") >&2; exit 1")" "$(reactive_nuc)"
expect fail "a check-devcontainer.sh that fails without a finding fails the harness" "check-devcontainer.sh exited 1 without a finding" \
  run "$(devcontained devcontainer-silent 'exit 1')" "$(reactive_nuc)"
expect pass "a finding of devcontainer/lock maps to the rule by its row" "" \
  compare "$(bash_out devcontainer-lock "$LOCK_FINDING")" 1 \
  "$(raw_nuc_out devcontainer-lock ".devcontainer/devcontainer-lock.json: devcontainer/lock: $LOCK_TEXT (NUC-container-hides-host-daemon §3) [devcontainer/lock:absent]")" 1
OLD_LOCK="$(printf '.devcontainer/devcontainer-lock.json: %s (%s-0004)' "$LOCK_TEXT" ADR)"
expect fail "a finding of devcontainer/lock worded as another release is a difference" "only bash reports: devcontainer/lock .devcontainer/devcontainer-lock.json" \
  compare "$(bash_out devcontainer-old "$OLD_LOCK")" 1 \
  "$(raw_nuc_out devcontainer-old ".devcontainer/devcontainer-lock.json: devcontainer/lock: $LOCK_TEXT (NUC-container-hides-host-daemon §3) [devcontainer/lock:absent]")" 1

# The pins of the copied fixture tables (TEST-template-table-cases): a tree of the template,
# TEMPLATE-SETUP.md in it, whose tag v9.9.9 the harness fetches from its source through file://.
src="$(source_repo tables)"
mkdir -p "$src/scripts/fixtures" && echo "a case" > "$src/scripts/fixtures/t.txt"
echo "# Template setup" > "$src/TEMPLATE-SETUP.md"
git -C "$src" add -A
git -C "$src" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -q -m tables
git -C "$src" tag v9.9.9
git -C "$src" rev-parse HEAD > "$work/src-tables.commit"
tables="$(corpus "$work/tables.toml" tables)"
printf 'v9.9.9 %s t.txt\n' "$(git -C "$src" rev-parse v9.9.9:scripts/fixtures/t.txt)" > "$work/blobs-good"
printf 'v9.9.9 %s t.txt\n' "$(printf 'other\n' | git hash-object --stdin)" > "$work/blobs-other"
sed 's/^v9.9.9/v9.9.8/' "$work/blobs-good" > "$work/blobs-no-tag"
# pinned <blobs> <corpus> — the harness on the corpus, the pins those of the file.
# shellcheck disable=SC2317,SC2329 # run through expect
pinned() { PARITY_TABLE_BLOBS="$1" run "$2" "$(reactive_nuc)"; }
expect pass "a pin the template's tag bears out" "" pinned "$work/blobs-good" "$tables"
expect fail "a pin the template's tag does not bear out" "t.txt is not the blob v9.9.9 holds at scripts/fixtures/t.txt" \
  pinned "$work/blobs-other" "$tables"
expect fail "a pin whose tag cannot be read" "the tag v9.9.8 cannot be read" pinned "$work/blobs-no-tag" "$tables"
expect fail "a pin without a tree of the template in the corpus" "no tree of the template in the corpus" \
  pinned "$work/blobs-good" "$c"

source_repo trace-fails "" 'echo "Traceability check FAILED:"; exit 1' > /dev/null
expect fail "a script that fails without a finding fails the harness, though the other passes" \
  "bash: scripts/check-traceability.sh exited 1 without a finding" \
  run "$(corpus "$work/trace-fails.toml" trace-fails)" "$(reactive_nuc)"

# The comparisons run in parallel; a run reads the same whatever the number of jobs, findings and
# failures in the order of the corpus and the variants.
source_repo agree-too > /dev/null
many="$(corpus "$work/many.toml" agree trace-fails agree-too)"
# jobs_run <n> — the harness on three trees, two of which differ, with n jobs; its output and exit.
# shellcheck disable=SC2317,SC2329 # run through check
jobs_run() {
  PARITY_JOBS="$1" run "$many" "$(stand_in silent)" > "$work/jobs-$1.out" 2>&1
  echo "exit $?" >> "$work/jobs-$1.out"
}
jobs_run 1
jobs_run 8
check "one job and eight report the same, in the same order" cmp -s "$work/jobs-1.out" "$work/jobs-8.out"
check "the run with eight jobs fails as the run with one does" grep -qx 'exit 1' "$work/jobs-8.out"
check "the run names each tree's difference" \
  bash -c "grep -q 'agree at .*variant deleted — 2 difference' \"$work/jobs-8.out\" && grep -q 'agree-too at .*variant deleted — 2 difference' \"$work/jobs-8.out\""
# run_jobs <n> <corpus> <nuc> <variants> — the harness as run does it, with n jobs.
# shellcheck disable=SC2317,SC2329 # run through expect
run_jobs() { local n="$1"; shift; PARITY_JOBS="$n" run "$@"; }

# The variants are measured against the tree's own output only once it is whole: a tree whose own
# run is slow and already reports the finding, and a variant that adds none, which runs fast.
source_repo slow-bare > /dev/null
d="$work/src-slow-bare"
printf '%s\n' '#!/usr/bin/env bash' '[[ -f notes.txt ]] || sleep 2' \
  "printf 'Documentation checks FAILED:\\n\\n  - %s\\n\\n1 problem(s) found.\\n' \"$MISSING\"" \
  'exit 1' > "$d/scripts/check-docs.sh"
git -C "$d" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qam slow
git -C "$d" rev-parse HEAD > "$work/src-slow-bare.commit"
NOTES=$'notes\tdocs/template-release\tnotes.txt\twrite\tx'
expect fail "a variant is measured against the tree's own output once that is whole" \
  "variant notes — bash reports no finding of docs/template-release beyond the tree's own" \
  run_jobs 8 "$(corpus "$work/slow.toml" slow-bare)" \
    "$(stand_in always "README.md: docs/template-release: $MISSING_TEXT")" "$(variants "$work/notes.tsv" "$NOTES")"

# The outputs follow the corpus and the variants, not the order in which the jobs end: a slow first
# tree and a fast second, two variants each.
source_repo slow-first > /dev/null
d="$work/src-slow-first"
f="$d/scripts/check-docs.sh"
sed '2i sleep 2' "$f" > "$f.tmp" && cat "$f.tmp" > "$f" && rm "$f.tmp"
git -C "$d" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qam slow
git -C "$d" rev-parse HEAD > "$work/src-slow-first.commit"
RENAMED=$'renamed\tdocs/template-release\tREADME.md\tsed\ts/Template release:/Template rel:/'
two="$(variants "$work/two.tsv" "$DELETE" "$RENAMED")"
PARITY_JOBS=8 run "$(corpus "$work/order.toml" slow-first agree)" "$(reactive_nuc)" "$two" \
  | sed -nE 's/^Parity check: ([a-z-]+) at [0-9a-f]+(, variant ([a-z]+))? — .*/\1 \3/p' > "$work/order.out"
printf '%s\n' 'slow-first ' 'slow-first deleted' 'slow-first renamed' 'agree ' 'agree deleted' \
  'agree renamed' > "$work/order.want"
check "the outputs follow the corpus and the variants, not the end of the jobs" \
  cmp -s "$work/order.want" "$work/order.out"
expect fail "a repository named twice in the corpus is refused" "the repository 'agree' is named twice" \
  run "$(corpus "$work/twice.toml" agree agree)" "$(reactive_nuc)"
expect fail "a number of jobs that is none is refused" "PARITY_JOBS is 'many'" \
  env PARITY_JOBS=many PARITY_TABLE="$work/one-rule.tsv" bash "$HARNESS" --no-hooks --corpus "$many" \
    --cache "$work/cache" --nuc "$(reactive_nuc)" --variants "$DEFAULT"
# exits_two <command...> — whether the command exits 2, as a malformed input does.
# shellcheck disable=SC2317,SC2329 # run through check
exits_two() { "$@" > /dev/null 2>&1; (($? == 2)); }

# A shard makes a contiguous slice of the comparisons, each tree as pinned and each variant one,
# tree by tree in the order of the corpus, the slices of about equal cost by the costs of the
# corpus. The cases that a shard compares the trees at its places, 1/2 the first and third tree
# and 2/2 the second, are replaced, since their premise, the place of a tree, is gone: the cases
# below hold the slices, their cost, and a tree cut by a slice instead.
# run_shard <shard> <corpus> <nuc> [variants] — the harness as run does it, on one shard; a
# PARITY_WHOLE_COST in the environment holds.
# shellcheck disable=SC2317,SC2329 # run through expect
run_shard() {
  env PARITY_TABLE="$work/one-rule.tsv" bash "$HARNESS" --no-hooks --shard "$1" --corpus "$2" \
    --cache "$work/cache" --nuc "$3" --variants "${4:-$DEFAULT}"
}
# reported <file> — the trees and variants a run's output compared, in the order of its output.
reported() { sed -nE 's/^Parity check: ([a-z-]+) at [0-9a-f]+(, variant [a-z0-9]+)? — .*/\1\2/p' "$1"; }
# names <file> — the same, sorted, each as often as reported.
names() { reported "$1" | sort; }
# slice <file> <shard> <corpus> [variants] — what the shard reports, in order, into the file.
slice() { run_shard "$2" "$3" "$(stand_in silent)" "${4:-$DEFAULT}" > "$1.out" 2>&1; reported "$1.out" > "$1"; }
# lists <file> <line>... — whether the file holds the lines, in this order, and nothing else.
# shellcheck disable=SC2317,SC2329 # run through check
lists() { local f="$1"; shift; cmp -s "$f" <((($#)) && printf '%s\n' "$@"); }
# lines_are <file> <n> — whether the file has n lines.
# shellcheck disable=SC2317,SC2329 # run through check
lines_are() { (($(wc -l < "$1") == $2)); }

# The slices are contiguous, cut where the cost is: on agree, trace-fails, agree-too of cost 1
# each, one variant each, and no cost for the whole-corpus parts, four shards cut the six
# comparisons 1, 2, 1, 2, so that 2/4 and 3/4 each begin inside a tree.
uniform="$(corpus "$work/uniform.toml" agree:1 trace-fails:1 agree-too:1)"
for sh in 1/4 2/4 3/4 4/4; do PARITY_WHOLE_COST=0 slice "$work/cut-${sh/\//-}" "$sh" "$uniform"; done
check "the slice 1/4 is agree as pinned" lists "$work/cut-1-4" agree
check "the slice 2/4 is agree's variant and trace-fails as pinned" \
  lists "$work/cut-2-4" 'agree, variant deleted' trace-fails
check "the slice 3/4 is trace-fails's variant" lists "$work/cut-3-4" 'trace-fails, variant deleted'
check "the slice 4/4 is agree-too as pinned and its variant" \
  lists "$work/cut-4-4" agree-too 'agree-too, variant deleted'
run_shard 1/1 "$uniform" "$(stand_in silent)" > "$work/cut-1-1.out" 2>&1
names "$work/cut-1-1.out" > "$work/cut-all.names"
cat "$work"/cut-[1-4]-4 | sort > "$work/cut-quarters.names"
check "a run without --shard compares every tree as pinned and every variant" \
  lines_are "$work/cut-all.names" 6
check "four shards together report each comparison once, as one run does" \
  cmp -s "$work/cut-all.names" "$work/cut-quarters.names"
# A tree cut by a slice is compared as pinned in both, and reported by the slice that holds it: on
# agree and agree-too of cost 1, four shards take one comparison each, and a nuc that reports the
# missing release line on every tree differs from bash on each tree as pinned; the four shards
# report each such finding once, as one run does.
pair="$(corpus "$work/pair.toml" agree:1 agree-too:1)"
always="$(stand_in always "README.md: docs/template-release: $MISSING_TEXT")"
run_shard 1/1 "$pair" "$always" 2>&1 | grep 'only nuc reports' > "$work/once-all.found"
for sh in 1/4 2/4 3/4 4/4; do
  PARITY_WHOLE_COST=0 run_shard "$sh" "$pair" "$always" > "$work/once-${sh/\//-}.out" 2>&1
done
cat "$work"/once-[1-4]-4.out | grep 'only nuc reports' > "$work/once-quarters.found"
check "the one run reports the finding of each tree as pinned that nuc alone reports" \
  lines_are "$work/once-all.found" 2
check "the shards together report each finding of a tree as pinned once" \
  cmp -s "$work/once-all.found" "$work/once-quarters.found"
# A shard fails on its own comparisons alone: 1/4 holds agree as pinned, which differs, and 2/4
# holds only agree's variant, which agrees, measured against agree as pinned, which 2/4 compares,
# does not report, and does not fail on.
PARITY_WHOLE_COST=0 expect fail "the shard that holds a differing tree as pinned fails" "agree at" \
  run_shard 1/4 "$pair" "$always"
PARITY_WHOLE_COST=0 expect pass "a shard with only a variant of a tree whose comparison as pinned differs passes" "" \
  run_shard 2/4 "$pair" "$always"
check "that shard leaves the tree as pinned out of its output" \
  lists <(PARITY_WHOLE_COST=0 run_shard 2/4 "$pair" "$always" 2>&1 | reported /dev/stdin) 'agree, variant deleted'
# The slice that holds a variant alone measures it against its tree as pinned, which the other
# slice reports: the variant notes adds no finding to the slow tree's own, in 2/2.
sliced_slow="$(corpus "$work/slow-cut.toml" slow-bare:1)"
PARITY_WHOLE_COST=0 expect fail "a shard measures a variant against its tree as pinned, which it does not report" \
  "variant notes — bash reports no finding of docs/template-release beyond the tree's own" \
  run_shard 2/2 "$sliced_slow" "$always" "$work/notes.tsv"
PARITY_WHOLE_COST=0 run_shard 2/2 "$sliced_slow" "$always" "$work/notes.tsv" > "$work/notes-2-2.out" 2>&1
check "the shard that does not hold a tree as pinned leaves it out of its output" \
  lists <(reported "$work/notes-2-2.out") 'slow-bare, variant notes'
# The cut follows the cost: agree of cost 3 against 1 each for the others fills 1/2 alone.
dear="$(corpus "$work/dear.toml" agree:3 trace-fails:1 agree-too:1)"
for sh in 1/2 2/2; do PARITY_WHOLE_COST=0 slice "$work/dear-${sh/\//-}" "$sh" "$dear"; done
check "a dear tree fills the slice 1/2 alone" lists "$work/dear-1-2" agree 'agree, variant deleted'
check "the cheap trees fill the slice 2/2" lists "$work/dear-2-2" \
  trace-fails 'trace-fails, variant deleted' agree-too 'agree-too, variant deleted'
# A tree without a cost takes the largest of the corpus: trace-fails without one costs 2, as
# agree-too does, which cuts trace-fails; at a cost of 1, the cut would fall after it.
unpriced="$(corpus "$work/unpriced.toml" agree:1 trace-fails agree-too:2)"
for sh in 1/2 2/2; do PARITY_WHOLE_COST=0 slice "$work/unpriced-${sh/\//-}" "$sh" "$unpriced"; done
check "a tree without a cost takes the largest: 1/2 ends at trace-fails as pinned" \
  lists "$work/unpriced-1-2" agree 'agree, variant deleted' trace-fails
check "a tree without a cost takes the largest: 2/2 begins at its variant" \
  lists "$work/unpriced-2-2" 'trace-fails, variant deleted' agree-too 'agree-too, variant deleted'
for bad in 0 0.0 -1 abc '"2"' '1.'; do
  printf '[[repository]]\nname = "agree"\nurl = "file://%s"\ncommit = "%s"\ncost = %s\n' \
    "$work/src-agree" "$(cat "$work/src-agree.commit")" "$bad" > "$work/cost-bad.toml"
  check "a cost '$bad' exits 2, as a malformed input does" \
    exits_two run_shard 1/1 "$work/cost-bad.toml" "$(reactive_nuc)"
done
check "a cost of the whole-corpus parts that is none exits 2, as a malformed input does" \
  exits_two env PARITY_WHOLE_COST=x PARITY_TABLE="$work/one-rule.tsv" bash "$HARNESS" --no-hooks \
    --shard 1/2 --corpus "$uniform" --cache "$work/cache" --nuc "$(reactive_nuc)" --variants "$DEFAULT"
# The last slice ends with the cost of the whole-corpus parts: at 2, the six comparisons of cost
# 1 cut 4 to 2 instead of 3 to 3; at the default, far above them, the last shard compares none.
for sh in 1/2 2/2; do
  PARITY_WHOLE_COST=0 slice "$work/whole-0-${sh/\//-}" "$sh" "$uniform"
  PARITY_WHOLE_COST=2 slice "$work/whole-2-${sh/\//-}" "$sh" "$uniform"
  slice "$work/whole-default-${sh/\//-}" "$sh" "$uniform"
done
check "without a cost of the whole-corpus parts the slice 1/2 holds 3" lines_are "$work/whole-0-1-2" 3
check "without a cost of the whole-corpus parts the slice 2/2 holds 3" lines_are "$work/whole-0-2-2" 3
check "with a cost of 2 for the whole-corpus parts the last slice holds 2" \
  lists "$work/whole-2-2-2" agree-too 'agree-too, variant deleted'
check "with the default cost the last shard compares none of three cheap trees" \
  lists "$work/whole-default-2-2"
check "with the default cost the first shard compares all of them" lines_are "$work/whole-default-1-2" 6
# A tree the harness cannot fetch stays in the deal, so that it shifts no slice: on agree, gone,
# agree-too of cost 1, one variant each, three shards take a tree each, 2/3 gone and nothing else;
# every shard reports the failed fetch.
printf '[[repository]]\nname = "%s"\nurl = "file://%s"\ncommit = "%s"\ncost = 1\n' \
  gone "$work/src-none" "$(printf '0%.0s' {1..40})" > "$work/gone.entry"
corpus "$work/agree-only.toml" agree:1 > /dev/null
corpus "$work/agree-too-only.toml" agree-too:1 > /dev/null
cat "$work/agree-only.toml" "$work/gone.entry" "$work/agree-too-only.toml" > "$work/gone.toml"
for sh in 1/3 2/3 3/3; do PARITY_WHOLE_COST=0 slice "$work/gone-${sh/\//-}" "$sh" "$work/gone.toml"; done
check "a tree that cannot be fetched shifts no slice: 1/3 compares agree" \
  lists "$work/gone-1-3" agree 'agree, variant deleted'
check "a tree that cannot be fetched shifts no slice: 2/3 compares none" lists "$work/gone-2-3"
check "a tree that cannot be fetched shifts no slice: 3/3 compares agree-too" \
  lists "$work/gone-3-3" agree-too 'agree-too, variant deleted'
# all_name_gone <file>... — whether each output reports that gone could not be fetched.
# shellcheck disable=SC2317,SC2329 # run through check
all_name_gone() { local f; for f; do grep -q "gone — the cache could not be filled" "$f" || return 1; done; }
check "every shard reports the tree that cannot be fetched" all_name_gone "$work"/gone-[1-3]-3.out
# A variant that does not apply is reported by the shard that holds it alone: on agree and
# agree-too with the variants deleted and unfit, three shards cut the six comparisons 2, 2, 2, so
# 2/3 holds agree's unfit variant and 3/3 agree-too's.
UNFIT=$'unfit\tdocs/template-release\tNO-SUCH.md\tdelete\t-'
unfit="$(variants "$work/unfit.tsv" "$DELETE" "$UNFIT")"
for sh in 1/3 2/3 3/3; do PARITY_WHOLE_COST=0 slice "$work/unfit-${sh/\//-}" "$sh" "$pair" "$unfit"; done
# unfit_in <file> — the trees whose unfit variant the shard's output names, one per line.
unfit_in() { sed -nE 's/^Parity check: ([a-z-]+), variant unfit — its edit .*/\1/p' "$1"; }
check "a shard without an unfit variant reports none" lists <(unfit_in "$work/unfit-1-3.out")
check "the shard of agree's unfit variant reports it alone" lists <(unfit_in "$work/unfit-2-3.out") agree
check "the shard of agree-too's unfit variant reports it alone" \
  lists <(unfit_in "$work/unfit-3-3.out") agree-too
# A cost may be a decimal: agree of 0.5 and agree-too of 1.5 cut 3 to 1 at two shards.
decimal="$(corpus "$work/decimal.toml" agree:0.5 agree-too:1.5)"
expect pass "a decimal cost is read" "" run "$decimal" "$(reactive_nuc)"
PARITY_WHOLE_COST=0 slice "$work/decimal-1-2" 1/2 "$decimal"
check "a decimal cost deals as its value" lists "$work/decimal-1-2" agree 'agree, variant deleted' agree-too
for bad in -1 1.; do
  check "a cost of the whole-corpus parts '$bad' exits 2, as a malformed input does" \
    exits_two env PARITY_WHOLE_COST="$bad" PARITY_TABLE="$work/one-rule.tsv" bash "$HARNESS" --no-hooks \
      --shard 1/2 --corpus "$pair" --cache "$work/cache" --nuc "$(reactive_nuc)" --variants "$DEFAULT"
done
{ cat "$work/agree-only.toml"; echo 'cost = 2'; } > "$work/cost-twice.toml"
check "an entry with two costs exits 2, as a malformed input does" \
  exits_two run_shard 1/1 "$work/cost-twice.toml" "$(reactive_nuc)"
{ grep -v '^cost' "$work/agree-only.toml"; printf 'cost = -\ncost = 2\n'; } > "$work/cost-dash-twice.toml"
check "an entry with two costs exits 2 when the first reads '-'" \
  exits_two run_shard 1/1 "$work/cost-dash-twice.toml" "$(reactive_nuc)"
# A deal that does not come out whole fails rather than leave a shard nothing to compare: an awk
# that fails on the deal alone makes the one run exit 2.
mkdir -p "$work/broken-awk"
# shellcheck disable=SC2016 # the script is the stand-in's
printf '%s\n' '#!/usr/bin/env bash' 'for a; do [[ "$a" == whole=* ]] && exit 1; done' \
  "exec $(command -v awk) \"\$@\"" > "$work/broken-awk/awk"
chmod +x "$work/broken-awk/awk"
expect fail "a deal that does not come out whole fails" "the deal of the comparisons failed" \
  env PATH="$work/broken-awk:$PATH" PARITY_TABLE="$work/one-rule.tsv" bash "$HARNESS" --no-hooks \
    --corpus "$pair" --cache "$work/cache" --nuc "$(reactive_nuc)" --variants "$DEFAULT"
check "that run exits 2, as a malformed input does" \
  exits_two env PATH="$work/broken-awk:$PATH" PARITY_TABLE="$work/one-rule.tsv" bash "$HARNESS" \
    --no-hooks --corpus "$pair" --cache "$work/cache" --nuc "$(reactive_nuc)" --variants "$DEFAULT"
# A variant is measured only against a whole bash output of its tree as pinned: a tree whose
# check-traceability.sh fails without a finding unless notes.txt is there, and a variant that
# writes it and makes check-docs.sh report the release line. In 2/2, which does not report the tree
# as pinned, the variant agrees, and fires against the cut-short output, yet fails.
source_repo early-base "" '[[ -f notes.txt ]] || exit 1; '"$TRACE_PASSES" > /dev/null
d="$work/src-early-base"
f="$d/scripts/check-docs.sh"
sed 's/^if grep -q/if [[ ! -f notes.txt ]] \&\& grep -q/' "$f" > "$f.tmp" && cat "$f.tmp" > "$f" && rm "$f.tmp"
git -C "$d" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qam early
git -C "$d" rev-parse HEAD > "$work/src-early-base.commit"
noted="$work/nuc-noted"
# shellcheck disable=SC2016 # the script is the stand-in's
printf '%s\n' '#!/usr/bin/env bash' '[[ -f "$3/notes.txt" ]] || exit 0' \
  "echo $(printf %q "README.md: docs/template-release: $MISSING_TEXT [docs/template-release:line]"); exit 1" > "$noted"
chmod +x "$noted"
early="$(corpus "$work/early.toml" early-base:1)"
PARITY_WHOLE_COST=0 expect fail "a variant whose tree as pinned gave no whole bash output fails" \
  "variant notes — the tree as pinned gave no whole bash output to measure the variant against" \
  run_shard 2/2 "$early" "$noted" "$work/notes.tsv"
PARITY_WHOLE_COST=0 run_shard 2/2 "$early" "$noted" "$work/notes.tsv" > "$work/early-2-2.out" 2>&1
check "that variant agrees itself, so the cut-short tree as pinned alone fails it" \
  grep -q 'early-base at [0-9a-f]*, variant notes — 1 finding(s) of the ported rules, the same' "$work/early-2-2.out"
# The pins of the copied tables and the hook scenarios read all the template's trees, so they run
# in the last shard alone: a pin the tag does not bear out, and a corpus with no tree of the
# template for the hooks, fail 2/2 and pass 1/2, which compares the one tree.
# sharded <shard> <blobs> <corpus> [--no-hooks] — the harness on one shard with the pins given.
# shellcheck disable=SC2317,SC2329 # run through expect
sharded() {
  env PARITY_TABLE="$work/one-rule.tsv" PARITY_TABLE_BLOBS="$2" bash "$HARNESS" "${@:4}" \
    --shard "$1" --corpus "$3" --cache "$work/cache" --nuc "$(reactive_nuc)" --variants "$DEFAULT"
}
expect pass "a shard but the last holds no pin" "" sharded 1/2 "$work/blobs-other" "$tables" --no-hooks
expect fail "the last shard holds the pins" "is not the blob" \
  sharded 2/2 "$work/blobs-other" "$tables" --no-hooks
expect pass "a shard but the last runs no hook scenario" "" sharded 1/2 "$work/no-blobs" "$c"
expect fail "the last shard runs the hook scenarios" "the corpus pins no tree of the template" \
  sharded 2/2 "$work/no-blobs" "$c"
for bad in 0/2 3/2 2 x/y 1/0; do
  check "a shard '$bad' exits 2, as a malformed input does" \
    exits_two run_shard "$bad" "$many" "$(reactive_nuc)"
done
check "a number of jobs that is none exits 2, as a malformed input does" \
  exits_two env PARITY_JOBS=many PARITY_TABLE="$work/one-rule.tsv" bash "$HARNESS" --no-hooks \
    --corpus "$many" --cache "$work/cache" --nuc "$(reactive_nuc)" --variants "$DEFAULT"
printf 'v9.9.9 nohash t.txt\n' > "$work/blobs-malformed"
check "a malformed list of pins exits 2" exits_two pinned "$work/blobs-malformed" "$tables"
check "a missing list of pins exits 2" exits_two pinned "$work/no-such-blobs" "$tables"

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
  bash "$HARNESS" --no-hooks --corpus "$c" --cache "$work/cache" --nuc "$(reactive_nuc)" --variants ""
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

v="$(variants "$work/v-chomp.tsv" "$DELETE" $'chomped\tdocs/template-release\tREADME.md\tchomp\t-')"
expect fail "a variant that takes the last newline off applies, and is compared" "variant chomped — bash reports no finding" \
  run "$c" "$(reactive_nuc)" "$v"
source_repo unterminated > /dev/null
printf '%s' "no newline" > "$work/src-unterminated/NO-NEWLINE.md"
git -C "$work/src-unterminated" add -A
git -C "$work/src-unterminated" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qm unterminated
git -C "$work/src-unterminated" rev-parse HEAD > "$work/src-unterminated.commit"
v="$(variants "$work/v-chomp-none.tsv" "$DELETE" $'unchomped\tdocs/template-release\tNO-NEWLINE.md\tchomp\t-')"
expect fail "a chomp of a file without a last newline does not apply" "variant unchomped — its edit of NO-NEWLINE.md does not apply" \
  run "$(corpus "$work/unterminated.toml" unterminated)" "$(reactive_nuc)" "$v"

v="$(variants "$work/v-chomp-arg.tsv" "$DELETE" $'chomp-arg\tdocs/template-release\tREADME.md\tchomp\tx')"
expect fail "a chomp with an argument other than '-' is refused" "is not 'name<TAB>rule" run "$c" "$(reactive_nuc)" "$v"

v="$(variants "$work/v-chmod.tsv" "$DELETE" $'chmodded\tdocs/template-release\tREADME.md\tchmod\t+x')"
expect fail "a variant that sets the executable bit in the index applies, and is compared" "variant chmodded — bash reports no finding" \
  run "$c" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-chmod-same.tsv" "$DELETE" $'unchanged\tdocs/template-release\tREADME.md\tchmod\t-x')"
expect fail "a chmod that leaves the mode in the index as it was does not apply" "variant unchanged — its edit of README.md does not apply" \
  run "$c" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-chmod-untracked.tsv" "$DELETE" $'untracked\tdocs/template-release\tNEW.md\tchmod\t+x')"
expect fail "a chmod of a file git does not track does not apply" "variant untracked — its edit of NEW.md does not apply" \
  run "$c" "$(reactive_nuc)" "$v"
v="$(variants "$work/v-chmod-arg.tsv" "$DELETE" $'chmod-arg\tdocs/template-release\tREADME.md\tchmod\t755')"
expect fail "a chmod with an argument other than +x or -x is refused" "is not 'name<TAB>rule" run "$c" "$(reactive_nuc)" "$v"

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

# --- cases: the releases at which a rule's bash check changed ------------------------------

# template_repo <name> <release> <statement> — a tree of the template, carrying TEMPLATE-SETUP.md,
# on the release, whose check-docs.sh defines check_template_release with the statement.
template_repo() {
  source_repo "$1" "- **Template release:** $2" > /dev/null
  printf '# Setup\n' > "$work/src-$1/TEMPLATE-SETUP.md"
  printf '%s\n' '# The function of check 14.' 'check_template_release() {' "  $3" '}' \
    >> "$work/src-$1/scripts/check-docs.sh"
  git -C "$work/src-$1" add -A
  git -C "$work/src-$1" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qm template
  git -C "$work/src-$1" rev-parse HEAD > "$work/src-$1.commit"
}

# releases <file> <changed> — a table of releases with the one row of docs/template-release.
releases() {
  printf '# One rule.\ndocs/template-release\tscripts/check-docs.sh\tcheck_template_release\t%s\n' "$2" > "$1"
  echo "$1"
}

# run_releases <corpus> <table> — the harness on the corpus with the table of releases given.
# shellcheck disable=SC2317,SC2329 # run through expect
run_releases() {
  env PARITY_TABLE="$work/one-rule.tsv" PARITY_RELEASES="$2" bash "$HARNESS" --no-hooks --corpus "$1" \
    --cache "$work/cache" --nuc "$(reactive_nuc)" --variants "$DEFAULT"
}

template_repo tpl-old v0.5.0 ': one'
template_repo tpl-new v0.6.0 ': two'
template_repo tpl-same v0.6.0 ': one'
changed="$(corpus "$work/tpl-changed.toml" tpl-new tpl-old)"
expect pass "a change the table lists, at the later of two trees of the template, passes" "" \
  run_releases "$changed" "$(releases "$work/r-listed.tsv" v0.6.0)"
expect pass "a row with the releases its reason covers and the reason is read for its first four fields" "" \
  run_releases "$changed" "$(releases "$work/r-reasoned.tsv" $'v0.6.0\tv0.6.0\tthe reason')"
expect fail "a change the table does not list fails and names the rule" \
  "docs/template-release — its bash check changed at v0.6.0 across the template's trees of the corpus" \
  run_releases "$changed" "$(releases "$work/r-unlisted.tsv" -)"
# The releases are compared in the last shard alone, which has every tree of the template.
# run_releases_shard <shard> <corpus> <table> — run_releases on one shard.
# shellcheck disable=SC2317,SC2329 # run through expect
run_releases_shard() {
  env PARITY_TABLE="$work/one-rule.tsv" PARITY_RELEASES="$3" bash "$HARNESS" --no-hooks \
    --shard "$1" --corpus "$2" --cache "$work/cache" --nuc "$(reactive_nuc)" --variants "$DEFAULT"
}
expect pass "a shard but the last compares no releases" "" \
  run_releases_shard 1/2 "$changed" "$(releases "$work/r-unlisted-1.tsv" -)"
expect fail "the last shard compares the releases of every tree of the template" \
  "docs/template-release — its bash check changed at v0.6.0" \
  run_releases_shard 2/2 "$changed" "$(releases "$work/r-unlisted-2.tsv" -)"
expect fail "a change the table lists that the trees do not show fails" "changed at - across" \
  run_releases "$(corpus "$work/tpl-same.toml" tpl-old tpl-same)" "$(releases "$work/r-phantom.tsv" v0.6.0)"
stub="$work/src-tpl-same/scripts/check-docs.sh"
sed 's/^# The function of check 14\.$/# Check 14, reworded./' "$stub" > "$stub.tmp" && cat "$stub.tmp" > "$stub" && rm "$stub.tmp"
git -C "$work/src-tpl-same" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qam reworded
git -C "$work/src-tpl-same" rev-parse HEAD > "$work/src-tpl-same.commit"
expect pass "a change of its comments alone is no change" "" \
  run_releases "$(corpus "$work/tpl-reworded.toml" tpl-old tpl-same)" "$(releases "$work/r-reworded.tsv" -)"
# From v0.9.0 on the head of a function ends in a comment that names its check.
source_repo tpl-headed "- **Template release:** v0.6.0" > /dev/null
printf '# Setup\n' > "$work/src-tpl-headed/TEMPLATE-SETUP.md"
printf '%s\n' 'check_template_release() {  # check 14' '  : one' '}' >> "$work/src-tpl-headed/scripts/check-docs.sh"
git -C "$work/src-tpl-headed" add -A
git -C "$work/src-tpl-headed" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qm headed
git -C "$work/src-tpl-headed" rev-parse HEAD > "$work/src-tpl-headed.commit"
expect pass "a comment after the head of a function is no change" "" \
  run_releases "$(corpus "$work/tpl-headed.toml" tpl-old tpl-headed)" "$(releases "$work/r-headed.tsv" -)"
source_repo tpl-headed-two "- **Template release:** v0.6.0" > /dev/null
printf '# Setup\n' > "$work/src-tpl-headed-two/TEMPLATE-SETUP.md"
printf '%s\n' 'check_template_release() {  # check 14' '  : two' '}' >> "$work/src-tpl-headed-two/scripts/check-docs.sh"
git -C "$work/src-tpl-headed-two" add -A
git -C "$work/src-tpl-headed-two" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qm headed
git -C "$work/src-tpl-headed-two" rev-parse HEAD > "$work/src-tpl-headed-two.commit"
expect fail "the body after a commented head is compared" "changed at v0.6.0 across" \
  run_releases "$(corpus "$work/tpl-headed-two.toml" tpl-old tpl-headed-two)" "$(releases "$work/r-headed-two.tsv" -)"
expect pass "a corpus with one tree of the template compares nothing" "" \
  run_releases "$(corpus "$work/tpl-one.toml" tpl-old)" "$(releases "$work/r-one.tsv" v0.6.0)"
source_repo tpl-bare "- **Template release:** v0.6.0" > /dev/null
printf '# Setup\n' > "$work/src-tpl-bare/TEMPLATE-SETUP.md"
git -C "$work/src-tpl-bare" add -A
git -C "$work/src-tpl-bare" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qm bare
git -C "$work/src-tpl-bare" rev-parse HEAD > "$work/src-tpl-bare.commit"
expect pass "a tree of the template without the function has no such check, a change at its release" "" \
  run_releases "$(corpus "$work/tpl-bare.toml" tpl-old tpl-bare)" "$(releases "$work/r-bare.tsv" v0.6.0)"
expect fail "a check gone at a release the table does not list fails" "changed at v0.6.0 across" \
  run_releases "$(corpus "$work/tpl-bare.toml" tpl-old tpl-bare)" "$(releases "$work/r-bare-unlisted.tsv" -)"
printf '# One rule.\ndocs/template-release\tscripts/check-docs.sh\tcheck_nowhere\t-\n' > "$work/r-nowhere.tsv"
expect fail "a function no tree of the template has fails, as a mistyped one would" \
  "no tree of the template in the corpus has check_nowhere" run_releases "$changed" "$work/r-nowhere.tsv"
printf '# One rule.\ndocs/template-release\tscripts/check-docs.sh\tcheck_nowhere\tv9.0.0\n' > "$work/r-ahead.tsv"
expect pass "a function no tree has, whose check comes at a release ahead of the corpus, passes" "" \
  run_releases "$changed" "$work/r-ahead.tsv"
expect pass "a change at a release ahead of the corpus is not compared" "" \
  run_releases "$changed" "$(releases "$work/r-listed-ahead.tsv" v0.6.0,v9.0.0)"
# A sensor's row holds the libraries its script sources (API-sensor-cores-ask-the-shell).
# sensor_repo <name> <release> <lines of git.sh>... — a tree of the template whose
# scripts/sensor-x.sh sources scripts/lib/git.sh, which holds the lines.
sensor_repo() {
  local name="$1" release="$2"
  shift 2
  source_repo "$name" "- **Template release:** $release" > /dev/null
  printf '# Setup\n' > "$work/src-$name/TEMPLATE-SETUP.md"
  mkdir -p "$work/src-$name/scripts/lib"
  # shellcheck disable=SC2016 # the script is the tree's
  printf '%s\n' '#!/usr/bin/env bash' '# shellcheck source=lib/git.sh' \
    'source "$(dirname "${BASH_SOURCE[0]}")/lib/git.sh"' 'echo x' > "$work/src-$name/scripts/sensor-x.sh"
  printf '%s\n' "$@" > "$work/src-$name/scripts/lib/git.sh"
  git -C "$work/src-$name" add -A
  git -C "$work/src-$name" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -qm sensor
  git -C "$work/src-$name" rev-parse HEAD > "$work/src-$name.commit"
}
# sensor_releases <file> <rule> <changed> — a table of releases with the one row of the sensor.
sensor_releases() {
  printf '# One sensor.\n%s\tscripts/sensor-x.sh\t-\t%s\n' "$2" "$3" > "$1"
  echo "$1"
}
sensor_repo sens-old v0.5.0 ': one'
sensor_repo sens-new v0.6.0 ': two'
sensor_repo sens-reworded v0.6.0 '# Reworded.' ': one'
check "the library of the changed tree differs in a line that is no comment" \
  grep -qx ': two' "$work/src-sens-new/scripts/lib/git.sh"
check "the library of the reworded tree differs from the old one in a comment" \
  grep -qx '# Reworded.' "$work/src-sens-reworded/scripts/lib/git.sh"
check "the old tree's library has no such comment" \
  test "$(cat "$work/src-sens-old/scripts/lib/git.sh")" = ': one'
check "the sensor's script is the same in both trees" \
  cmp -s "$work/src-sens-old/scripts/sensor-x.sh" "$work/src-sens-new/scripts/sensor-x.sh"
lib_changed="$(corpus "$work/sens-changed.toml" sens-old sens-new)"
expect fail "a library a sensor sources that changed at a release its row lacks fails and names the library" \
  "r-lib-unlisted.tsv says -; at v0.6.0 in scripts/lib/git.sh" \
  run_releases "$lib_changed" "$(sensor_releases "$work/r-lib-unlisted.tsv" sense/x -)"
expect pass "a change of a library a sensor's row lists passes" "" \
  run_releases "$lib_changed" "$(sensor_releases "$work/r-lib-listed.tsv" sense/x v0.6.0)"
expect pass "a change of a library's comments alone is no change" "" \
  run_releases "$(corpus "$work/sens-reworded.toml" sens-old sens-reworded)" \
  "$(sensor_releases "$work/r-lib-reworded.tsv" sense/x -)"
expect pass "the libraries count for a sensor's row alone" "" \
  run_releases "$lib_changed" "$(sensor_releases "$work/r-lib-rule.tsv" docs/x -)"
printf 'docs/template-release\tREADME.md\tcheck_template_release\t-\n' > "$work/r-malformed.tsv"
expect fail "a malformed row of the table of releases is refused" "a row needs a rule ID" \
  run_releases "$c" "$work/r-malformed.tsv"

# --- cases: the hook scenarios -------------------------------------------------------------

ON_MAIN="pre-commit: refused — the checkout is on 'main', and changes reach main through a pull request (AGENTS.md, section 6). Create a branch first: git switch -c <name>"

# hook_tree <name> <extra hook line> [extra case] — a tree of the template at v0.7.0, or at
# HOOK_TREE_RELEASE, whose bash
# pre-commit refuses a commit on main, writing the extra line first where one is given, and whose
# self-test names the cases of the scenarios and the extra case. Printed as its path.
# shellcheck disable=SC2016 # the hook's and the self-test's own expansions
hook_tree() {
  local dir="$work/hooks-$1"
  mkdir -p "$dir/.githooks" "$dir/scripts"
  printf '# Template\n\n- **Template release:** %s\n' "${HOOK_TREE_RELEASE:-v0.7.0}" > "$dir/README.md"
  {
    echo '#!/usr/bin/env bash'
    [[ -n "$2" ]] && printf 'echo %q >&2\n' "$2"
    echo '[[ "$(git symbolic-ref --short -q HEAD)" == main ]] || exit 0'
    printf 'echo %q >&2\nexit 1\n' "$ON_MAIN"
  } > "$dir/.githooks/pre-commit"
  chmod +x "$dir/.githooks/pre-commit"
  printf 'expect refuse "a commit on main" "$d" "on main" commit -q --allow-empty -m x\n' > "$dir/scripts/test-git-hooks.sh"
  printf 'expect pass "a commit on a branch" "$d" "" commit -q --allow-empty -m x\n' >> "$dir/scripts/test-git-hooks.sh"
  [[ -n "${3:-}" ]] && printf 'expect pass "%s" "$d" "" commit -q --allow-empty -m x\n' "$3" >> "$dir/scripts/test-git-hooks.sh"
  echo "$dir"
}

# hook_nuc <name> <shell code> — a stand-in nuc whose 'hook pre-commit' runs the code, printed as
# its path; HOOK_AGREES is the code that answers as the bash does.
# shellcheck disable=SC2016 # the stand-in's own expansion
HOOK_AGREES='[ "$(git symbolic-ref --short -q HEAD)" = main ] || exit 0; echo "refs/heads/main: hook/commit-on-main: the checkout is on '"'"'main'"'"', and changes reach main through a pull request (AGENTS.md, section 6). Create a branch first: git switch -c <name> [hook/commit-on-main:main]"; exit 1'
hook_nuc() {
  local f="$work/hook-nuc-$1"
  printf '#!/bin/sh\n%s\n' "$2" > "$f"
  chmod +x "$f"
  echo "$f"
}

HOOK_SCENARIOS_FILE="$work/hook-scenarios.sh"
# shellcheck disable=SC2016 # the scenario file's own expansion
printf '%s\n' 'on_branch() { hp_git "$1" switch -q -c feature; }' \
  'hook_scenario on-main pre-commit "a commit on main" : commit -q --allow-empty -m x' \
  'hook_scenario on-branch pre-commit "a commit on a branch" on_branch commit -q --allow-empty -m x' \
  > "$HOOK_SCENARIOS_FILE"
NO_CASES="$work/hook-cases.tsv"
echo '# None.' > "$NO_CASES"

# run_hooks <nuc> <scenarios> <cases> <tree>... — the hook scenarios alone, on the trees.
# shellcheck disable=SC2317,SC2329 # run through expect
run_hooks() {
  local nuc="$1" scenarios="$2" cases="$3"; shift 3
  env PARITY_HOOK_SCENARIOS="$scenarios" PARITY_HOOK_CASES="$cases" \
    bash "$HARNESS" --nuc "$nuc" --hooks "$@"
}

tree="$(hook_tree plain "")"
expect pass "hook scenarios that agree pass" "" \
  run_hooks "$(hook_nuc agrees "$HOOK_AGREES")" "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$tree"
expect fail "a nuc whose exit status differs in one scenario fails" "bash exits 1, nuc exits 0" \
  run_hooks "$(hook_nuc passes 'exit 0')" "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$tree"
# A tree later than the release whose hooks nuc ports is left out, its difference not compared;
# with no tree left, nothing is, which fails.
out="$(env PARITY_HOOKS_PORTED=v0.6.0 PARITY_HOOK_SCENARIOS="$HOOK_SCENARIOS_FILE" PARITY_HOOK_CASES="$NO_CASES" \
  bash "$HARNESS" --nuc "$(hook_nuc passes 'exit 0')" --hooks "$tree" 2>&1)"; rc=$?
if ((rc == 1)) && [[ "$out" == *"the tree at v0.7.0 is later than v0.6.0, whose hooks nuc ports"* \
  && "$out" == *"every tree of the template is later than v0.6.0"* && "$out" != *"nuc exits"* ]]; then
  passed=$((passed + 1))
else
  failed=$((failed + 1))
  echo "FAIL: a tree later than the hooks nuc ports is left out (exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
fi
# A tree whose blob list pins its hooks carries nuc's shims: it is left out of the scenarios and of
# the coverage whatever release it names, so neither its hook's line no row maps nor the case only
# its self-test names fails; with no other tree, nothing is compared, which fails.
shim="$(hook_tree shim "pre-commit: a line no row maps" "a case of the shims")"
printf 'release v0.2.2\n%s pre-commit\n' "$(printf '0%.0s' {1..40})" > "$shim/.githooks/nuc.blobs"
out="$(run_hooks "$(hook_nuc agrees "$HOOK_AGREES")" "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$tree" "$shim" 2>&1)"; rc=$?
out_alone="$(run_hooks "$(hook_nuc passes 'exit 0')" "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$shim" 2>&1)"; rc_alone=$?
if ((rc == 0 && rc_alone == 1)) \
  && [[ "$out" == *"the tree at v0.7.0 pins its hooks in .githooks/nuc.blobs"* && "$out" == *"on 1 tree(s)"* \
  && "$out_alone" == *"or pins nuc's shims as its hooks, so no hook is compared"* && "$out_alone" != *"nuc exits"* ]]; then
  passed=$((passed + 1))
else
  failed=$((failed + 1))
  echo "FAIL: a tree whose blob list pins its hooks is left out (exit $rc, alone $rc_alone)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out"$'\n'"$out_alone" >&2
fi
# The bound of the hooks nuc ports is v0.11.1 unless PARITY_HOOKS_PORTED names another: a tree at
# v0.11.1 is compared, one at v0.11.2 left out.
at_bound="$(HOOK_TREE_RELEASE=v0.11.1 hook_tree bound "")"
past_bound="$(HOOK_TREE_RELEASE=v0.11.2 hook_tree past "")"
out="$(run_hooks "$(hook_nuc agrees "$HOOK_AGREES")" "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$at_bound" "$past_bound" 2>&1)"; rc=$?
if ((rc == 0)) && [[ "$out" == *"the tree at v0.11.2 is later than v0.11.1, whose hooks nuc ports"* \
  && "$out" == *"on 1 tree(s)"* ]]; then
  passed=$((passed + 1))
else
  failed=$((failed + 1))
  echo "FAIL: the default bound of the hooks nuc ports is v0.11.1 (exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
fi
# The scenario of the defaults, without README and nuc.toml, runs on the newest tree left in, not
# on a newer one left out by the bound or by its pinned hooks: a nuc that differs there alone fails
# on the tree of v0.7.0.
cp "$HOOK_SCENARIOS_FILE" "$work/hook-scenarios-bare.sh"
echo 'hook_scenario --bare bare-main pre-commit "a commit on main" : commit -q --allow-empty -m x' >> "$work/hook-scenarios-bare.sh"
newer="$(HOOK_TREE_RELEASE=v0.8.0 hook_tree newer "")"
pinned_newer="$(HOOK_TREE_RELEASE=v0.9.0 hook_tree pinned-newer "")"
printf 'release v0.2.2\n%s pre-commit\n' "$(printf '0%.0s' {1..40})" > "$pinned_newer/.githooks/nuc.blobs"
out="$(env PARITY_HOOKS_PORTED=v0.8.0 PARITY_HOOK_SCENARIOS="$work/hook-scenarios-bare.sh" PARITY_HOOK_CASES="$NO_CASES" \
  bash "$HARNESS" --nuc "$(hook_nuc bare-differs "[ -f README.md ] || exit 0; $HOOK_AGREES")" --hooks "$tree" "$newer" "$pinned_newer" 2>&1)"; rc=$?
out_newest="$(env PARITY_HOOKS_PORTED=v0.7.0 PARITY_HOOK_SCENARIOS="$work/hook-scenarios-bare.sh" PARITY_HOOK_CASES="$NO_CASES" \
  bash "$HARNESS" --nuc "$(hook_nuc bare-differs "[ -f README.md ] || exit 0; $HOOK_AGREES")" --hooks "$tree" "$newer" "$pinned_newer" 2>&1)"; rc_newest=$?
if ((rc == 1 && rc_newest == 1)) && [[ "$out" == *"bare-main at v0.8.0"* && "$out" != *"bare-main at v0.7.0"* \
  && "$out_newest" == *"bare-main at v0.7.0"* && "$out_newest" == *"the tree at v0.8.0 is later than v0.7.0"* \
  && "$out_newest" == *"the tree at v0.9.0 pins its hooks"* ]]; then
  passed=$((passed + 1))
else
  failed=$((failed + 1))
  echo "FAIL: the scenario of the defaults runs on the newest tree left in (exit $rc, $rc_newest)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out"$'\n'"$out_newest" >&2
fi
expect pass "a tree at the bound of the hooks nuc ports is compared" "" \
  env PARITY_HOOKS_PORTED=v0.7.0 PARITY_HOOK_SCENARIOS="$HOOK_SCENARIOS_FILE" PARITY_HOOK_CASES="$NO_CASES" \
  bash "$HARNESS" --nuc "$(hook_nuc agrees "$HOOK_AGREES")" --hooks "$tree"
# shellcheck disable=SC2016 # the stand-in's own expansion
expect fail "a refusal mapped to another rule ID fails" "only nuc: rule hook/merge-checks refs/heads/main" \
  run_hooks "$(hook_nuc other-rule '[ "$(git symbolic-ref --short -q HEAD)" = main ] || exit 0; echo "refs/heads/main: hook/merge-checks: x [hook/merge-checks:main]"; exit 1')" \
  "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$tree"
expect fail "a refusal that cites another release than the tree carries fails" \
  "only nuc: rule hook/commit-on-main refs/heads/main NUC-main-only-via-pr §1, §4" \
  run_hooks "$(hook_nuc cites "${HOOK_AGREES/AGENTS.md, section 6/NUC-main-only-via-pr §1, §4}")" \
  "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$tree"
# shellcheck disable=SC2016 # the stand-in's own expansion
expect fail "a refusal without its key fails" "nuc refuses with hook/commit-on-main without its key" \
  run_hooks "$(hook_nuc unkeyed '[ "$(git symbolic-ref --short -q HEAD)" = main ] || exit 0; echo "refs/heads/main: hook/commit-on-main: on main"; exit 1')" \
  "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$tree"
expect fail "a nuc that agrees without nuc.toml and differs with it fails" "on-main at v0.7.0, with nuc.toml" \
  run_hooks "$(hook_nuc configured "[ -f nuc.toml ] && exit 0; $HOOK_AGREES")" "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$tree"
expect fail "a line of a bash hook the table does not know fails" "a line of the hook the table does not know: pre-commit: a notice nobody maps" \
  run_hooks "$(hook_nuc agrees "$HOOK_AGREES")" "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$(hook_tree unknown "pre-commit: a notice nobody maps")"
expect fail "a self-test case no scenario covers fails" "the self-test case 'a case nobody covers' is covered by no scenario" \
  run_hooks "$(hook_nuc agrees "$HOOK_AGREES")" "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$(hook_tree uncovered "" "a case nobody covers")"
printf 'a case no self-test has\tno case of a hook: a test of the harness\n' > "$work/hook-cases-stale.tsv"
expect fail "a listed case the self-test lacks fails" "lists the case 'a case no self-test has', which no tree's self-test has" \
  run_hooks "$(hook_nuc agrees "$HOOK_AGREES")" "$HOOK_SCENARIOS_FILE" "$work/hook-cases-stale.tsv" "$tree"
cp "$HOOK_SCENARIOS_FILE" "$work/hook-scenarios-stale.sh"
echo 'hook_scenario stale pre-commit "a case of no release" : commit -q --allow-empty -m x' >> "$work/hook-scenarios-stale.sh"
expect fail "a scenario that names a case no release has fails" "it names the case 'a case of no release', which no tree's self-test has" \
  run_hooks "$(hook_nuc agrees "$HOOK_AGREES")" "$work/hook-scenarios-stale.sh" "$NO_CASES" "$tree"
cp "$HOOK_SCENARIOS_FILE" "$work/hook-scenarios-unreached.sh"
echo 'hook_scenario unreached pre-commit "a commit on main" on_branch commit -q --allow-empty -m x' >> "$work/hook-scenarios-unreached.sh"
expect fail "a scenario whose bash side does not end as its case expects fails" "the scenario does not reach its case" \
  run_hooks "$(hook_nuc agrees "$HOOK_AGREES")" "$work/hook-scenarios-unreached.sh" "$NO_CASES" "$tree"
# A case is held on the trees whose self-test names it: the scenario that does not reach the case
# of the tree of v0.8.0 fails there, and is compared on the tree of v0.7.0, which lacks the case.
later="$(HOOK_TREE_RELEASE=v0.8.0 hook_tree later "" "a case of the later tree")"
cp "$HOOK_SCENARIOS_FILE" "$work/hook-scenarios-later.sh"
echo 'hook_scenario elsewhere pre-commit "a case of the later tree" : commit -q --allow-empty -m x' >> "$work/hook-scenarios-later.sh"
out="$(run_hooks "$(hook_nuc agrees "$HOOK_AGREES")" "$work/hook-scenarios-later.sh" "$NO_CASES" "$tree" "$later" 2>&1)"; rc=$?
if ((rc == 1)) && [[ "$out" == *"elsewhere at v0.8.0 — the bash side exits 1"* \
  && "$out" != *"elsewhere at v0.7.0 — the bash side"* ]]; then
  passed=$((passed + 1))
else
  failed=$((failed + 1))
  echo "FAIL: a case is held only on the trees whose self-test names it (exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
fi
printf '%s\t%s\t%s\n' "^pre-commit: notice (.+)$" notice/test '\1' > "$work/hooks-notice.tsv"
cat "$ROOT/tests/parity/hooks.tsv" >> "$work/hooks-notice.tsv"
expect fail "a notice whose field differs fails" "only nuc: rule notice/test B" \
  env PARITY_HOOK_TABLE="$work/hooks-notice.tsv" PARITY_HOOK_SCENARIOS="$HOOK_SCENARIOS_FILE" PARITY_HOOK_CASES="$NO_CASES" \
  bash "$HARNESS" --nuc "$(hook_nuc notice "echo 'pre-commit: notice B' >&2; $HOOK_AGREES")" --hooks "$(hook_tree notice "pre-commit: notice A")"
expect fail "a record of green trees that differs fails" "only nuc: record x" \
  run_hooks "$(hook_nuc record "echo x > .git/checks-green-trees; $HOOK_AGREES")" "$HOOK_SCENARIOS_FILE" "$NO_CASES" "$tree"
# A tree whose bash merge driver takes theirs, and a stand-in nuc whose driver takes the base.
dtree="$(hook_tree driver "" "the driver merges")"
# shellcheck disable=SC2016 # the driver's own expansion
printf '#!/usr/bin/env bash\ncat "$3" > "$2"\n' > "$dtree/.githooks/merge-adr-index"
chmod +x "$dtree/.githooks/merge-adr-index"
# shellcheck disable=SC2016 # the scenario file's own expansion
printf '%s\n' 'driven() {' \
  '  echo "docs/adr/README.md merge=adr-index" > "$1/.gitattributes"; mkdir -p "$1/docs/adr"' \
  '  echo base > "$1/docs/adr/README.md"; hp_git "$1" add -A; hp_git "$1" commit -q -m base' \
  '  hp_git "$1" switch -q -c other; echo theirs > "$1/docs/adr/README.md"; hp_git "$1" commit -q -am t' \
  '  hp_git "$1" switch -q -c feat/x main; echo ours > "$1/docs/adr/README.md"; hp_git "$1" commit -q -am o' \
  '}' \
  'hook_scenario driven merge-adr-index "the driver merges" driven merge -q -m m other' > "$work/hook-scenarios-driver.sh"
cat "$HOOK_SCENARIOS_FILE" >> "$work/hook-scenarios-driver.sh"
# shellcheck disable=SC2016 # the stand-in's own expansion
expect pass "a merge driver that writes the same index passes" "" \
  run_hooks "$(hook_nuc driver-same '[ "$1" = hook ] && [ "$2" = merge-adr-index ] && { cat "$5" > "$4"; exit 0; }; '"$HOOK_AGREES")" \
  "$work/hook-scenarios-driver.sh" "$NO_CASES" "$dtree"
# shellcheck disable=SC2016 # the stand-in's own expansion
expect fail "a merge driver that writes another index fails" "the ADR index differs after the merge" \
  run_hooks "$(hook_nuc driver-other '[ "$1" = hook ] && [ "$2" = merge-adr-index ] && { cat "$3" > "$4"; exit 0; }; '"$HOOK_AGREES")" \
  "$work/hook-scenarios-driver.sh" "$NO_CASES" "$dtree"
expect fail "hook scenarios without a tree of the template fail" "the corpus pins no tree of the template" \
  run_hooks "$(hook_nuc agrees "$HOOK_AGREES")" "$HOOK_SCENARIOS_FILE" "$NO_CASES"
printf 'pattern\tdocs/x\t-\n' > "$work/hooks-bad.tsv"
expect fail "a row of the hook table that names no hook/ rule is refused" "is not 'pattern<TAB>hook/<rule>|notice/<name><TAB>fields|-'" \
  env PARITY_HOOK_TABLE="$work/hooks-bad.tsv" PARITY_HOOK_SCENARIOS="$HOOK_SCENARIOS_FILE" PARITY_HOOK_CASES="$NO_CASES" \
  bash "$HARNESS" --nuc "$(hook_nuc agrees "$HOOK_AGREES")" --hooks "$tree"

# --- cases: the replay of the take-ups --------------------------------------------------------

# A template with the tags v0.9.0 and v0.10.0, and a project whose take-up of v0.10.0 is the commit
# after the one it starts from; the stand-ins of nuc write as each case needs, in the directory the
# replay runs them in (API-upgrade-by-blob-toml-optional).
tu_git() { git -c core.hooksPath=/dev/null -c user.name=t -c user.email=t@x.invalid -C "$@"; }
tu_template="$work/tu-template" tu_project="$work/tu-project"
mkdir -p "$tu_template" "$tu_project"
tu_git "$tu_template" init -q -b main
printf -- '- **Template release:** v0.9.0\n' > "$tu_template/README.md"
echo old > "$tu_template/a.txt"
tu_git "$tu_template" add -A && tu_git "$tu_template" commit -q -m v0.9.0
tu_git "$tu_template" tag -a -m v0.9.0 v0.9.0
echo new > "$tu_template/a.txt"
tu_git "$tu_template" commit -q -am v0.10.0
tu_git "$tu_template" tag -a -m v0.10.0 v0.10.0
tu_git "$tu_project" init -q -b main
printf -- '- **Template release:** v0.9.0\n' > "$tu_project/README.md"
echo old > "$tu_project/a.txt"
echo '*.log' > "$tu_project/.gitignore"
tu_git "$tu_project" add -A && tu_git "$tu_project" commit -q -m start
tu_from="$(tu_git "$tu_project" rev-parse HEAD)"
printf -- '- **Template release:** v0.10.0\n' > "$tu_project/README.md"
echo new > "$tu_project/a.txt"
tu_git "$tu_project" commit -q -am take-up
tu_to="$(tu_git "$tu_project" rev-parse HEAD)"
printf 'file://%s\t%s\t%s\tfile://%s\n' "$tu_project" "$tu_from" "$tu_to" "$tu_template" > "$work/take-ups.tsv"
echo '# None.' > "$work/take-up-none.tsv"
printf 'v0.10.0\tb.txt\tthe template only\n' > "$work/take-up-b.tsv"
printf 'file://%s\t%s\t%s\tfile://%s\n' "$work/tu-gone" "$tu_from" "$tu_to" "$tu_template" > "$work/take-ups-gone.tsv"

# tu_nuc <name> <body> — a stand-in of nuc whose run of 'upgrade' is the body.
tu_nuc() { hook_nuc "take-up-$1" "$2"; }
TU_AGREES='printf -- "- **Template release:** v0.10.0\n" > README.md; echo new > a.txt'

# run_take_ups <nuc> <differences> — the replay alone, on the table of the one take-up.
# shellcheck disable=SC2317,SC2329 # run through expect
run_take_ups() {
  env PARITY_TAKE_UPS="$work/take-ups.tsv" PARITY_TAKE_UP_DIFFERENCES="$2" \
    bash "$HARNESS" --cache "$work/tu-cache" --nuc "$1" --take-ups
}

expect pass "a replay that writes what the take-up holds passes" "" \
  run_take_ups "$(tu_nuc agrees "$TU_AGREES")" "$work/take-up-none.tsv"
expect fail "a replay that writes a path the human left alone fails and names it" \
  "b.txt (??) is written where the take-up has none" \
  run_take_ups "$(tu_nuc more "$TU_AGREES; echo extra > b.txt")" "$work/take-up-none.tsv"
expect pass "a path the table of differences lists for the release passes" "" \
  run_take_ups "$(tu_nuc listed "$TU_AGREES; echo extra > b.txt")" "$work/take-up-b.tsv"
expect fail "a row of the table of differences no replay meets fails" "is listed, and no replay writes it" \
  run_take_ups "$(tu_nuc agrees "$TU_AGREES")" "$work/take-up-b.tsv"
expect fail "a replay that writes other bytes fails" "a.txt ( M) holds other bytes than the take-up" \
  run_take_ups "$(tu_nuc other 'printf -- "- **Template release:** v0.10.0\n" > README.md; echo other > a.txt')" \
  "$work/take-up-none.tsv"
expect pass "a path nuc lists as a judgment is not compared" "" \
  run_take_ups "$(tu_nuc judged "$TU_AGREES; echo mine > c.txt; echo 'c.txt: upgrade/file: merge it [upgrade/file:both]'; exit 1")" \
  "$work/take-up-none.tsv"
expect fail "a replay where nuc upgrade cannot decide fails" "nuc upgrade exits 2" \
  run_take_ups "$(tu_nuc refuses 'echo refused >&2; exit 2')" "$work/take-up-none.tsv"
printf 'file://%s\t%s\n' "$tu_project" "$tu_from" > "$work/take-ups-bad.tsv"
expect fail "a row of the table of take-ups without its four fields is refused" \
  "is not '<url><TAB><commit><TAB><commit><TAB><template url>'" \
  env PARITY_TAKE_UPS="$work/take-ups-bad.tsv" PARITY_TAKE_UP_DIFFERENCES="$work/take-up-none.tsv" \
  bash "$HARNESS" --cache "$work/tu-cache" --nuc "$(tu_nuc agrees "$TU_AGREES")" --take-ups

# What the replay cannot reach or set up fails, and names it, rather than skipping the take-up.
expect fail "a take-up whose commits cannot be fetched fails" "its commits cannot be fetched from" \
  env PARITY_TAKE_UPS="$work/take-ups-gone.tsv" PARITY_TAKE_UP_DIFFERENCES="$work/take-up-none.tsv" \
  bash "$HARNESS" --cache "$work/tu-cache" --nuc "$(tu_nuc agrees "$TU_AGREES")" --take-ups
printf 'file://%s\t%s\t%s\tfile://%s\n' "$tu_project" "$tu_from" "$tu_to" "$work/tu-no-template" > "$work/take-ups-no-tags.tsv"
expect fail "a take-up whose tags cannot be fetched fails" "the tags cannot be fetched from" \
  env PARITY_TAKE_UPS="$work/take-ups-no-tags.tsv" PARITY_TAKE_UP_DIFFERENCES="$work/take-up-none.tsv" \
  bash "$HARNESS" --cache "$work/tu-cache-tags" --nuc "$(tu_nuc agrees "$TU_AGREES")" --take-ups
tu_git "$tu_project" checkout -q -b no-line "$tu_from"
echo '# No release line.' > "$tu_project/README.md"
tu_git "$tu_project" commit -q -am no-line
tu_no_line="$(tu_git "$tu_project" rev-parse HEAD)"
tu_git "$tu_project" checkout -q main
printf 'file://%s\t%s\t%s\tfile://%s\n' "$tu_project" "$tu_from" "$tu_no_line" "$tu_template" > "$work/take-ups-no-line.tsv"
expect fail "a take-up whose README names no release fails" "its README names no release" \
  env PARITY_TAKE_UPS="$work/take-ups-no-line.tsv" PARITY_TAKE_UP_DIFFERENCES="$work/take-up-none.tsv" \
  bash "$HARNESS" --cache "$work/tu-cache" --nuc "$(tu_nuc agrees "$TU_AGREES")" --take-ups
printf 'v0.10.0\tb.txt\n' > "$work/take-up-bad.tsv"
expect fail "a row of the table of differences without its reason is refused" \
  "is not '<release><TAB><path><TAB><reason>'" \
  run_take_ups "$(tu_nuc agrees "$TU_AGREES")" "$work/take-up-bad.tsv"
# Each way a path can differ from the take-up fails, an ignored path and the executable bit
# included.
expect fail "a replay that deletes a path the take-up keeps fails" \
  "a.txt ( D) is deleted where the take-up keeps it" \
  run_take_ups "$(tu_nuc deletes 'printf -- "- **Template release:** v0.10.0\n" > README.md; rm a.txt')" \
  "$work/take-up-none.tsv"
expect fail "a replay that writes an ignored path the human left alone fails" \
  "x.log (!!) is written where the take-up has none" \
  run_take_ups "$(tu_nuc ignored "$TU_AGREES; echo log > x.log")" "$work/take-up-none.tsv"
expect fail "a replay that writes another executable bit fails" \
  "a.txt ( M) has another executable bit than the take-up" \
  run_take_ups "$(tu_nuc mode "$TU_AGREES; chmod +x a.txt")" "$work/take-up-none.tsv"
# A run over a corpus reads the tables of the take-ups before any comparison, and replays them in
# its last shard.
c="$(corpus "$work/tu-agree.toml" agree)"
# run_with_take_ups <table> <corpus> <nuc> — the harness on the corpus, replaying the table.
# shellcheck disable=SC2317,SC2329 # run through expect
run_with_take_ups() {
  (export PARITY_TAKE_UPS="$1" PARITY_TAKE_UP_DIFFERENCES="$work/take-up-none.tsv"; run "$2" "$3")
}
out="$(run_with_take_ups "$work/take-ups-bad.tsv" "$c" "$(reactive_nuc)" 2>&1)"; rc=$?
if ((rc == 2)) && [[ "$out" == *"is not '<url><TAB><commit><TAB><commit><TAB><template url>'"* \
  && "$out" != *"finding(s) of the ported rules"* ]]; then
  passed=$((passed + 1))
else
  failed=$((failed + 1))
  echo "FAIL: a run over a corpus refuses a malformed table of the take-ups before any comparison (exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
fi
expect fail "a run over a corpus replays the take-ups in its last shard" "its commits cannot be fetched from" \
  run_with_take_ups "$work/take-ups-gone.tsv" "$c" "$(reactive_nuc)"

summary "Parity harness self-test"
