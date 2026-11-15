#!/usr/bin/env bash
#
# Comment width check for the project's own code.
#
# A comment line is at most 100 columns, as a line of Markdown prose is under the rule
# docs/prose-width of scripts/check-docs.sh, so that a diff shows the sentence that changed. Code
# lines are left to the formatters. The check reads every file git tracks or would track, ignored
# files left out, that ends in .rs, .sh, .yml, or .yaml, and fails on a comment line longer than
# 100 columns: a line whose first character after its indentation is '//' in a .rs file, or '#'
# in the others, the shebang of line 1 excepted. Characters are counted, not bytes, so an em dash
# is one column. A line whose text after its marker is one word, such as a link, cannot break and
# passes, and so does the '# Inputs:' line of a self-test, which scripts/check-all.sh reads as one
# line. The check reads lines, not the language: a line of a heredoc or of a string literal that
# starts with the marker counts as a comment.
#
# A file the template ships and the project carries unchanged is left out: a rewrap there is merge
# work at every take-up of a release, and its long lines are a finding about the template.
# scripts/template.blobs names those files, one '<blob> <path>' a line, under a first line
# '# Template release <version>'; a file is left out while its blob is the one the list names for
# its path, and held once the project changes it. A list whose first line is not that one fails.
# The version must be the one the README's 'Template release' line names, so that a take-up
# writes the list anew, from a clone that carries the template's tags:
#
#     git -C <clone> ls-tree -r <tag> | awk '{print $3, $4}' | grep -E '\.(rs|sh|ya?ml)$'
#
# Without the list, every file is held.
#
# Pure bash + git. Usage:
#     scripts/check-comment-width.sh [--root <dir>]
# The root defaults to this repository; scripts/test-check-comment-width.sh passes fixtures.
# Exit code 0 when the check passes, 1 otherwise, and 2 on an argument it does not read or a root
# that is no directory.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/bash.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/bash.sh"

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/check.sh"

# shellcheck source-path=SCRIPTDIR source=lib/root.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/root.sh"
usage="scripts/check-comment-width.sh [--root <dir>]"
enter_root "${BASH_SOURCE[0]}" "$usage" "$@"
rest_args "${BASH_SOURCE[0]}" "$usage" 0 "${ARGS[@]}"
LIST="scripts/template.blobs"

# columns <text> — sets COLUMNS_OF to the characters of the text: its bytes less the UTF-8
# continuation bytes, as scripts/check-docs.sh counts them.
columns() {
  local LC_ALL=C cont
  cont="${1//[!$'\x80'-$'\xbf']/}"
  COLUMNS_OF=$((${#1} - ${#cont}))
}

# The template's files, by path, and the release the list is of.
declare -A template=()
list_release=""
if [[ -f "$ROOT/$LIST" ]]; then
  first=1
  while IFS= read -r line || [[ -n "$line" ]]; do
    if ((first)); then
      first=0
      if [[ "$line" =~ ^#\ Template\ release\ (v[0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
        list_release="${BASH_REMATCH[1]}"
      else
        add_error "$LIST: the first line is not '# Template release <version>', so the list names no release [width/blobs:header]"
      fi
    elif [[ "$line" =~ ^([0-9a-f]{40,64})\ (.+)$ ]]; then
      template["${BASH_REMATCH[2]}"]="${BASH_REMATCH[1]}"
    fi
  done < "$ROOT/$LIST"
  # The first version on the README's first line that labels the Template release, in any form
  # docs/template-release reads: plain, in backticks, or as the text of a link.
  readme_release="$(grep -m1 -E 'Template release[:*]' "$ROOT/README.md" 2>/dev/null \
    | grep -m1 -oE 'v[0-9]+\.[0-9]+\.[0-9]+' | head -n 1)"
  if [[ -n "$list_release" && "$list_release" != "$readme_release" ]]; then
    add_error "$LIST: the list is of the template release ${list_release:-none}, the README names ${readme_release:-none} — write it anew from the release taken up [width/blobs:release]"
  fi
fi

# NUL-separated, so that a name beyond ASCII comes as it is, not quoted by core.quotePath.
if ! listed="$(git -C "$ROOT" ls-files -z --cached --others --exclude-standard \
                 -- '*.rs' '*.sh' '*.yml' '*.yaml' | tr '\0' '\n'; exit "${PIPESTATUS[0]}")"; then
  echo "check-comment-width: git cannot list the files of $ROOT" >&2
  exit 2
fi
files=()
[[ -n "$listed" ]] && mapfile -t files <<< "$listed"

# The blob of each file that the list names, in one call.
declare -A blob=()
named=()
for rel in "${files[@]}"; do
  [[ -v template["$rel"] && -f "$ROOT/$rel" ]] && named+=("$rel")
done
if ((${#named[@]})); then
  mapfile -t hashes < <(cd "$ROOT" && printf '%s\n' "${named[@]}" | git hash-object --stdin-paths)
  for i in "${!named[@]}"; do blob["${named[i]}"]="${hashes[i]}"; done
fi

held=0 skipped=0
for rel in "${files[@]}"; do
  [[ -f "$ROOT/$rel" ]] || continue
  if [[ -v blob["$rel"] && "${blob[$rel]}" == "${template[$rel]}" ]]; then
    skipped=$((skipped + 1))
    continue
  fi
  held=$((held + 1))
  marker='#'
  [[ "$rel" == *.rs ]] && marker='//'
  n=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    n=$((n + 1))
    ((${#line} > 100)) || continue
    text="${line#"${line%%[![:space:]]*}"}"
    [[ "$text" == "$marker"* ]] || continue
    [[ $n -eq 1 && "$text" == '#!'* ]] && continue
    # scripts/check-all.sh reads the Inputs of a self-test from its first such line alone.
    [[ "$rel" == *.sh && "$line" == '# Inputs:'* ]] && continue
    columns "$line"
    ((COLUMNS_OF > 100)) || continue
    # The text after the marker, its repeats, and a '!' of a doc comment: one word cannot break.
    body="${text#"${text%%[!/#!]*}"}"
    body="${body#"${body%%[![:space:]]*}"}"
    [[ "$body" == *[[:space:]]* ]] || continue
    add_error "$rel:$n: $COLUMNS_OF columns — wrap the comment at 100 [width/comment:wide]"
  done < "$ROOT/$rel"
done

if ((${#errors[@]})); then
  echo "Comment width check FAILED:" >&2
  printf '  - %s\n' "${errors[@]}" >&2
  exit 1
fi
echo "Comment width check passed ($held files held to 100 columns, $skipped of the template left out) [width/check:passed]"
exit 0
