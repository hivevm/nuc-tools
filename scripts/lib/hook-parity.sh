# shellcheck shell=bash
# The scenario harness of the hooks (ADR-0022), sourced by scripts/check-parity.sh.
#
# Each scenario of tests/parity/hook-scenarios.sh builds a fixture repository through git and runs
# one git command in it, once with the bash hook of each template tree given and once with the
# reference shim under shims/ and nuc on the PATH. The fixture's README carries the release line
# of the tree, which the bash ignores; the nuc side runs twice, without nuc.toml and with one that
# turns on every hook/ rule the table maps to. A scenario marked bare runs without README and
# without nuc.toml, against the newest tree alone.
#
# Compared per scenario and tree: the exit status of the git command; the hook's lines, those that
# start with the hook's name and a colon, each mapped by tests/parity/hooks.tsv to a hook/ rule
# ID and the ref it guards, and the findings nuc writes in the line form of ADR-0020, as sets; and
# the lines of the stand-in checks, verbatim. Git's own lines are not compared. A hook's line no
# row maps, and a rule of nuc the table maps no line to, is a difference.
#
# Each scenario's bash side must end as every case it names expects, pass or refuse, so that a
# setup that no longer reaches its state fails rather than agrees on nothing.
#
# Coverage: the 'expect' names of each tree's scripts/test-git-hooks.sh must each be named by a
# scenario or listed in tests/parity/hook-cases.tsv with a reason; an uncovered name, a listed
# name no tree has, a scenario that names a case no tree has, and no tree at all fail.
#
# PARITY_HOOK_SCENARIOS, PARITY_HOOK_TABLE, and PARITY_HOOK_CASES name other files, for
# scripts/test-check-parity.sh.
#
# Usage: hook_parity <nuc> <work dir> <tree>... — returns 1 on any difference, 2 on a malformed
# table, case list, or scenario file.

HOOK_SCENARIOS="${PARITY_HOOK_SCENARIOS:-$ROOT/tests/parity/hook-scenarios.sh}"
HOOK_TABLE="${PARITY_HOOK_TABLE:-$ROOT/tests/parity/hooks.tsv}"
HOOK_CASES="${PARITY_HOOK_CASES:-$ROOT/tests/parity/hook-cases.tsv}"
SHIMS="$ROOT/shims"

# The scenarios, filled by hook_scenario as the scenario file is sourced.
hs_names=() hs_hooks=() hs_cases=() hs_setups=() hs_args=() hs_bare=()

