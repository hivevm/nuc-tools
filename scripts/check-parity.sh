#!/usr/bin/env bash
#
# Parity harness (TEST-parity-harness). Verifies: Q-1 Q-2
#
# Runs the bash checks a repository of the corpus carries and nuc on the same tree, and fails on
# any difference in their findings or their exit status. The corpus is tests/parity/corpus.toml:
# each repository by URL and commit, fetched into an ignored cache under .scratch/parity/; a
# cache that cannot be filled fails the harness, never skips the repository (Q-1). A tree whose
# README names a template release before v0.5.0, or none, is refused and never compared.
#
# Each tree is compared as pinned and on each variant of tests/parity/variants.tsv that is for it:
# an edit that makes a ported rule fire, applied to a copy in a directory of the harness's own,
# never to the cache. A variant is for the trees it names, or for every tree when it names none.
# It fails on a tree it is for where its edit does not apply, or where bash reports no finding of
# its rule, since agreement on a rule that never fires decides nothing; so does one that names a
# tree the corpus lacks.
#
# The comparisons run in parallel, one job per processor or PARITY_JOBS, and their outputs follow
# in the order of the corpus and the variants, after what the preparation of the trees reports,
# such as a refused tree. Every run computes every tree and variant anew. With --shard <k>/<n>, a
# run makes only its share of the comparisons: each tree as pinned and each variant for it is one,
# in the order of the corpus, each tree as pinned first and then its variants, and the n shards
# take contiguous slices of that sequence of about equal cost. A comparison costs the 'cost' of its
# tree in the corpus. WHOLE_COST, the seconds of the parts that read all the template's trees at
# once, the pins of the copied fixture tables, the releases a rule's check changed at, and the
# hook scenarios, which run in the last shard alone, n/n, is appended at the end of the sequence.
# The deal reads the corpus and the variants alone, so that every shard deals the same, a tree it
# refuses or cannot fetch included. A slice that starts inside a tree also compares that tree as
# pinned, since its variants are measured against it, but reports that comparison only where it is
# its own; the n runs from 1/n to n/n report every comparison once, on as many machines, and a
# shard fails only on its own. A variant whose tree as pinned gave no whole bash output fails.
# Every shard fills and checks the whole cache, so that a malformed corpus, a tree it refuses or
# cannot fetch, and a variant that names a tree the corpus lacks are reported by and fail each.
# For each repository and each variant:
#   1. The bash scripts with a ported rule run in the tree, one after the other, with the
#      environment cleared to PATH, HOME, LC_ALL=C, and that of a pull request into main marked
#      ready (CI_ENV): scripts/check-docs.sh and scripts/check-traceability.sh (BASH_CHECKS),
#      whose findings are their '  - ' lines; scripts/check-hooks.sh where the tree has it, from
#      v0.12.0 on, and scripts/check-devcontainer.sh where it has it, from v0.5.0 on, whose
#      findings are their '  - ' lines too; and scripts/check-shell.sh where it has
#      the rule shell/mode, from v0.10.0 on, whose findings are its lines that end in a key of
#      shell/.
#      Their outputs are read as one, and bash fails when
#      either fails; a tree that lacks one of them, or a script that fails without a finding,
#      fails the harness.
#   2. nuc runs 'check' on the tree, with a nuc.toml the harness writes beside it, ignored through
#      the cache's .git/info/exclude, that turns on every rule the table maps to. Where the tree
#      has a nuc.toml of its own, the harness's keeps its minimum and its hook/ rules, which
#      check-hooks.sh holds; the bash checks run after it is written, and read the same file.
#   3. The findings are normalized to (rule ID, file, line) by tests/parity/normalize.tsv and
#      compared as sets. A bash message no row matches is a difference; one a row maps to '-' is
#      a check not ported yet, left out. The exit status is compared for the ported rules: nuc
#      exits 1 exactly when bash reports a finding of a ported rule, and never 2.
#
# The bash check of a rule changes between the template's releases where tests/parity/releases.tsv
# says, and nowhere else: each function it names is compared, comment lines left out, across the
# trees of the corpus that are the template's, those that still carry TEMPLATE-SETUP.md, in the
# order of their releases. A change the table does not list, or one it lists that the trees do not
# show, fails the harness and names the rule; tests/releases.rs holds the rules to the table
# (API-version-coupling-hook-defaults). A tree whose script lacks the function has no such check,
# and its body counts as empty, so the release a check comes or goes at is a change. A release the
# table names that is later than every tree of the template is ahead of the corpus and not
# compared; a function no tree has fails, unless its row names such a release. With fewer than two
# trees of the template there is nothing to compare. A helper the function calls is not compared.
#
# Before the comparisons, each blob of tests/tables.blobs, the pins of the copies of the
# template's fixture tables under tests/tables/, is held to the blob the template's tag of its
# release has at scripts/fixtures/, fetched from the corpus's template into the cache
# (TEST-template-table-cases).
#
# Then the hook scenarios run on the template's trees of the corpus, through
# scripts/lib/hook-parity.sh (TEST-hook-scenarios-on-bash-hooks): each scenario of
# tests/parity/hook-scenarios.sh through git, with the tree's bash hook and with the reference shim
# and nuc, compared as that file says. A tree whose .githooks/nuc.blobs pins its hooks carries
# nuc's shims, not bash hooks, and is left out of them.
#
# The tree of the corpus runs its own bash scripts: the corpus is pinned by commit, and a pull
# request that moves a pin is reviewed like code. nuc is built from the working tree by
# 'cargo build --locked'; the harness is a test of the build, as scripts/check-build.sh is
# (SEC-gates-run-released-nuc).
#
# Pure bash + coreutils/git/awk, and cargo for the build. Usage:
#     scripts/check-parity.sh [--root <dir>] [--corpus <file>] [--cache <dir>] [--nuc <binary>]
#                             [--variants <file>] [--no-hooks] [--shard <k>/<n>]
#     scripts/check-parity.sh [--root <dir>] [--nuc <binary>] --hooks <template tree>...
#     scripts/check-parity.sh --compare <bash output> <bash exit> <nuc output> <nuc exit>
# --root names the repository whose corpus, tables, and build it reads, this one by default; the
# harness changes into it, so a relative path given to an option or in PARITY_TABLE or
# PARITY_RELEASES is read from that root, not from the directory it was called in.
# --nuc runs the given binary instead of building one; --no-hooks leaves the hook scenarios out;
# --hooks runs them alone, on the trees given; --shard runs one of n shares of the comparisons;
# --compare compares one pair of outputs, for scripts/test-check-parity.sh. PARITY_TABLE names
# another normalization table, PARITY_RELEASES another table of releases, PARITY_TABLE_BLOBS
# another list of the blobs of the copied fixture tables, PARITY_JOBS the number of comparisons
# that run at once, and PARITY_WHOLE_COST other seconds for the parts the last shard alone runs.
# Exit code 0 when every repository and variant agrees, 1 on a difference, a refused tree, a
# variant that fits no tree or does not fire, a cache that cannot be filled, or a pin of a copied
# table the template's tag does not bear out, 2 on a missing or malformed list of pins, a malformed
# corpus, table, or variants file, a cost of a tree that is no number of seconds above 0, a
# PARITY_JOBS that is no number of jobs, a PARITY_WHOLE_COST that is none, a shard that is not
# <k>/<n> with 1 <= k <= n, a deal of the comparisons that does not come out whole, a root that
# does not exist, or an argument it does not read.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/bash.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/bash.sh"

