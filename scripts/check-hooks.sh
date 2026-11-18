#!/usr/bin/env bash
#
# Pin check of the git hooks (NUC-hooks-run-nuc §2, §3).
#
# The hooks under .githooks/ are the reference shims of a release of hivevm/nuc-tools, copied
# unchanged. The blob list .githooks/nuc.blobs names that release, the blob of each hook, and each
# hook/ rule of that release. Its lines:
#
#   release v<X.Y.Z>     the release the hooks come from, once
#   <blob> <hook>        one per hook of the five: the git blob of the file .githooks/<hook>
#   rule <rule ID>       one per hook/ rule of that release
#
# Lines that start with '#' and empty lines are no entries. The check fails on what would break a
# clause:
#
#   §2 Pin: the blob list exists and names a release, every hook of the five names a blob, it pins
#      no other file, and each hook it names is a file whose blob, as git would store it, is the
#      one listed. A file of the project's own pinned here would escape the checks that skip
#      pinned files (scripts/lib/pin.sh).
#   §3 nuc.toml: nuc.toml exists at the root, its minimum is no older than the release, and its
#      rules name every hook/ rule the list names. nuc.toml is read as the subset nuc reads: a
#      line 'minimum = "<X.Y.Z>"', and the quoted strings of 'rules = [...]', on one line or on
#      the lines up to the closing bracket, each '#' outside a string starting a comment to the
#      end of its line.
#
# git hashes each hook with its line endings converted, as it stores it, so the pin holds on every
# platform. The check reads no network and runs no nuc.
#
# Pure bash + git/coreutils/grep/awk. Usage:
#     scripts/check-hooks.sh [--root <dir>]
# The root defaults to this repository; scripts/test-check-hooks.sh passes fixtures.
# Exit code 0 when the check passes, 1 otherwise, and 2 on an argument it does not read or a root
# that is no directory.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/bash.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/bash.sh"

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/check.sh"

# shellcheck source-path=SCRIPTDIR source=lib/pin.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/pin.sh"

# shellcheck source-path=SCRIPTDIR source=lib/root.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/root.sh"
usage="scripts/check-hooks.sh [--root <dir>]"
enter_root "${BASH_SOURCE[0]}" "$usage" "$@"
rest_args "${BASH_SOURCE[0]}" "$usage" 0 "${ARGS[@]}"
LIST=".githooks/nuc.blobs"
TOML="nuc.toml"
VERSION_RE='^([0-9]+)\.([0-9]+)\.([0-9]+)$'

# older <a> <b> — whether the version a is older than b, both X.Y.Z.
older() {
  local -a x y
  local i
  [[ "$1" =~ $VERSION_RE ]] && x=("${BASH_REMATCH[@]:1}")
  [[ "$2" =~ $VERSION_RE ]] && y=("${BASH_REMATCH[@]:1}")
  for i in 0 1 2; do
    ((10#${x[i]} < 10#${y[i]})) && return 0
    ((10#${x[i]} > 10#${y[i]})) && return 1
  done
  return 1
}

release="" rules=()
declare -A listed=()
if [[ ! -f "$ROOT/$LIST" ]]; then
  add_error "$LIST: not found — the hooks are pinned by their blobs beside them (NUC-hooks-run-nuc §2) [hooks/pin:absent]"
else
  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%$'\r'}"
    [[ -z "$line" || "$line" == \#* ]] && continue
    read -r first second rest <<< "$line"
    if [[ "$first" == release && "$second" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ && -z "$rest" ]]; then
      release="${second#v}"
    elif [[ "$first" == rule && "$second" == hook/* && -z "$rest" ]]; then
      rules+=("$second")
    elif [[ "$first" =~ $PINNED_BLOB_RE && -z "$rest" ]] && pinned_hook "$second"; then
      listed["$second"]="$first"
    elif [[ "$first" =~ $PINNED_BLOB_RE && -n "$second" && -z "$rest" ]]; then
      add_error "$LIST: pins '$second', which is none of the five hooks — a file of the project's own is held by the checks, not by a pin (NUC-hooks-run-nuc §2) [hooks/pin:foreign]"
    else
      add_error "$LIST: '$line' is no entry — write 'release v<X.Y.Z>', '<blob> <hook>', or 'rule <rule ID>' (NUC-hooks-run-nuc §2) [hooks/pin:entry]"
    fi
  done < "$ROOT/$LIST"
  [[ -n "$release" ]] \
    || add_error "$LIST: names no release — the hooks come from one release of hivevm/nuc-tools (NUC-hooks-run-nuc §2) [hooks/pin:release]"
  for hook in "${PINNED_HOOKS[@]}"; do
    [[ -n "${listed[$hook]:-}" ]] \
      || add_error "$LIST: names no blob for the hook '$hook' — every hook is pinned (NUC-hooks-run-nuc §2) [hooks/pin:unlisted]"
  done
  mapfile -t names < <(printf '%s\n' "${!listed[@]}" | sort)
  for hook in "${names[@]}"; do
    if [[ ! -f "$ROOT/.githooks/$hook" ]]; then
      add_error ".githooks/$hook: not found, though $LIST pins it (NUC-hooks-run-nuc §2) [hooks/pin:missing]"
    elif ! blob="$(cd "$ROOT" && git hash-object -- ".githooks/$hook")"; then
      add_error ".githooks/$hook: git cannot hash it (NUC-hooks-run-nuc §2) [hooks/pin:missing]"
    elif [[ "$blob" != "${listed[$hook]}" ]]; then
      add_error ".githooks/$hook: differs from the shim of nuc-tools v$release that $LIST pins — a hook is the release's file unchanged; take it again from shims/ of that release (NUC-hooks-run-nuc §1, §2) [hooks/pin:differs]"
    fi
  done
fi

if [[ ! -f "$ROOT/$TOML" ]]; then
  add_error "$TOML: not found — it names the minimum nuc and the hook/ rules the hooks run (NUC-hooks-run-nuc §3) [hooks/toml:absent]"
else
  minimum="$(sed -nE 's/^[[:space:]]*minimum[[:space:]]*=[[:space:]]*"([^"]*)".*/\1/p' "$ROOT/$TOML" | head -n 1)"
  # The quoted strings from 'rules =' to the closing bracket, each comment dropped: a '#' outside a
  # string to the end of its line.
  on="$(awk '
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
    END { print text }' "$ROOT/$TOML" | grep -oE '"[^"]*"' | tr -d '"')"
  if [[ ! "$minimum" =~ $VERSION_RE ]]; then
    add_error "$TOML: names no minimum '<X.Y.Z>' (NUC-hooks-run-nuc §3) [hooks/toml:minimum]"
  elif [[ -n "$release" ]] && older "$minimum" "$release"; then
    add_error "$TOML: its minimum $minimum is older than nuc-tools v$release, the release of the hooks — a nuc older than its shims decides what no release tested (NUC-hooks-run-nuc §3) [hooks/toml:minimum]"
  fi
  for rule in "${rules[@]}"; do
    grep -qxF -- "$rule" <<< "$on" \
      || add_error "$TOML: its rules do not name '$rule' — with a nuc.toml its list governs which rules run, and no project removes a hook/ rule (NUC-hooks-run-nuc §3) [hooks/toml:rule]"
  done
fi

if ((${#errors[@]})); then
  echo "Hooks pin check FAILED:" >&2
  printf '  - %s\n' "${errors[@]}" >&2
  exit 1
fi

echo "Hooks pin check passed (${#listed[@]} hooks of nuc-tools v$release, ${#rules[@]} hook/ rules in $TOML). [hooks/check:passed]"
exit 0
