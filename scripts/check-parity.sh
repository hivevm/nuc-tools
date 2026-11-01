#!/usr/bin/env bash
#
# Parity harness (ADR-0022). Verifies: Q-2
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
# For each repository and each variant:
#   1. The bash scripts with a ported rule run in the tree, one after the other, with the
#      environment cleared to PATH, HOME, LC_ALL=C, and that of a pull request into main marked
#      ready (CI_ENV): scripts/check-docs.sh and scripts/check-traceability.sh (BASH_CHECKS),
#      whose findings are their '  - ' lines. Their outputs are read as one, and bash fails when
#      either fails; a tree that lacks one of them, or a script that fails without a finding,
#      fails the harness.
#   2. nuc runs 'check' on the tree, with a nuc.toml the harness writes beside it, ignored through
#      the cache's .git/info/exclude, that turns on every rule the table maps to.
#   3. The findings are normalized to (rule ID, file, line) by tests/parity/normalize.tsv and
#      compared as sets. A bash message no row matches is a difference; one a row maps to '-' is
#      a check not ported yet, left out. The exit status is compared for the ported rules: nuc
#      exits 1 exactly when bash reports a finding of a ported rule, and never 2.
#
# The tree of the corpus runs its own bash scripts: the corpus is pinned by commit, and a pull
# request that moves a pin is reviewed like code. nuc is built from the working tree by
# 'cargo build --locked'; the harness is a test of the build, as scripts/check-build.sh is
# (ADR-0018).
#
# Pure bash + coreutils/git/awk, and cargo for the build. Usage:
#     scripts/check-parity.sh [--corpus <file>] [--cache <dir>] [--nuc <binary>] [--variants <file>]
#     scripts/check-parity.sh --compare <bash output> <bash exit> <nuc output> <nuc exit>
# --nuc runs the given binary instead of building one; --compare compares one pair of outputs,
# for scripts/test-check-parity.sh. PARITY_TABLE names another normalization table.
# Exit code 0 when every repository and variant agrees, 1 on a difference, a refused tree, a
# variant that fits no tree or does not fire, or a cache that cannot be filled, 2 on a malformed
# corpus, table, or variants file or an unknown argument.

set -uo pipefail

# The harness reads the bash output byte by byte, as the checks write it under LC_ALL=C: in a
# UTF-8 locale a byte that is no UTF-8 would keep a table row from matching its message.
export LC_ALL=C

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TABLE="${PARITY_TABLE:-$ROOT/tests/parity/normalize.tsv}"

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$ROOT/scripts/lib/check.sh"

# Git hands a hook its repository through variables that would bind the cache's git commands to
# this repository; they are not the harness's (scripts/lib/selftest.sh does the same).
# shellcheck disable=SC2046 # one variable name per word
unset $(git rev-parse --local-env-vars)

die() { echo "Parity check: $1" >&2; exit 2; }