# The harness reads the bash output byte by byte, as the checks write it under LC_ALL=C: in a
# UTF-8 locale a byte that is no UTF-8 would keep a table row from matching its message.
export LC_ALL=C

# The libraries are this script's own, wherever --root points; enter_root changes the directory.
LIB="$(cd "$(dirname "${BASH_SOURCE[0]}")/lib" && pwd)"
# shellcheck source-path=SCRIPTDIR source=lib/root.sh
source "$LIB/root.sh"
# The usage is one line, as enter_root reads which options take a value from it.
usage="scripts/check-parity.sh [--root <dir>] [--corpus <file>] [--cache <dir>] [--nuc <binary>] [--variants <file>] [--no-hooks] [--shard <k>/<n>] [--hooks <template tree>...] | --compare <bash output> <bash exit> <nuc output> <nuc exit>"
enter_root "${BASH_SOURCE[0]}" "$usage" "$@"
set -- "${ARGS[@]}"
TABLE="${PARITY_TABLE:-$ROOT/tests/parity/normalize.tsv}"
TABLE_BLOBS="${PARITY_TABLE_BLOBS:-$ROOT/tests/tables.blobs}"
RELEASES="${PARITY_RELEASES:-$ROOT/tests/parity/releases.tsv}"

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$LIB/check.sh"
# shellcheck source-path=SCRIPTDIR source=lib/pin.sh
source "$LIB/pin.sh"
# shellcheck source-path=SCRIPTDIR source=lib/hook-parity.sh
source "$LIB/hook-parity.sh"

# Git hands a hook its repository through variables that would bind the cache's git commands to
# this repository; they are not the harness's (scripts/lib/selftest.sh does the same).
# shellcheck disable=SC2046 # one variable name per word
unset $(git rev-parse --local-env-vars)

die() { echo "Parity check: $1" >&2; exit 2; }