# hook_scenario [--bare] <name> <hook> <cases, separated by '|'> <setup function> <git args...>
# — register a scenario; the setup function gets the fixture's path, and its git commands run
# without hooks.
hook_scenario() {
  local bare=0
  [[ "$1" == --bare ]] && { bare=1; shift; }
  (($# >= 5)) || { echo "Parity check: $HOOK_SCENARIOS: a scenario needs a name, a hook, cases, a setup, and a git command: '$*'" >&2; return 2; }
  hs_names+=("$1") hs_hooks+=("$2") hs_cases+=("$3") hs_setups+=("$4") hs_bare+=("$bare")
  shift 4
  hs_args+=("$(printf '%q ' "$@")")
}

# hp_git <dir> <git args...> — git in the fixture with an identity and without hooks.
hp_git() {
  local dir="$1"; shift
  git -C "${dir:?}" -c user.name=t -c user.email=t@example.invalid -c core.hooksPath=/dev/null "$@"
}

# hp_checks <dir> <exit code> — give the fixture a scripts/check-all.sh that exits as told.
hp_checks() {
  mkdir -p "$1/scripts"
  # shellcheck disable=SC2016 # the stand-in's own expansion
  printf '#!/usr/bin/env bash\necho "check output for the test, args: $*"\nexit %s\n' "$2" > "$1/scripts/check-all.sh"
}

# hp_docs <dir> <exit code> — give the fixture a scripts/check-docs.sh that exits as told.
hp_docs() {
  mkdir -p "$1/scripts"
  printf '#!/usr/bin/env bash\necho "docs output for the test"\nexit %s\n' "$2" > "$1/scripts/check-docs.sh"
}

# hp_repo <dir> — a fixture with one commit on main, the Dev Container's merge settings, and a
# bare remote 'origin' beside it, as the template's hook self-test builds its own.
hp_repo() {
  mkdir -p "$1"
  git -C "${1:?}" init -q -b main
  hp_git "$1" commit -q --allow-empty -m init
  hp_git "$1" config merge.ff false
  hp_git "$1" config pull.ff only
  git init -q --bare "$1.git"
  hp_git "$1" remote add origin "$1.git"
}

# hp_lines <hook> <output> — the compared lines of one run, one key per line, sorted: 'rule' with
# the rule ID and the ref, or 'out' with a stand-in's line; and 'unknown' with a line that is the
# hook's but no row maps, or a rule of nuc the table does not know.
hp_lines() {
  local hook="$1" line i matched
  while IFS= read -r line; do
    if [[ "$line" == "check output for the test"* || "$line" == "docs output for the test"* ]]; then
      printf 'out\t%s\n' "$line"
    elif [[ "$line" == "$hook: "* ]]; then
      matched=""
      for i in "${!hp_patterns[@]}"; do
        if [[ "$line" =~ ${hp_patterns[i]} ]]; then
          matched="${hp_ids[i]}"$'\t'"${hp_fields[i]}"
          break
        fi
      done
      if [[ -n "$matched" ]]; then printf 'rule\t%s\n' "$matched"; else printf 'unknown\t%s\n' "$line"; fi
    elif [[ "$line" =~ ^([^\ :]+):\ (hook/[a-z0-9-]+):\  ]]; then
      if _contains "${BASH_REMATCH[2]}" "${hp_ids[@]}"; then
        printf 'rule\t%s\t%s\n' "${BASH_REMATCH[2]}" "${BASH_REMATCH[1]}"
      else
        printf 'unknown\tnuc reports %s, which the table maps no line to\n' "${BASH_REMATCH[2]}"
      fi
    fi
  done < "$2" | sort -u
}

# hp_run <scenario> <hooks dir> <release or ''> <nuc.toml or ''> <path> — build the scenario's
# fixture at the path, run its git command with the hooks, and leave the output in <path>.out and
# the exit status in <path>.rc.
hp_run() {
  local s="$1" hooks="$2" release="$3" config="$4" dir="$5" rc
  hp_repo "$dir"
  "${hs_setups[s]}" "$dir" > /dev/null 2>&1
  [[ -n "$release" ]] && printf '# Fixture\n\n- **Template release:** %s\n' "$release" > "$dir/README.md"
  [[ -n "$config" ]] && printf '%s\n' "$config" > "$dir/nuc.toml"
  git -C "$dir" config core.hooksPath "$hooks"
  local -a args
  eval "args=(${hs_args[s]})"
  (cd "$dir" && env -i PATH="$hp_bin:$PATH" HOME="$HOME" LC_ALL=C \
    git -c user.name=t -c user.email=t@example.invalid "${args[@]}") > "$dir.out" 2>&1
  rc=$?
  echo "$rc" > "$dir.rc"
}

# hp_reached <scenario> <bash path> <label> — whether the bash side ends as each case the scenario
# names expects, pass or refuse: a setup that no longer reaches its state would agree on nothing.
hp_reached() {
  local s="$1" rc c ok=0
  rc="$(cat "$2.rc")"
  IFS='|' read -ra named <<< "${hs_cases[s]}"
  for c in "${named[@]}"; do
    case "${hp_kinds[$c]:-}" in
      pass) ((rc == 0)) && continue ;;
      refuse) ((rc != 0)) && continue ;;
      *) continue ;;
    esac
    echo "Parity check: $3 — the bash side exits $rc, where the self-test case '$c' expects ${hp_kinds[$c]}; the scenario does not reach its case" >&2
    ok=1
  done
  return "$ok"
}