patterns=() rules=() modes=() places=()
while IFS=$'\t' read -r pattern rule mode place extra; do
  [[ -z "$pattern" || "$pattern" == \#* ]] && continue
  if ! [[ -n "$rule" && ( "$mode" == "line" || "$mode" == "-" ) && -z "${extra:-}" ]] \
    || ! [[ -z "${place:-}" || "$place" == =?* || "$place" == +?* || "$place" == "@adr" ]]; then
    die "$TABLE: the row '$pattern' is not 'pattern<TAB>rule ID<TAB>line|-[<TAB>=path|+prefix|@adr]'"
  fi
  patterns+=("$pattern") rules+=("$rule") modes+=("$mode") places+=("${place:-}")
done < "$TABLE" || die "$TABLE: unreadable"

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

# compare <label> <bash output> <bash exit> <nuc output> <nuc exit> — print the differences of
# one pair and return 1 when there are any.
compare() {
  local label="$1" bash_out="$2" bash_rc="$3" nuc_out="$4" nuc_rc="$5"
  local -a diffs=() bash_keys=() nuc_keys=()
  local line msg i rule file n text bash_any=0
  while IFS= read -r line; do
    case "$line" in
      "  - "*)
        bash_any=1 msg="${line#  - }" rule=""
        for i in "${!patterns[@]}"; do
          [[ "$msg" =~ ${patterns[i]} ]] && { rule="${rules[i]}"; break; }
        done
        if [[ -z "$rule" ]]; then
          diffs+=("bash message the table does not know: $msg")
        elif [[ "$rule" != "-" ]]; then
          file="${msg%%: *}" n=""
          if [[ "$file" =~ ^(.*):([0-9]+)$ ]]; then
            file="${BASH_REMATCH[1]}"
            [[ "${modes[i]}" == "line" ]] && n="${BASH_REMATCH[2]}"
          fi
          # The text is the message after its location, or the whole message where the table
          # places it at a path of its own.
          text="${msg#*: }"
          case "${places[i]}" in
            =*) file="${places[i]#=}" text="$msg" ;;
            +*) file="${places[i]#+}$file" ;;
            @adr) file="${msg%% *}" file="docs/adr/${file#ADR-}" text="$msg" ;;
          esac
          bash_keys+=("$(key "$rule" "$file" "$n" "$text")")
        fi
        ;;
      "" | "Documentation checks FAILED:" | "Documentation checks passed." \
        | "Traceability check FAILED:" | "Traceability check passed ("*")." ) ;;
      *) [[ "$line" =~ ^[0-9]+\ problem\(s\)\ found\.$ ]] || diffs+=("bash output not understood: $line") ;;
    esac
  done < "$bash_out"
  while IFS= read -r line; do
    if [[ "$line" =~ ^([^:]+)(:([0-9]+))?:\ ([a-z0-9]+/[a-z0-9-]+):\  ]]; then
      rule="${BASH_REMATCH[4]}"
      [[ "$(mode_of "$rule")" == "unknown" ]] && diffs+=("nuc reports '$rule', which the table maps no message to")
      file="${BASH_REMATCH[1]}" n="${BASH_REMATCH[3]}" text="${line#"${BASH_REMATCH[0]}"}"
      # Where the table places the message by its ADR, nuc's file is cut to that ADR's key.
      for i in "${!patterns[@]}"; do
        if [[ "${rules[i]}" == "$rule" && "${places[i]}" == "@adr" && "$text" =~ ${patterns[i]} ]]; then
          if [[ "$file" =~ ^docs/adr/([0-9]{4})-[^/]*\.md$ || "$file" =~ ^docs/adr/(draft-[^/]+)\.md$ ]]; then
            file="docs/adr/${BASH_REMATCH[1]}"
          fi
          break
        fi
      done
      nuc_keys+=("$(key "$rule" "$file" "$n" "$text")")
    else
      diffs+=("nuc output not understood: $line")
    fi
  done < "$nuc_out"

  local only_bash only_nuc
  only_bash="$(comm -23 <(printf '%s\n' "${bash_keys[@]}" | sort -u) <(printf '%s\n' "${nuc_keys[@]}" | sort -u))"
  only_nuc="$(comm -13 <(printf '%s\n' "${bash_keys[@]}" | sort -u) <(printf '%s\n' "${nuc_keys[@]}" | sort -u))"
  while IFS= read -r line; do [[ -n "$line" ]] && diffs+=("only bash reports: ${line//$'\t'/ }"); done <<< "$only_bash"
  while IFS= read -r line; do [[ -n "$line" ]] && diffs+=("only nuc reports: ${line//$'\t'/ }"); done <<< "$only_nuc"

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
  compare "pair" "$2" "$3" "$4" "$5"
  exit
fi

corpus="$ROOT/tests/parity/corpus.toml" cache="$ROOT/.scratch/parity" nuc=""
variants="$ROOT/tests/parity/variants.tsv"
# The earliest template release a tree of the corpus may carry.
FLOOR=v0.5.0
while (($#)); do
  case "$1" in
    --corpus) corpus="${2:-}"; shift 2 || die "--corpus takes a file" ;;
    --cache) cache="${2:-}"; shift 2 || die "--cache takes a directory" ;;
    --nuc) nuc="${2:-}"; shift 2 || die "--nuc takes a binary" ;;
    --variants) [[ -n "${2:-}" ]] || die "--variants takes a file"; variants="$2"; shift 2 ;;
    *) die "unknown argument '$1'" ;;
  esac
done