# load_table — read the normalization table, once the arguments are read: an argument the harness
# does not read is refused before it reads anything.
load_table() {
  patterns=() rules=() modes=() places=()
  while IFS=$'\t' read -r pattern rule mode place extra; do
    [[ -z "$pattern" || "$pattern" == \#* ]] && continue
    if ! [[ -n "$rule" && ( "$mode" == "line" || "$mode" == "-" ) && -z "${extra:-}" ]] \
      || ! [[ -z "${place:-}" || "$place" == =?* || "$place" == +?* || "$place" == "@adr" \
              || "$place" == "@word" ]]; then
      die "$TABLE: the row '$pattern' is not 'pattern<TAB>rule ID<TAB>line|-[<TAB>=path|+prefix|@adr|@word]'"
    fi
    patterns+=("$pattern") rules+=("$rule") modes+=("$mode") places+=("${place:-}")
  done < "$TABLE" || die "$TABLE: unreadable"
}

# mode_of <rule> — whether the comparison keeps the line of the rule's findings: line or '-'.
mode_of() {
  local i
  for i in "${!rules[@]}"; do [[ "${rules[i]}" == "$1" ]] && { echo "${modes[i]}"; return; }; done
  echo "unknown"
}

# key <rule> <file> <line> <text> — the normalized finding: its line kept only where the table
# says so, and its text, which nuc words as the bash does.
key() {
  local line="$3"
  [[ "$(mode_of "$1")" == "line" ]] || line=""
  printf '%s\t%s\t%s\t%s\n' "$1" "$2" "$line" "$(canonical "$4")"
}

# canonical <text> — the text without the bytes that are no UTF-8 and without U+FFFD: the bash
# prints such a byte as it is, nuc shows it as U+FFFD, and both name the same byte.
canonical() {
  local text
  text="$(printf '%s' "$1" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)"
  printf '%s' "${text//$'\xef\xbf\xbd'/}"
}

# The key that ends a bash message from v0.9.1 on, '[<rule>:<reason>]', as nuc's lines end.
BASH_KEY_RE=' \[[a-z0-9]+/[a-z0-9-]+:([a-z]+)\]$'

# unkeyed <message> — the bash message without its key: the table matches the wording before it.
unkeyed() {
  local msg="$1"
  [[ "$msg" =~ $BASH_KEY_RE ]] && msg="${msg% \[*\]}"
  printf '%s' "$msg"
}

# compare <label> <bash output> <bash exit> <nuc output> <nuc exit> — print the differences of
# one pair and return 1 when there are any.
compare() {
  local label="$1" bash_out="$2" bash_rc="$3" nuc_out="$4" nuc_rc="$5"
  local -a diffs=() bash_keys=() nuc_keys=() nuc_files=()
  local line msg i f best rule file n text reason bash_any=0 k
  # The reason of each finding nuc reports, by its key; and of each the bash reports with a key,
  # from v0.9.1 on, whose reason is compared with nuc's. The key's rule ID is not compared, since
  # it is the one of the bash's release.
  local -A nuc_reasons=() bash_reasons=()
  while IFS= read -r line; do
    # The location ends before the leftmost ': <rule ID>: ' of a rule the table knows, so that a
    # path holding ': ' stays whole and a message quoting such a pair keeps it; a line without
    # one is read as before, its location up to the first colon.
    rule="" file=""
    for i in "${!rules[@]}"; do
      f="${line%%": ${rules[i]}: "*}"
      if [[ "$f" != "$line" ]] && [[ -z "$rule" || ${#f} -lt ${#file} ]]; then
        rule="${rules[i]}" file="$f"
      fi
    done
    if [[ -z "$rule" && "$line" =~ ^([^:]+(:[0-9]+)?):\ ([a-z0-9]+/[a-z0-9-]+):\  ]]; then
      file="${BASH_REMATCH[1]}" rule="${BASH_REMATCH[3]}"
    fi
    if [[ -n "$rule" ]]; then
      n="" text="${line#"$file: $rule: "}"
      # The line ends in its key, '[<rule>:<reason>]' (API-command-line-with-measure), which
      # is no part of the bash's wording before v0.9.1; a finding without it is a difference.
      reason=""
      if [[ "$text" =~ \ \["$rule":([a-z]+)\]$ ]]; then
        reason="${BASH_REMATCH[1]}"
        text="${text% \["$rule":*\]}"
      else
        diffs+=("nuc reports a finding of '$rule' without its key: $line")
      fi
      [[ "$(mode_of "$rule")" == "unknown" ]] && diffs+=("nuc reports '$rule', which the table maps no message to")
      if [[ "$file" =~ ^(.*):([0-9]+)$ ]]; then file="${BASH_REMATCH[1]}" n="${BASH_REMATCH[2]}"; fi
      nuc_files+=("$file")
      # Where the table places the message by its ADR, nuc's file is cut to that ADR's key.
      for i in "${!patterns[@]}"; do
        if [[ "${rules[i]}" == "$rule" && "${places[i]}" == "@adr" && "$text" =~ ${patterns[i]} ]]; then
          if [[ "$file" =~ ^docs/adr/([0-9]{4})-[^/]*\.md$ || "$file" =~ ^docs/adr/(draft-[^/]+)\.md$ \
                || "$file" =~ ^docs/adr/([A-Za-z]+-[^/]+)\.md$ ]]; then
            file="docs/adr/${BASH_REMATCH[1]}"
          fi
          break
        fi
      done
      k="$(key "$rule" "$file" "$n" "$text")"
      nuc_keys+=("$k")
      nuc_reasons["$k"]="$reason"
    else
      diffs+=("nuc output not understood: $line")
    fi
  done < "$nuc_out"
  while IFS= read -r line; do
    case "$line" in
      "  - "*)
        bash_any=1 msg="$(unkeyed "${line#  - }")" rule="" reason=""
        [[ "$line" =~ $BASH_KEY_RE ]] && reason="${BASH_REMATCH[1]}"
        for i in "${!patterns[@]}"; do
          [[ "$msg" =~ ${patterns[i]} ]] && { rule="${rules[i]}"; break; }
        done
        if [[ -z "$rule" ]]; then
          diffs+=("bash message the table does not know: $msg")
        elif [[ "$rule" != "-" ]]; then
          # The location is the longest file nuc names that the message opens with, so that a
          # path holding ': ' stays whole; else the message up to its first ': '.
          file="${msg%%: *}" text="${msg#*: }" n="" best=""
          for f in "${nuc_files[@]}"; do
            if [[ "$msg" == "$f: "* || "$msg" =~ ^"$f":[0-9]+:\  ]] && ((${#f} > ${#best})); then
              best="$f"
            fi
          done
          if [[ -n "$best" ]]; then
            text="${msg#"$best"}" file="$best"
            [[ "$text" =~ ^:([0-9]+):\  ]] && file="$best:${BASH_REMATCH[1]}"
            text="${text#*: }"
          fi
          if [[ "$file" =~ ^(.*):([0-9]+)$ ]]; then
            file="${BASH_REMATCH[1]}"
            [[ "${modes[i]}" == "line" ]] && n="${BASH_REMATCH[2]}"
          fi
          # The text is the message after its location, or the whole message where the table
          # places it at a path of its own.
          case "${places[i]}" in
            =*) file="${places[i]#=}" text="$msg" ;;
            +*) file="${places[i]#+}$file" ;;
            @adr) file="${msg%% *}" file="docs/adr/${file#ADR-}" text="$msg" ;;
            @word) file="${msg%% *}" text="${msg#* }" ;;
          esac
          k="$(key "$rule" "$file" "$n" "$text")"
          bash_keys+=("$k")
          [[ -n "$reason" ]] && bash_reasons["$k"]="$reason"
        fi
        ;;
      "" | "Documentation checks FAILED:" | "Documentation checks passed." \
        | "Traceability check FAILED:" | "Traceability check passed ("*")." ) ;;
      *) [[ "$line" =~ ^[0-9]+\ problem\(s\)\ found\.$ ]] || diffs+=("bash output not understood: $line") ;;
    esac
  done < "$bash_out"

  local only_bash only_nuc
  only_bash="$(comm -23 <(printf '%s\n' "${bash_keys[@]}" | sort -u) <(printf '%s\n' "${nuc_keys[@]}" | sort -u))"
  only_nuc="$(comm -13 <(printf '%s\n' "${bash_keys[@]}" | sort -u) <(printf '%s\n' "${nuc_keys[@]}" | sort -u))"
  while IFS= read -r line; do [[ -n "$line" ]] && diffs+=("only bash reports: ${line//$'\t'/ }"); done <<< "$only_bash"
  while IFS= read -r line; do [[ -n "$line" ]] && diffs+=("only nuc reports: ${line//$'\t'/ }"); done <<< "$only_nuc"
  for k in "${!bash_reasons[@]}"; do
    [[ -v nuc_reasons["$k"] && "${nuc_reasons["$k"]}" != "${bash_reasons["$k"]}" ]] \
      && diffs+=("nuc gives the reason '${nuc_reasons["$k"]}' where bash gives '${bash_reasons["$k"]}': ${k//$'\t'/ }")
  done

  ((bash_any || bash_rc == 0)) || diffs+=("bash exited $bash_rc without a finding")
  local expected=0
  ((${#bash_keys[@]})) && expected=1
  if [[ "$nuc_rc" != 0 && "$nuc_rc" != 1 ]]; then
    diffs+=("nuc could not decide (exit $nuc_rc): $(head -c 300 "$nuc_out.err" 2>/dev/null)")
  elif ((nuc_rc != expected)); then
    diffs+=("nuc exited $nuc_rc where bash reports $((${#bash_keys[@]})) finding(s) of the ported rules")
  fi

  if ((${#diffs[@]})); then
    echo "Parity check: $label — ${#diffs[@]} difference(s):" >&2
    printf '  - %s\n' "${diffs[@]}" >&2
    return 1
  fi
  echo "Parity check: $label — ${#bash_keys[@]} finding(s) of the ported rules, the same from bash and nuc."
}

if [[ "${1:-}" == "--compare" ]]; then
  (($# == 5)) || die "--compare takes <bash output> <bash exit> <nuc output> <nuc exit>"
  load_table
  compare "pair" "$2" "$3" "$4" "$5"
  exit
fi

corpus="$ROOT/tests/parity/corpus.toml" cache="$ROOT/.scratch/parity" nuc=""
variants="$ROOT/tests/parity/variants.tsv"
# Whether the hook scenarios run, and the trees they run on alone under --hooks.
hooks=1 hooks_only=0
hook_trees=()
# The share of the comparisons this run makes, k of n; 1/1 is all of them.
shard=1/1
# The earliest template release a tree of the corpus may carry.
FLOOR=v0.5.0
while (($#)); do
  case "$1" in
    --corpus) corpus="${2:-}"; shift 2 || die "--corpus takes a file" ;;
    --cache) cache="${2:-}"; shift 2 || die "--cache takes a directory" ;;
    --nuc) nuc="${2:-}"; shift 2 || die "--nuc takes a binary" ;;
    --variants) [[ -n "${2:-}" ]] || die "--variants takes a file"; variants="$2"; shift 2 ;;
    --no-hooks) hooks=0; shift ;;
    --shard) shard="${2:-}"; shift 2 || die "--shard takes <k>/<n>" ;;
    --hooks) hooks_only=1; shift; while (($#)) && [[ "$1" != --* ]]; do hook_trees+=("$1"); shift; done ;;
    *) rest_args "${BASH_SOURCE[0]}" "$usage" 0 "$@" ;;
  esac
done
if ! [[ "$shard" =~ ^([1-9][0-9]*)/([1-9][0-9]*)$ ]] || ((BASH_REMATCH[1] > BASH_REMATCH[2])); then
  die "the shard is '$shard', not <k>/<n> with 1 <= k <= n"
fi
shard_k="${BASH_REMATCH[1]}" shard_n="${BASH_REMATCH[2]}"
# Whether this run makes the parts that read all the template's trees at once: the last shard.
whole=$((shard_k == shard_n))
load_table

# The corpus, one 'name url commit' line per repository, and the cost of one comparison of each
# tree, its key 'cost' or '-' where it has none.
mapfile -t repositories < <(awk '
  /^[[:space:]]*(#|$)/ { next }
  /^\[\[repository\]\]$/ { if (n++) print name, url, commit, cost; name = url = commit = ""; cost = "-"; costed = 0; next }
  match($0, /^(name|url|commit) = "[^"]*"$/) {
    value = $0; sub(/^[a-z]+ = "/, "", value); sub(/"$/, "", value)
    if ($1 == "name") name = value; else if ($1 == "url") url = value; else commit = value
    next
  }
  n && /^cost = [^ ]+$/ && !costed++ { cost = $3; next }
  { print "malformed: " $0; exit }
  END { if (n) print name, url, commit, cost }
' "$corpus" 2>/dev/null) || die "$corpus: unreadable"
((${#repositories[@]})) || die "$corpus: no repository"
seen_names=() costs=()
for entry in "${repositories[@]}"; do
  read -r name url commit cost extra <<< "$entry"
  [[ "$name" =~ ^[a-z0-9-]+$ && -n "$url" && "$commit" =~ ^[0-9a-f]{40}$ && -z "$extra" ]] \
    || die "$corpus: an entry needs a name of [a-z0-9-], a url, and a 40-hex commit: '$entry'"
  [[ "$cost" == - || ( "$cost" =~ ^[0-9]+(\.[0-9]+)?$ && "$cost" =~ [1-9] ) ]] \
    || die "$corpus: the repository '$name' has the cost '$cost', not a number of seconds above 0"
  # A name is the directory of its tree in the cache and the files of its comparisons.
  _contains "$name" "${seen_names[@]}" && die "$corpus: the repository '$name' is named twice"
  seen_names+=("$name") costs+=("$cost")
done
# A tree without a cost takes the largest cost of the corpus, or 1 where no tree has one: an
# unmeasured tree is dealt as the dearest, so that its shard is not overloaded.
default_cost="$(printf '%s\n' "${costs[@]}" | grep -vx -- - | sort -g | tail -n 1)"
default_cost="${default_cost:-1}"

# The number of comparisons that run at once: PARITY_JOBS, or one per processor.
jobs_max="${PARITY_JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 1)}"
[[ "$jobs_max" =~ ^[1-9][0-9]*$ ]] || die "PARITY_JOBS is '$jobs_max', not a number of jobs"
# The seconds the parts that read all the template's trees at once take, which the last shard
# alone runs, appended at the end of the sequence: PARITY_WHOLE_COST, or WHOLE_COST. Measured with
# PARITY_JOBS=4, as tests/parity/corpus.toml measures the cost of a tree, the hook scenarios took
# 146 s, the pins of the copied tables and the releases a few more.
WHOLE_COST=150
whole_cost="${PARITY_WHOLE_COST:-$WHOLE_COST}"
[[ "$whole_cost" =~ ^[0-9]+(\.[0-9]+)?$ ]] || die "PARITY_WHOLE_COST is '$whole_cost', not a number of seconds"

# The table of releases, one 'rule<TAB>script<TAB>function<TAB>releases' entry per row; the fields
# after them, the releases a reason covers and the reason, are tests/releases.rs's, not the
# harness's.
release_rows=()
while IFS= read -r row || [[ -n "$row" ]]; do
  [[ "$row" =~ ^[[:space:]]*(#|$) ]] && continue
  IFS=$'\t' read -r r_rule r_script r_function r_changed _ <<< "$row"
  [[ "$r_rule" =~ ^[a-z]+/[a-z0-9-]+$ && "$r_script" == scripts/*.sh \
    && "$r_function" =~ ^(-|[a-z_][a-z0-9_]*)$ && "$r_changed" =~ ^(-|v[0-9.]+(,v[0-9.]+)*)$ ]] \
    || die "$RELEASES: a row needs a rule ID, a script under scripts/, a function or '-', and releases or '-': '$row'"
  release_rows+=("$r_rule $r_script $r_function $r_changed")
done < "$RELEASES" || die "$RELEASES: unreadable"

if [[ -z "$nuc" ]]; then
  (cd "$ROOT" && cargo build --locked -q) || { echo "Parity check: nuc could not be built" >&2; exit 1; }
  nuc="${CARGO_TARGET_DIR:-$ROOT/target}/debug/nuc"
fi

# The nuc.toml of the harness: every rule the table maps to, at the lowest minimum.
ported="$(printf '%s\n' "${rules[@]}" | grep -vx -- '-' | sort -u | sed 's/.*/"&"/' | paste -sd, -)"

# tree_config <dir> — the nuc.toml of the harness for the tree: every rule the table maps to; and
# where the tree has a nuc.toml of its own, its minimum in place of the lowest and its hook/ rules
# after them, read as scripts/check-hooks.sh reads them, so that the check of the file's clause 3
# (NUC-hooks-run-nuc §3) decides the tree's own minimum and hook/ rules on both sides.
tree_config() {
  local minimum="0.0.0" hooks=""
  if [[ -f "$1/nuc.toml" ]]; then
    local own
    own="$(sed -nE 's/^[[:space:]]*minimum[[:space:]]*=[[:space:]]*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/p' "$1/nuc.toml" | head -n 1)"
    [[ -n "$own" ]] && minimum="$own"
    hooks="$(awk '
      {
        line = ""; quoted = 0
        for (i = 1; i <= length($0); i++) {
          c = substr($0, i, 1)
          if (c == "\"") quoted = !quoted
          else if (c == "#" && !quoted) break
          line = line c
        }
      }
      !open && line ~ /^[[:space:]]*rules[[:space:]]*=/ { open = 1; sub(/^[^=]*=/, "", line) }
      open { text = text " " line; if (index(line, "]")) open = 0 }
      END { print text }' "$1/nuc.toml" | grep -oE '"hook/[^"]*"' | paste -sd, -)"
  fi
  printf 'minimum = "%s"\nrules = [%s%s]\n' "$minimum" "$ported" "${hooks:+,$hooks}"
}

work="$(mktemp -d)"
# kill_tree <pid> — stop the process and every process it started, the deepest first, so that no
# bash check or nuc of a comparison runs on after the harness: one left running would write into a
# cached tree or into $work after the trap removed it.
kill_tree() {
  local child
  for child in $(pgrep -P "$1" 2>/dev/null); do kill_tree "$child"; done
  kill "$1" 2>/dev/null
}
# stop_jobs — stop every comparison still running, with what it started, and wait for them.
stop_jobs() {
  local job
  for job in $(jobs -p); do kill_tree "$job"; done
  wait
}
# A run stopped stops its comparisons and what they started.
trap 'stop_jobs; rm -rf "$work"' EXIT
cgit() { git -c core.hooksPath=/dev/null -c core.fsmonitor=false -C "$@"; }

# release_of <README> — the release its first Template release line names, read as check 14 of
# check-docs.sh reads it: a vX.Y.Z at the start of the value, after one opening bracket or
# backquote at most, followed by the end, a space, a punctuation mark, or a full stop that ends
# the sentence; nothing for 'unreleased', a pre-release, or any other value.
release_of() {
  local line value num='(0|[1-9][0-9]*)'
  line="$(grep -m1 -E '^[[:space:]]*([-*] )?\*\*Template release:\*\*' "$1" 2>/dev/null)" || return 0
  value="${line#*\*\*Template release:\*\*}"
  value="${value#"${value%%[![:space:]]*}"}"
  case "$value" in '['* | '`'*) value="${value:1}" ;; esac
  # shellcheck disable=SC2016 # the backquote is Markdown to match, not a command substitution
  local re='^(v'"$num"'\.'"$num"'\.'"$num"')($|[][:space:],;:)`]|\.([[:space:]]|$))'
  [[ "$value" =~ $re ]] && echo "${BASH_REMATCH[1]}"
}

# --hooks: the hook scenarios alone, on the template trees given.
if ((hooks_only)); then
  hook_parity "$nuc" "$work" "${hook_trees[@]}"
  rc=$?
  ((rc)) && echo "Parity check FAILED." >&2
  exit "$rc"
fi

# check_body <script> <function> — the function's lines, from its 'name() {' line to the first
# '}' alone on a line, or the whole script for '-', comment lines left out; fails when the script
# or the function is missing. From v0.9.0 on the template ends the head in a comment that names
# its check, '# check 15' or '# docs/prose-width', which is left out with it.
check_body() {
  [[ -f "$1" ]] || return 1
  if [[ "$2" == - ]]; then
    grep -v '^[[:space:]]*#' "$1"
    return 0
  fi
  awk -v head="$2() {" '
    !on && ($0 == head || (index($0, head) == 1 && substr($0, length(head) + 1) ~ /^[ \t]+#/)) {
      on = 1
      print head
      next
    }
    on && !/^[[:space:]]*#/ { print }
    on && $0 == "}" { done = 1; exit }
    END { exit !done }
  ' "$1"
}

# at_least_floor <vX.Y.Z> — whether the release is FLOOR or later.
at_least_floor() {
  [[ -n "$1" ]] && [[ "$(printf '%s\n%s\n' "$FLOOR" "$1" | sort -V | head -1)" == "$FLOOR" ]]
}

# The environment of a pull request into main marked ready, in which both run, so that a check
# that reads where a change lands (check 16) speaks as it does in CI.
CI_ENV=(GITHUB_BASE_REF=main PULL_REQUEST_DRAFT=false)

# The bash scripts whose rules nuc ports, run in this order; check-hooks.sh and
# check-devcontainer.sh, each run where the tree has it; and check-shell.sh, run where it has the
# one rule of it nuc ports, whose finding lines end in a key of shell/.
BASH_CHECKS=(scripts/check-docs.sh scripts/check-traceability.sh)
HOOKS_CHECK=scripts/check-hooks.sh
DEVCONTAINER_CHECK=scripts/check-devcontainer.sh
SHELL_CHECK=scripts/check-shell.sh
SHELL_FINDING_RE='^[^ ].* \[shell/[a-z-]+:[a-z]+\]$'

# run_pair <label> <tree> <output prefix> — run bash and nuc on the tree and compare them.
run_pair() {
  local label="$1" dir="$2" out="$3" bash_rc=0 rc script nuc_rc
  : > "$out.bash"
  for script in "${BASH_CHECKS[@]}"; do
    if [[ ! -f "$dir/$script" ]]; then
      echo "Parity check: $label — the tree has no $script" >&2
      return 1
    fi
    (cd "$dir" && env -i PATH="$PATH" HOME="$HOME" LC_ALL=C "${CI_ENV[@]}" bash "$script") > "$out.script" 2>&1
    rc=$?
    # Each script that fails names a finding: the joined output cannot tell which one failed.
    if ((rc)) && ! grep -q '^  - ' "$out.script"; then
      echo "Parity check: $label — bash: $script exited $rc without a finding" >&2
      return 1
    fi
    cat "$out.script" >> "$out.bash"
    ((bash_rc)) || bash_rc=$rc
  done
  # check-hooks.sh of a tree that has it, from v0.12.0 on, and check-devcontainer.sh, which every
  # release from v0.5.0 on has: their findings are their '  - ' lines, as those of check-docs.sh
  # are, and each fails only with one.
  for script in "$HOOKS_CHECK" "$DEVCONTAINER_CHECK"; do
    [[ -f "$dir/$script" ]] || continue
    (cd "$dir" && env -i PATH="$PATH" HOME="$HOME" LC_ALL=C "${CI_ENV[@]}" bash "$script") > "$out.script" 2>&1
    rc=$?
    if ((rc)) && ! grep -q '^  - ' "$out.script"; then
      echo "Parity check: $label — bash: $script exited $rc without a finding" >&2
      return 1
    fi
    grep '^  - ' "$out.script" >> "$out.bash"
    ((bash_rc)) || bash_rc=$rc
  done
  # check-shell.sh of a tree that has the rule shell/mode, from v0.10.0 on: its findings are the
  # lines that end in a key of shell/, read as findings of the other scripts are; the rest of its
  # output, ShellCheck's among it, and its exit, which a rule nuc leaves to the bash sets too, are
  # not compared (ARCH-rules-need-only-git).
  if [[ -f "$dir/$SHELL_CHECK" ]] && grep -qF '[shell/mode:exec]' "$dir/$SHELL_CHECK"; then
    (cd "$dir" && env -i PATH="$PATH" HOME="$HOME" LC_ALL=C "${CI_ENV[@]}" bash "$SHELL_CHECK") > "$out.script" 2>&1
    if grep -E "$SHELL_FINDING_RE" "$out.script" | sed 's/^/  - /' >> "$out.bash"; then
      ((bash_rc)) || bash_rc=1
    fi
  fi
  # The bash output is whole: a variant may be measured against it.
  : > "$out.whole"
  env -i PATH="$PATH" HOME="$HOME" "${CI_ENV[@]}" "$nuc" check --repo "$dir" > "$out.nuc" 2> "$out.nuc.err"
  nuc_rc=$?
  compare "$label" "$out.bash" "$bash_rc" "$out.nuc" "$nuc_rc"
}

# apply_variant <copy> <path> <sed|delete|write|chomp|chmod> <argument> — apply the edit to the
# copy of a tree. It fails when the edit does not apply: the file is missing, or for 'write'
# already there, resolves outside the copy, the sed expression fails or changes nothing, for
# 'chomp' the file does not end with a newline, or for 'chmod' git does not track the file or its
# mode in the index stays as it was. 'write' creates the file with the argument as its one line,
# its backslash escapes read as printf's %b reads them, as \x3a for a colon, in a directory the
# copy already has; 'chomp' takes the newline off the file's last line, which sed cannot; 'chmod'
# sets or clears the executable bit in the copy's index, +x or -x, where a rule reads the mode git
# keeps rather than the file's. sed runs sandboxed, so an expression runs no command and reads or
# writes no other file.
apply_variant() {
  local copy="$1" file="$1/$2"
  if [[ "$3" == write ]]; then
    [[ ! -e "$file" && ! -L "$file" && -d "$(dirname "$file")" ]] || return 1
    local parent
    parent="$(realpath "$(dirname "$file")")"
    [[ "$parent" == "$(realpath "$copy")" || "$parent" == "$(realpath "$copy")"/* ]] || return 1
    printf '%b\n' "$4" 2>/dev/null > "$file" || return 1
    return 0
  fi
  # A symbolic link is deleted itself, wherever it points, when its directory is in the copy.
  if [[ "$3" == delete && -L "$file" ]]; then
    local dir
    dir="$(realpath "$(dirname "$file")")"
    [[ "$dir" == "$(realpath "$copy")" || "$dir" == "$(realpath "$copy")"/* ]] || return 1
    rm -f "$file"
    return 0
  fi
  [[ -f "$file" ]] || return 1
  [[ "$(realpath -m "$file")" == "$(realpath "$copy")"/* ]] || return 1
  case "$3" in
    delete) rm -f "$file" ;;
    chomp)
      [[ "$(tail -c 1 "$file" | od -An -c | tr -d ' ')" == '\n' ]] || return 1
      truncate -s -1 "$file" || return 1
      ;;
    chmod)
      local before after
      before="$(cgit "$copy" ls-files -s -- ":(literal)$2")"
      [[ -n "$before" ]] || return 1
      cgit "$copy" update-index --chmod="$4" -- "$2" 2>/dev/null || return 1
      after="$(cgit "$copy" ls-files -s -- ":(literal)$2")"
      [[ "$after" != "$before" ]] || return 1
      ;;
    sed)
      sed --sandbox -E "$4" "$file" > "$file.after" 2>/dev/null || { rm -f "$file.after"; return 1; }
      if cmp -s "$file" "$file.after"; then rm -f "$file.after"; return 1; fi
      mv "$file.after" "$file"
      ;;
  esac
}

# rule_messages <bash output> <rule> — the bash messages the table maps to the rule, sorted.
rule_messages() {
  local line msg i
  while IFS= read -r line; do
    [[ "$line" == "  - "* ]] || continue
    msg="$(unkeyed "${line#  - }")"
    for i in "${!patterns[@]}"; do
      if [[ "$msg" =~ ${patterns[i]} ]]; then
        [[ "${rules[i]}" == "$2" ]] && printf '%s\n' "$msg"
        break
      fi
    done
  done < "$1" | sort -u
}

# fires <tree's bash output> <variant's bash output> <rule> — whether bash reports a finding of
# the rule on the variant that it does not report on the tree itself.
fires() {
  [[ -n "$(comm -13 <(rule_messages "$1" "$3") <(rule_messages "$2" "$3"))" ]]
}

# The variants, one row each: name, rule, path in the tree, 'sed' with an extended expression,
# 'delete' with '-', 'write' with a line, 'chomp' with '-', or 'chmod' with +x or -x, and
# optionally the names of the corpus trees it is for, separated by commas, all separated by tabs.
# Without names it is for every tree. The path stays inside the tree and out of its .git; every
# ported rule brings one at least.
variant_names=() variant_rules=() variant_paths=() variant_ops=() variant_args=() variant_trees=()
[[ -r "$variants" ]] || die "$variants: unreadable"
while IFS=$'\t' read -r vname vrule vpath vop varg vtrees extra || [[ -n "$vname" ]]; do
  [[ -z "$vname" || "$vname" == \#* ]] && continue
  if ! [[ "$vname" =~ ^[a-z0-9-]+$ && -n "$vpath" && -z "${extra:-}" ]] \
    || ! [[ -z "${vtrees:-}" || "$vtrees" =~ ^[a-z0-9-]+(,[a-z0-9-]+)*$ ]] \
    || ! [[ ( "$vop" == sed && -n "$varg" ) || ( "$vop" =~ ^(delete|chomp)$ && "$varg" == - ) || ( "$vop" == write && -n "$varg" ) || ( "$vop" == chmod && "$varg" =~ ^[-+]x$ ) ]]; then
    die "$variants: the row '$vname' is not 'name<TAB>rule<TAB>path<TAB>sed|delete|write|chomp|chmod<TAB>argument[<TAB>tree,…]'"
  fi
  # The repository's own .git is no part of the tree: an edit there could point git, and so both
  # tools, at another repository.
  [[ "$vpath" != /* && "/$vpath/" != */../* && "/$vpath/" != */.git/* ]] \
    || die "$variants: the variant '$vname' edits '$vpath', which is no path inside the tree"
  _contains "$vname" "${variant_names[@]}" && die "$variants: the variant '$vname' is named twice"
  [[ "$(mode_of "$vrule")" != unknown ]] \
    || die "$variants: the variant '$vname' names the rule '$vrule', which the table maps no message to"
  variant_names+=("$vname") variant_rules+=("$vrule") variant_paths+=("$vpath")
  variant_ops+=("$vop") variant_args+=("$varg") variant_trees+=("${vtrees:-}")
done < "$variants"
for rule in $(printf '%s\n' "${rules[@]}" | grep -vx -- '-' | sort -u); do
  _contains "$rule" "${variant_rules[@]}" \
    || die "$variants: the ported rule '$rule' brings no variant; a rule that never fires agrees on every clean tree"
done

failed=0
# A tree a variant names that the corpus lacks: a moved or renamed pin would drop the comparison.
corpus_names=()
for entry in "${repositories[@]}"; do corpus_names+=("${entry%% *}"); done
for i in "${!variant_names[@]}"; do
  IFS=, read -ra named <<< "${variant_trees[i]}"
  for tree in "${named[@]}"; do
    if ! _contains "$tree" "${corpus_names[@]}"; then
      echo "Parity check: variant ${variant_names[i]} — it names the tree '$tree', which the corpus lacks" >&2
      failed=1
    fi
  done
done
# The trees of the corpus, in its order, and those whose cache is filled and whose release is
# compared.
template_trees=() trees=() template_url=""
declare -A ready=()
for entry in "${repositories[@]}"; do
  read -r name url commit _ <<< "$entry"
  dir="$cache/$name"
  trees+=("$name $commit $dir")
  if [[ "$(cgit "$dir" rev-parse -q --verify HEAD 2>/dev/null)" != "$commit" ]]; then
    if ! { rm -rf "$dir" && mkdir -p "$dir" && cgit "$dir" init -q \
      && cgit "$dir" fetch -q --depth 1 "$url" "$commit" \
      && cgit "$dir" checkout -q --detach FETCH_HEAD; }; then
      echo "Parity check: $name — the cache could not be filled from $url at $commit" >&2
      failed=1
      continue
    fi
  fi
  if ! { cgit "$dir" reset -q --hard "$commit" && cgit "$dir" clean -q -fdx; }; then
    echo "Parity check: $name — the cache could not be reset to $commit" >&2
    failed=1
    continue
  fi
  release="$(release_of "$dir/README.md")"
  if ! at_least_floor "$release"; then
    echo "Parity check: $name — its tree carries template release '${release:-none}', not $FLOOR or later; the corpus holds none earlier" >&2
    failed=1
    continue
  fi
  [[ -f "$dir/TEMPLATE-SETUP.md" ]] && template_trees+=("$release $dir") template_url="$url"
  tree_config "$dir" > "$dir/nuc.toml.harness" && mv "$dir/nuc.toml.harness" "$dir/nuc.toml"
  grep -qx '/nuc.toml' "$dir/.git/info/exclude" 2>/dev/null || echo '/nuc.toml' >> "$dir/.git/info/exclude"

  ready["$name"]=1
done

# check_table_blobs — the pins of the copies of the template's fixture tables
# (TEST-template-table-cases): each line '<release> <blob> <file>' of TABLE_BLOBS names the
# blob of scripts/fixtures/<file> at the release's tag of the template, which a repository of its
# own in the cache fetches from the corpus's template where it lacks the tag. A tag it cannot read
# fails, and so does a blob the tag does not hold; no tree of that release need be pinned.
check_table_blobs() {
  local line release blob file extra at tags="$cache/.tables"
  [[ -f "$TABLE_BLOBS" ]] || { echo "Parity check: $TABLE_BLOBS — not found" >&2; exit 2; }
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -z "$line" || "$line" == "#"* ]] && continue
    read -r release blob file extra <<< "$line"
    if ! [[ "$release" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ && "$blob" =~ ^[0-9a-f]{40}$ \
      && "$file" =~ ^[A-Za-z0-9._-]+$ && -z "$extra" ]]; then
      echo "Parity check: $TABLE_BLOBS — the line '$line' is not '<release> <blob> <file>'" >&2
      exit 2
    fi
    if [[ -z "$template_url" ]]; then
      echo "Parity check: $TABLE_BLOBS — no tree of the template in the corpus to read $release from" >&2
      failed=1
      return
    fi
    if ! cgit "$tags" rev-parse -q --verify "refs/tags/$release" > /dev/null 2>&1 \
      && ! { { [[ -d "$tags/.git" ]] || { mkdir -p "$tags" && cgit "$tags" init -q; }; } \
        && cgit "$tags" fetch -q --depth 1 "$template_url" "refs/tags/$release:refs/tags/$release"; } 2> /dev/null; then
      echo "Parity check: $TABLE_BLOBS — the tag $release cannot be read from $template_url" >&2
      failed=1
      continue
    fi
    at="$(cgit "$tags" rev-parse -q --verify "refs/tags/$release:scripts/fixtures/$file" 2> /dev/null)"
    if [[ "$at" != "$blob" ]]; then
      echo "Parity check: $TABLE_BLOBS — $file is not the blob $release holds at scripts/fixtures/$file (${at:-none there})" >&2
      failed=1
    fi
  done < "$TABLE_BLOBS"
}
((whole)) && check_table_blobs

# run_variant <name> <commit> <dir> <index> — bash and nuc on a copy of the tree with the variant
# applied, in a directory of its own, never on the cache; fails on a difference, on an edit that
# does not apply, and on a variant whose rule bash does not report beyond the tree's own.
run_variant() {
  local name="$1" commit="$2" dir="$3" i="$4" label vdir out rc=0
  label="$name, variant ${variant_names[i]}"
  vdir="$work/variants/$name/${variant_names[i]}"
  if ! { mkdir -p "$(dirname "$vdir")" && cp -a "$dir" "$vdir"; }; then
    echo "Parity check: $label — the tree could not be copied" >&2
    return 1
  fi
  if ! apply_variant "$vdir" "${variant_paths[i]}" "${variant_ops[i]}" "${variant_args[i]}"; then
    echo "Parity check: $label — its edit of ${variant_paths[i]} does not apply to this tree" >&2
    rm -rf "$vdir"
    return 1
  fi
  out="$work/$name.${variant_names[i]}"
  run_pair "$name at ${commit:0:12}, variant ${variant_names[i]}" "$vdir" "$out" || rc=1
  # A tree as pinned whose bash run ended early gives nothing to measure the variant against, and
  # the shard that does not report that comparison would pass the variant on it.
  if [[ ! -f "$work/$name.whole" ]]; then
    echo "Parity check: $label — the tree as pinned gave no whole bash output to measure the variant against" >&2
    rc=1
  elif ! fires "$work/$name.bash" "$out.bash" "${variant_rules[i]}"; then
    echo "Parity check: $label — bash reports no finding of ${variant_rules[i]} beyond the tree's own; the variant does not make its rule fire" >&2
    rc=1
  fi
  rm -rf "$vdir"
  return "$rc"
}

# First the trees as pinned that the shard needs, whose bash output their variants are measured
# against once it is whole, then the shard's own variants, each a job of its own, writing into
# files of its own.
# spawn <unit> <command...> — run the command as a job into the unit's files, once a slot is free.
spawn() {
  local unit="$1"
  shift
  while (($(jobs -rp | wc -l) >= jobs_max)); do wait -n; done
  { "$@" > "$work/$unit.log" 2> "$work/$unit.err"; echo "$?" > "$work/$unit.rc"; } &
}
# report <unit> — replay the unit's output, and fail the harness where the unit failed.
report() {
  cat "$work/$1.log"
  cat "$work/$1.err" >&2
  [[ "$(cat "$work/$1.rc" 2>/dev/null)" == 0 ]] || failed=1
}
# The comparisons of one whole run, tree by tree in the order of the corpus: each tree as pinned,
# then each variant that is for it, in the order of the variants, each costing its tree's cost.
# The shards take contiguous slices of this sequence of about equal cost. The cost of the parts
# that read all the template's trees, which the last shard alone runs, is appended at the end of
# the sequence, so that it falls in the last slice: a comparison goes to the shard in whose share
# of the whole cost its middle falls. The deal reads the corpus and the variants alone, never what
# a fetch did, so that every shard deals the same. A shard keeps its own, '<tree> -' for a tree as
# pinned and '<tree> <variant index>' for a variant, of the trees that are ready.
sequence=()
for j in "${!trees[@]}"; do
  read -r name commit dir <<< "${trees[j]}"
  cost="${costs[j]/#-/$default_cost}"
  for i in - "${!variant_names[@]}"; do
    # A variant that names its trees is for those alone.
    [[ "$i" == - || -z "${variant_trees[i]}" || ",${variant_trees[i]}," == *",$name,"* ]] || continue
    sequence+=("$name $i $cost")
  done
done
count="${#sequence[@]}"
mapfile -t dealt < <(printf '%s\n' "${sequence[@]}" | awk -v n="$shard_n" -v whole="$whole_cost" '
  { cost[NR] = $3; total += $3 }
  END {
    total += whole
    for (u = 1; u <= NR; u++) {
      s = int((at + cost[u] / 2) * n / total) + 1
      print (s > n ? n : s)
      at += cost[u]
    }
  }')
# A deal that lost a comparison would leave a shard nothing to compare, and pass it.
((${#dealt[@]} == count)) || die "the deal of the comparisons failed"
for slice_of in "${dealt[@]}"; do
  if ! [[ "$slice_of" =~ ^[1-9][0-9]*$ ]] || ((slice_of > shard_n)); then
    die "the deal of the comparisons failed"
  fi
done
mine=()
declare -A needed=() at=()
for tree in "${trees[@]}"; do
  read -r name commit dir <<< "$tree"
  at["$name"]="$commit $dir"
done
for u in "${!sequence[@]}"; do
  read -r name i _ <<< "${sequence[u]}"
  if ((dealt[u] == shard_k)) && [[ -v ready["$name"] ]]; then
    mine+=("$name $i")
    needed["$name"]=1
  fi
done
# A variant is measured against its tree's own bash output, so the shard compares every tree as
# pinned that one of its variants is for; it reports that comparison only where it is its own, so
# that the shards together report each one once.
for tree in "${trees[@]}"; do
  read -r name commit dir <<< "$tree"
  [[ -v needed["$name"] ]] && spawn "unit-$name" run_pair "$name at ${commit:0:12}" "$dir" "$work/$name"
done
wait
units=()
for unit in "${mine[@]}"; do
  read -r name i <<< "$unit"
  if [[ "$i" == - ]]; then
    units+=("unit-$name")
    continue
  fi
  read -r commit dir <<< "${at["$name"]}"
  spawn "unit-$name.$i" run_variant "$name" "$commit" "$dir" "$i"
  units+=("unit-$name.$i")
done
wait
for unit in "${units[@]}"; do report "$unit"; done

# The releases at which each rule's bash check changed, across the template's trees in order.
if ((whole && ${#template_trees[@]} >= 2)); then
  mapfile -t template_trees < <(printf '%s\n' "${template_trees[@]}" | sort -V -k1,1)
  for row in "${release_rows[@]}"; do
    read -r r_rule r_script r_function r_changed <<< "$row"
    changed=() previous="" first=1 present=0
    for tree in "${template_trees[@]}"; do
      t_release="${tree%% *}" t_dir="${tree#* }"
      # A release whose script has no such function has no such check yet, or no longer: its body
      # is empty, so that the release it comes or goes at is a change.
      if body="$(check_body "$t_dir/$r_script" "$r_function")"; then present=1; else body=""; fi
      ((first)) || [[ "$body" == "$previous" ]] || changed+=("$t_release")
      previous="$body" first=0
    done
    # A release the row names that is later than every tree of the template in the corpus is ahead
    # of the corpus: it is not compared until a tree of it is pinned.
    newest="${template_trees[-1]%% *}" listed=() ahead=()
    IFS=, read -r -a releases_named <<< "${r_changed/#-/}"
    for t_release in "${releases_named[@]}"; do
      if [[ "$(printf '%s\n' "$newest" "$t_release" | sort -V | tail -n 1)" != "$newest" ]]; then
        ahead+=("$t_release")
      else
        listed+=("$t_release")
      fi
    done
    # A function no tree has is compared nowhere: only a row whose check comes at a release ahead
    # of the corpus may name one, so that a mistyped function or script is no silent pass (Q-1).
    if ((!present)) && ((${#ahead[@]} == 0)); then
      echo "Parity check: $r_rule — no tree of the template in the corpus has $r_function in $r_script, and the row names no release ahead of the corpus" >&2
      failed=1
      continue
    fi
    ((present)) || echo "Parity check: $r_rule — its check comes at ${ahead[*]}, ahead of the corpus, and nothing of it is compared yet"
    r_changed="$(IFS=,; echo "${listed[*]:--}")"
    got="$(IFS=,; echo "${changed[*]:--}")"
    if [[ "$got" != "$r_changed" ]]; then
      echo "Parity check: $r_rule — its bash check changed at $got across the template's trees of the corpus, where ${RELEASES#"$ROOT/"} says $r_changed" >&2
      failed=1
    fi
  done
fi

# The hook scenarios, on the template's trees of the corpus (TEST-hook-scenarios-on-bash-hooks).
if ((hooks && whole)); then
  tree_dirs=()
  for tree in "${template_trees[@]}"; do tree_dirs+=("${tree#* }"); done
  hook_parity "$nuc" "$work" "${tree_dirs[@]}"
  case $? in 0) ;; 2) exit 2 ;; *) failed=1 ;; esac
fi

if ((failed)); then
  echo "Parity check FAILED." >&2
  exit 1
fi
slice=""
((shard_n > 1)) && slice=", shard $shard: ${#mine[@]} of the $count comparisons"
echo "Parity check passed (${#repositories[@]} repositories$slice, ${#variant_names[@]} variants, rules: ${ported:-none})."
