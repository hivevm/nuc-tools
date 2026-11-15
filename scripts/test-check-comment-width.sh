#!/usr/bin/env bash
#
# Self-test of scripts/check-comment-width.sh.
#
# Each case builds a git repository under a temporary directory — a README that names the template
# release, a file of the template with a long comment and the list that names its blob, and a file
# of the project with short comments — runs the check against it, and asserts the exit code and,
# for a failing case, the key and the data of the finding. The cases stand in
# scripts/fixtures/check-comment-width.txt, which says its format, so that a port of the check is
# held to them. The table is read twice, the second time against a copy of the check that rewords
# its messages.
#
# Inputs: scripts/check-comment-width.sh scripts/lib/ scripts/fixtures/check-comment-width.txt
#
# Usage:
#     scripts/test-check-comment-width.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
CHECK="$ROOT/scripts/check-comment-width.sh"
TABLE="$ROOT/scripts/fixtures/check-comment-width.txt"

# filled <n> [fill] — text of exactly n characters: words of the fill character, a space between.
filled() {
  local n="$1" fill="${2:-w}" out="" i
  for ((i = 0; i < n; i++)); do
    if ((i % 8 == 7 && i < n - 1)); then out+=" "; else out+="$fill"; fi
  done
  printf '%s' "$out"
}

# comment_line <marker> <n> <indent> [fill] — a comment line of exactly n characters.
comment_line() {
  local marker="$1" n="$2" indent="$3" fill="${4:-w}" head
  head="$(printf '%*s' "$indent" '')$marker "
  printf '%s%s\n' "$head" "$(filled $((n - ${#head})) "$fill")"
}

# blobs <dir> <release> <path>... — write the list of the template's blobs.
blobs() {
  local dir="$1" release="$2" path
  shift 2
  {
    echo "# Template release $release"
    for path in "$@"; do echo "$(git -C "$dir" hash-object "$path") $path"; done
  } > "$dir/scripts/template.blobs"
}

# repo <name> — a fixture that passes the check, printed as its path.
repo() {
  local dir="$work/$1"
  mkdir -p "$dir/scripts" "$dir/src"
  git init -q "$dir"
  echo '- **Template release:** v0.11.0' > "$dir/README.md"
  { echo '#!/usr/bin/env bash'; comment_line '#' 120 0; echo 'echo tpl'; } > "$dir/scripts/tpl.sh"
  printf '// A short comment.\nfn main() {}\n' > "$dir/src/own.rs"
  blobs "$dir" v0.11.0 scripts/tpl.sh
  echo "$dir"
}

# table_cases <prefix> — run each case of the table against $CHECK.
table_cases() {
  local n=0 line want needle name tree s op arg d path text release out
  local -a args
  while IFS= read -r line; do
    [[ "$line" =~ ^(pass|fail) ]] || continue
    n=$((n + 1))
    IFS=$'\x1f' read -r want needle name tree <<< "${line// | /$'\x1f'}"
    d="$(repo "$1-$n")" release=v0.11.0
    [[ "$tree" != - ]] || tree=""
    while IFS= read -r s; do
      [[ -n "$s" ]] || continue
      read -r op arg <<< "$s"
      read -r -a args <<< "$arg"
      case "$op" in
        rm) rm -f "${d:?}/$arg" ;;
        comment | word | dash)
          path="${args[0]}"
          mkdir -p "$(dirname "$d/$path")"
          case "$op" in
            comment) comment_line "${args[1]}" "${args[2]}" "${args[3]:-0}" ;;
            word) printf '%s %s\n' "${args[1]}" \
                    "$(printf '%*s' $((args[2] - ${#args[1]} - 1)) '' | tr ' ' u)" ;;
            dash) comment_line "${args[1]}" "${args[2]}" 0 '—' ;;
          esac >> "$d/$path" ;;
        code)
          mkdir -p "$(dirname "$d/${args[0]}")"
          printf 'let x = "%s";\n' "$(filled $((args[1] - 10)))" >> "$d/${args[0]}" ;;
        write)
          path="${arg%% *}" text="${s#write "${arg%% *}" }"
          mkdir -p "$(dirname "$d/$path")" && printf '%s' "${text//\\n/$'\n'}" > "$d/$path" ;;
        blobs) blobs "$d" "$release" "${args[@]}" ;;
        release) release="${args[0]}"; echo "- **Template release:** $release" > "$d/README.md" ;;
        *) echo "fixture table: unknown step '$op'" >&2; exit 1 ;;
      esac
    done <<< "${tree//; /$'\n'}"
    out="$(bash "$CHECK" --root "$d" 2>&1)"
    table_case "$name" "$want" "$needle" "$?" "$out"
  done < "$TABLE"
  ((n > 0)) || { echo "FAIL: no case read from $TABLE" >&2; failed=$((failed + 1)); }
}
table_cases table

# --- the key decides, not the wording ------------------------------------------------------

copy="$work/reworded/scripts"
mkdir -p "$copy"
cp -R "$ROOT/scripts/lib" "$copy/"
sed -E '/\[width\//s/ (a|is|no|not|the|of) / reworded /g' "$CHECK" > "$copy/check-comment-width.sh"
keyed="$(grep -c '\[width/' "$CHECK")"
check "a copy of the check rewords each message with a key" \
  test "$keyed" -gt 0 -a "$keyed" -eq "$(grep -c ' reworded .*\[width/' "$copy/check-comment-width.sh")"
CHECK="$copy/check-comment-width.sh"
table_cases reworded
CHECK="$ROOT/scripts/check-comment-width.sh"

# --- arguments -----------------------------------------------------------------------------

# refused <argument...> — the check stops with exit 2 and names the first argument.
# shellcheck disable=SC2317,SC2329 # run by check
refused() {
  local out rc
  out="$(bash "$CHECK" "$@" 2>&1)"; rc=$?
  ((rc == 2)) && [[ "$out" == *"unknown argument '$1'"* ]]
}
check "a root given as a first argument is refused" refused "$work"
check "an argument the check does not read is refused" refused --verbose

summary "Comment width self-test"