# hp_compare <label> <hook> <bash path> <nuc path> — print the differences of one pair and return
# 1 when there are any.
hp_compare() {
  local label="$1" hook="$2" b="$3" n="$4" line
  local -a diffs=()
  local b_rc n_rc
  b_rc="$(cat "$b.rc")" n_rc="$(cat "$n.rc")"
  ((b_rc == n_rc)) || diffs+=("bash exits $b_rc, nuc exits $n_rc")
  hp_lines "$hook" "$b.out" > "$b.keys"
  hp_lines "$hook" "$n.out" > "$n.keys"
  while IFS=$'\t' read -r kind rest; do
    [[ "$kind" == unknown ]] && diffs+=("bash: a line of the hook the table does not know: $rest")
  done < "$b.keys"
  while IFS=$'\t' read -r kind rest; do
    [[ "$kind" == unknown ]] && diffs+=("nuc: a line of the hook the table does not know: $rest")
  done < "$n.keys"
  while IFS= read -r line; do
    [[ -n "$line" && "$line" != unknown* ]] && diffs+=("only bash: ${line//$'\t'/ }")
  done <<< "$(comm -23 "$b.keys" "$n.keys")"
  while IFS= read -r line; do
    [[ -n "$line" && "$line" != unknown* ]] && diffs+=("only nuc: ${line//$'\t'/ }")
  done <<< "$(comm -13 "$b.keys" "$n.keys")"
  if ((${#diffs[@]})); then
    echo "Parity check: $label — ${#diffs[@]} difference(s):" >&2
    printf '  - %s\n' "${diffs[@]}" >&2
    return 1
  fi
}

# hook_parity <nuc> <work dir> <tree>...
hook_parity() {
  local nuc="$1" hw="$2/hooks"; shift 2
  local -a trees=("$@")
  local failed=0 pattern id field extra name reason

  hp_patterns=() hp_ids=() hp_fields=()
  while IFS=$'\t' read -r pattern id field extra || [[ -n "$pattern" ]]; do
    [[ -z "$pattern" || "$pattern" == \#* ]] && continue
    if ! [[ "$id" =~ ^hook/[a-z0-9-]+$ && -n "$field" && -z "${extra:-}" ]]; then
      echo "Parity check: $HOOK_TABLE: the row '$pattern' is not 'pattern<TAB>hook/<rule><TAB>ref|-'" >&2
      return 2
    fi
    hp_patterns+=("$pattern") hp_ids+=("$id") hp_fields+=("$field")
  done < "$HOOK_TABLE" || { echo "Parity check: $HOOK_TABLE: unreadable" >&2; return 2; }

  local -a listed=() listed_reasons=()
  while IFS=$'\t' read -r name reason extra || [[ -n "$name" ]]; do
    [[ -z "$name" || "$name" == \#* ]] && continue
    if [[ -z "${reason:-}" || -n "${extra:-}" ]]; then
      echo "Parity check: $HOOK_CASES: the row '$name' is not 'case<TAB>reason'" >&2
      return 2
    fi
    listed+=("$name") listed_reasons+=("$reason")
  done < "$HOOK_CASES" || { echo "Parity check: $HOOK_CASES: unreadable" >&2; return 2; }

  hs_names=() hs_hooks=() hs_cases=() hs_setups=() hs_args=() hs_bare=()
  # shellcheck source=/dev/null # the scenario file is data of the harness
  source "$HOOK_SCENARIOS" || { echo "Parity check: $HOOK_SCENARIOS: malformed" >&2; return 2; }
  ((${#hs_names[@]})) || { echo "Parity check: $HOOK_SCENARIOS: no scenario" >&2; return 2; }

  if ((${#trees[@]} == 0)); then
    echo "Parity check: hooks — the corpus pins no tree of the template, so no hook is compared" >&2
    return 1
  fi

  # The 'expect' names of each tree's self-test, and the newest tree.
  local tree release newest="" newest_release="" t_names kind
  local -a all_names=()
  # What each case expects, pass or refuse, by its name.
  declare -gA hp_kinds=()
  for tree in "${trees[@]}"; do
    release="$(release_of "$tree/README.md")"
    if [[ ! -f "$tree/scripts/test-git-hooks.sh" ]]; then
      echo "Parity check: hooks — the tree at ${release:-$tree} has no scripts/test-git-hooks.sh" >&2
      failed=1
      continue
    fi
    t_names="$(grep -oE '^expect (pass|refuse) "[^"]*"' "$tree/scripts/test-git-hooks.sh" \
      | sed -E 's/^expect (pass|refuse) "(.*)"$/\1\t\2/')"
    while IFS=$'\t' read -r kind name; do
      [[ -n "$name" ]] || continue
      all_names+=("$name")
      hp_kinds["$name"]="$kind"
    done <<< "$t_names"
    if [[ -z "$newest" || "$(printf '%s\n%s\n' "$newest_release" "$release" | sort -V | tail -1)" == "$release" ]]; then
      newest="$tree" newest_release="$release"
    fi
  done
  mapfile -t all_names < <(printf '%s\n' "${all_names[@]}" | sed '/^$/d' | sort -u)

  local -a covered=()
  local s c
  for s in "${!hs_names[@]}"; do
    IFS='|' read -ra named <<< "${hs_cases[s]}"
    for c in "${named[@]}"; do
      if ! _contains "$c" "${all_names[@]}"; then
        echo "Parity check: hook scenario ${hs_names[s]} — it names the case '$c', which no tree's self-test has" >&2
        failed=1
      fi
      covered+=("$c")
    done
  done
  for c in "${all_names[@]}"; do
    _contains "$c" "${covered[@]}" "${listed[@]}" && continue
    echo "Parity check: hooks — the self-test case '$c' is covered by no scenario and not listed in ${HOOK_CASES#"$ROOT/"}" >&2
    failed=1
  done
  for c in "${listed[@]}"; do
    _contains "$c" "${all_names[@]}" && continue
    echo "Parity check: hooks — ${HOOK_CASES#"$ROOT/"} lists the case '$c', which no tree's self-test has" >&2
    failed=1
  done

  # nuc on the PATH of the nuc side, and the nuc.toml that turns on every hook/ rule.
  hp_bin="$hw/bin"
  mkdir -p "$hp_bin"
  ln -sf "$(realpath "$nuc")" "$hp_bin/nuc"
  local ids config
  ids="$(printf '%s\n' "${hp_ids[@]}" | sort -u | sed 's/.*/"&"/' | paste -sd, -)"
  config="minimum = \"0.0.0\"
rules = [$ids]"

  local n=0 hook label b side
  for s in "${!hs_names[@]}"; do
    hook="${hs_hooks[s]}"
    if [[ ! -f "$SHIMS/$hook" ]]; then
      echo "Parity check: hook scenario ${hs_names[s]} — there is no shim shims/$hook" >&2
      failed=1
      continue
    fi
    mkdir -p "$hw/shim-$hook"
    cp "$SHIMS/$hook" "$hw/shim-$hook/$hook"
    for tree in "${trees[@]}"; do
      release="$(release_of "$tree/README.md")"
      ((hs_bare[s])) && [[ "$tree" != "$newest" ]] && continue
      if [[ ! -f "$tree/.githooks/$hook" ]]; then
        echo "Parity check: hooks — the tree at $release has no .githooks/$hook" >&2
        failed=1
        continue
      fi
      mkdir -p "$hw/bash-$release-$hook"
      cp "$tree/.githooks/$hook" "$hw/bash-$release-$hook/$hook"
      n=$((n + 1))
      b="$hw/$n.bash"
      if ((hs_bare[s])); then
        hp_run "$s" "$hw/bash-$release-$hook" "" "" "$b"
        hp_reached "$s" "$b" "hook scenario ${hs_names[s]} at $release, bare" || failed=1
        hp_run "$s" "$hw/shim-$hook" "" "" "$hw/$n.nuc"
        hp_compare "hook scenario ${hs_names[s]} at $release, bare" "$hook" "$b" "$hw/$n.nuc" || failed=1
        continue
      fi
      hp_run "$s" "$hw/bash-$release-$hook" "$release" "" "$b"
      hp_reached "$s" "$b" "hook scenario ${hs_names[s]} at $release" || failed=1
      for side in plain configured; do
        if [[ "$side" == plain ]]; then
          hp_run "$s" "$hw/shim-$hook" "$release" "" "$hw/$n.$side"
        else
          hp_run "$s" "$hw/shim-$hook" "$release" "$config" "$hw/$n.$side"
        fi
        label="hook scenario ${hs_names[s]} at $release"
        [[ "$side" == configured ]] && label+=", with nuc.toml"
        hp_compare "$label" "$hook" "$b" "$hw/$n.$side" || failed=1
      done
    done
  done
  ((failed)) && return 1
  echo "Parity check: hooks — ${#hs_names[@]} scenario(s) on ${#trees[@]} tree(s), the same from bash and nuc; ${#listed[@]} self-test case(s) listed as no scenario."
}
