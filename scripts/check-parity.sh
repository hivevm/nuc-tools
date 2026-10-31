#!/usr/bin/env bash
#
# Parity harness (ADR-0012, Q-2).
#
# Runs the bash checks a repository of the corpus carries and nuc on the same tree, and fails on
# any difference in their findings or their exit status. The corpus is tests/parity/corpus.toml:
# each repository by URL and commit, fetched into an ignored cache under .scratch/parity/; a
# cache that cannot be filled fails the harness, never skips the repository (Q-1).
#
# For each repository:
#   1. The bash scripts with a ported rule run in the tree, with the environment cleared to PATH,
#      HOME, and LC_ALL=C: today scripts/check-docs.sh, whose findings are its '  - ' lines.
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
#     scripts/check-parity.sh [--corpus <file>] [--cache <dir>] [--nuc <binary>]
#     scripts/check-parity.sh --compare <bash output> <bash exit> <nuc output> <nuc exit>
# --nuc runs the given binary instead of building one; --compare compares one pair of outputs,
# for scripts/test-check-parity.sh. PARITY_TABLE names another normalization table.
# Exit code 0 when every repository agrees, 1 on a difference or a cache that cannot be filled,
# 2 on a malformed corpus or table or an unknown argument.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TABLE="${PARITY_TABLE:-$ROOT/tests/parity/normalize.tsv}"

# Git hands a hook its repository through variables that would bind the cache's git commands to
# this repository; they are not the harness's (scripts/lib/selftest.sh does the same).
# shellcheck disable=SC2046 # one variable name per word
unset $(git rev-parse --local-env-vars)

die() { echo "Parity check: $1" >&2; exit 2; }

patterns=() rules=() modes=()
while IFS=$'\t' read -r pattern rule mode extra; do
  [[ -z "$pattern" || "$pattern" == \#* ]] && continue
  [[ -n "$rule" && ( "$mode" == "line" || "$mode" == "-" ) && -z "${extra:-}" ]] \
    || die "$TABLE: the row '$pattern' is not 'pattern<TAB>rule ID<TAB>line|-'"
  patterns+=("$pattern") rules+=("$rule") modes+=("$mode")
done < "$TABLE" || die "$TABLE: unreadable"

# mode_of <rule> — whether the comparison keeps the line of the rule's findings: line or '-'.
mode_of() {
  local i
  for i in "${!rules[@]}"; do [[ "${rules[i]}" == "$1" ]] && { echo "${modes[i]}"; return; }; done
  echo "unknown"
}

# key <rule> <file> <line> — the normalized finding, its line kept only where the table says so.
key() {
  local line="$3"
  [[ "$(mode_of "$1")" == "line" ]] || line=""
  printf '%s\t%s\t%s\n' "$1" "$2" "$line"
}

# compare <label> <bash output> <bash exit> <nuc output> <nuc exit> — print the differences of
# one pair and return 1 when there are any.
compare() {
  local label="$1" bash_out="$2" bash_rc="$3" nuc_out="$4" nuc_rc="$5"
  local -a diffs=() bash_keys=() nuc_keys=()
  local line msg i rule file n bash_any=0
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
          if [[ "${modes[i]}" == "line" && "$file" =~ ^(.*):([0-9]+)$ ]]; then
            file="${BASH_REMATCH[1]}" n="${BASH_REMATCH[2]}"
          fi
          bash_keys+=("$(key "$rule" "$file" "$n")")
        fi
        ;;
      "" | "Documentation checks FAILED:" | "Documentation checks passed.") ;;
      *) [[ "$line" =~ ^[0-9]+\ problem\(s\)\ found\.$ ]] || diffs+=("bash output not understood: $line") ;;
    esac
  done < "$bash_out"
  while IFS= read -r line; do
    if [[ "$line" =~ ^([^:]+)(:([0-9]+))?:\ ([a-z0-9]+/[a-z0-9-]+):\  ]]; then
      rule="${BASH_REMATCH[4]}"
      [[ "$(mode_of "$rule")" == "unknown" ]] && diffs+=("nuc reports '$rule', which the table maps no message to")
      nuc_keys+=("$(key "$rule" "${BASH_REMATCH[1]}" "${BASH_REMATCH[3]}")")
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
while (($#)); do
  case "$1" in
    --corpus) corpus="${2:-}"; shift 2 || die "--corpus takes a file" ;;
    --cache) cache="${2:-}"; shift 2 || die "--cache takes a directory" ;;
    --nuc) nuc="${2:-}"; shift 2 || die "--nuc takes a binary" ;;
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

failed=0
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
  printf '%s\n' "$config" > "$dir/nuc.toml"
  grep -qx '/nuc.toml' "$dir/.git/info/exclude" 2>/dev/null || echo '/nuc.toml' >> "$dir/.git/info/exclude"

  (cd "$dir" && env -i PATH="$PATH" HOME="$HOME" LC_ALL=C bash scripts/check-docs.sh) > "$work/$name.bash" 2>&1
  bash_rc=$?
  env -i PATH="$PATH" HOME="$HOME" "$nuc" check --repo "$dir" > "$work/$name.nuc" 2> "$work/$name.nuc.err"
  nuc_rc=$?
  compare "$name at ${commit:0:12}" "$work/$name.bash" "$bash_rc" "$work/$name.nuc" "$nuc_rc" || failed=1
done

if ((failed)); then
  echo "Parity check FAILED." >&2
  exit 1
fi
echo "Parity check passed (${#repositories[@]} repositories, rules: ${ported:-none})."