# The corpus, one 'name url commit' line per repository.
mapfile -t repositories < <(awk '
  /^[[:space:]]*(#|$)/ { next }
  /^\[\[repository\]\]$/ { if (n++) print name, url, commit; name = url = commit = ""; next }
  match($0, /^(name|url|commit) = "[^"]*"$/) {
    value = $0; sub(/^[a-z]+ = "/, "", value); sub(/"$/, "", value)
    if ($1 == "name") name = value; else if ($1 == "url") url = value; else commit = value
    next
  }
  { print "malformed: " $0; exit }
  END { if (n) print name, url, commit }
' "$corpus" 2>/dev/null) || die "$corpus: unreadable"
((${#repositories[@]})) || die "$corpus: no repository"
for entry in "${repositories[@]}"; do
  read -r name url commit <<< "$entry"
  [[ "$name" =~ ^[a-z0-9-]+$ && -n "$url" && "$commit" =~ ^[0-9a-f]{40}$ ]] \
    || die "$corpus: an entry needs a name of [a-z0-9-], a url, and a 40-hex commit: '$entry'"
done

if [[ -z "$nuc" ]]; then
  (cd "$ROOT" && cargo build --locked -q) || { echo "Parity check: nuc could not be built" >&2; exit 1; }
  nuc="${CARGO_TARGET_DIR:-$ROOT/target}/debug/nuc"
fi

# The nuc.toml of the harness: every rule the table maps to, at the lowest minimum.
ported="$(printf '%s\n' "${rules[@]}" | grep -vx -- '-' | sort -u | sed 's/.*/"&"/' | paste -sd, -)"
config="minimum = \"0.0.0\"
rules = [$ported]"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
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

# at_least_floor <vX.Y.Z> — whether the release is FLOOR or later.
at_least_floor() {
  [[ -n "$1" ]] && [[ "$(printf '%s\n%s\n' "$FLOOR" "$1" | sort -V | head -1)" == "$FLOOR" ]]
}

# The environment of a pull request into main marked ready, in which both run, so that a check
# that reads where a change lands (check 16) speaks as it does in CI.
CI_ENV=(GITHUB_BASE_REF=main PULL_REQUEST_DRAFT=false)

# The bash scripts whose rules nuc ports, run in this order.
BASH_CHECKS=(scripts/check-docs.sh scripts/check-traceability.sh)

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
  env -i PATH="$PATH" HOME="$HOME" "${CI_ENV[@]}" "$nuc" check --repo "$dir" > "$out.nuc" 2> "$out.nuc.err"
  nuc_rc=$?
  compare "$label" "$out.bash" "$bash_rc" "$out.nuc" "$nuc_rc"
}

# apply_variant <copy> <path> <sed|delete|write|chomp> <argument> — apply the edit to the copy of a
# tree. It fails when the edit does not apply: the file is missing, or for 'write' already there,
# resolves outside the copy, the sed expression fails or changes nothing, or for 'chomp' the file
# does not end with a newline. 'write' creates the file with the argument as its one line, in a
# directory the copy already has; 'chomp' takes the newline off the file's last line, which sed
# cannot. sed runs sandboxed,
# so an expression runs no command and reads or writes no other file.
apply_variant() {
  local copy="$1" file="$1/$2"
  if [[ "$3" == write ]]; then
    [[ ! -e "$file" && ! -L "$file" && -d "$(dirname "$file")" ]] || return 1
    local parent
    parent="$(realpath "$(dirname "$file")")"
    [[ "$parent" == "$(realpath "$copy")" || "$parent" == "$(realpath "$copy")"/* ]] || return 1
    printf '%s\n' "$4" 2>/dev/null > "$file" || return 1
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
    msg="${line#  - }"
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
# 'delete' with '-', 'write' with a line, or 'chomp' with '-', and optionally the names of the corpus trees it is
# for, separated by commas, all separated by tabs. Without names it is for every tree. The path
# stays inside the tree and out of its .git; every ported rule brings one at least.
variant_names=() variant_rules=() variant_paths=() variant_ops=() variant_args=() variant_trees=()
[[ -r "$variants" ]] || die "$variants: unreadable"
while IFS=$'\t' read -r vname vrule vpath vop varg vtrees extra || [[ -n "$vname" ]]; do
  [[ -z "$vname" || "$vname" == \#* ]] && continue
  if ! [[ "$vname" =~ ^[a-z0-9-]+$ && -n "$vpath" && -z "${extra:-}" ]] \
    || ! [[ -z "${vtrees:-}" || "$vtrees" =~ ^[a-z0-9-]+(,[a-z0-9-]+)*$ ]] \
    || ! [[ ( "$vop" == sed && -n "$varg" ) || ( "$vop" =~ ^(delete|chomp)$ && "$varg" == - ) || ( "$vop" == write && -n "$varg" ) ]]; then
    die "$variants: the row '$vname' is not 'name<TAB>rule<TAB>path<TAB>sed|delete|write|chomp<TAB>argument[<TAB>tree,…]'"
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
for entry in "${repositories[@]}"; do
  read -r name url commit <<< "$entry"
  dir="$cache/$name"
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
  printf '%s\n' "$config" > "$dir/nuc.toml"
  grep -qx '/nuc.toml' "$dir/.git/info/exclude" 2>/dev/null || echo '/nuc.toml' >> "$dir/.git/info/exclude"

  run_pair "$name at ${commit:0:12}" "$dir" "$work/$name" || failed=1

  # Every variant on this tree, on a copy of it in a directory of its own, never on the cache.
  for i in "${!variant_names[@]}"; do
    # A variant that names its trees is for those alone.
    [[ -z "${variant_trees[i]}" || ",${variant_trees[i]}," == *",$name,"* ]] || continue
    label="$name, variant ${variant_names[i]}"
    vdir="$work/variants/$name/${variant_names[i]}"
    if ! { mkdir -p "$(dirname "$vdir")" && cp -a "$dir" "$vdir"; }; then
      echo "Parity check: $label — the tree could not be copied" >&2
      failed=1
      continue
    fi
    if ! apply_variant "$vdir" "${variant_paths[i]}" "${variant_ops[i]}" "${variant_args[i]}"; then
      echo "Parity check: $label — its edit of ${variant_paths[i]} does not apply to this tree" >&2
      failed=1
      rm -rf "$vdir"
      continue
    fi
    out="$work/$name.${variant_names[i]}"
    run_pair "$name at ${commit:0:12}, variant ${variant_names[i]}" "$vdir" "$out" || failed=1
    if ! fires "$work/$name.bash" "$out.bash" "${variant_rules[i]}"; then
      echo "Parity check: $label — bash reports no finding of ${variant_rules[i]} beyond the tree's own; the variant does not make its rule fire" >&2
      failed=1
    fi
    rm -rf "$vdir"
  done
done

if ((failed)); then
  echo "Parity check FAILED." >&2
  exit 1
fi
echo "Parity check passed (${#repositories[@]} repositories, ${#variant_names[@]} variants, rules: ${ported:-none})."
