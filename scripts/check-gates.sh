#!/usr/bin/env bash
#
# Gates check for this repository (SEC-gates-run-released-nuc).
#
# This repository builds the tool that derived projects run as their gates; its own gates never
# run the 'nuc' built from the change they judge. The checks stay the template's bash, and the
# hooks run a released 'nuc' that CI installs from the checksums committed under release/gate/.
# This check reads the gates and what they run — scripts/check-all.sh, every hook under .githooks/,
# .github/workflows/checks.yml, every script under scripts/, and release/install-nuc.sh — and fails
# on each line that would run a build of the working tree (clause 5) or 'nuc' outside the hooks
# (clause 2):
#
#   1. 'cargo run', or its alias 'cargo r', which builds and runs the working tree.
#   2. 'cargo install' with '--path', which installs it.
#   3. A path under 'target/', or a 'debug/nuc' or 'release/nuc' under another target directory,
#      where cargo puts what it builds.
#   4. 'nuc' run as a command: at the start of a line or a 'run:' value; after ';', '&', '|',
#      '(', '$(', or a backquote; or after 'env' with its options and assignments, 'exec',
#      'command', 'time', or 'xargs'. The hooks .githooks/nuc.blobs pins are not read for it: they
#      are the template's shims, which run the released 'nuc' on the PATH (scripts/lib/pin.sh).
#      Nor is the one line of scripts/test-git-hooks.sh that asks 'nuc --version'.
#
# It also fails where CI would run a hook on a 'nuc' it did not install (clause 3):
#
#   5. A job of checks.yml that runs scripts/test-git-hooks.sh, scripts/test-ready-for-main.sh, or
#      scripts/test-normalize-remap.sh, each of which runs a hook, without a step before it that
#      runs 'release/install-nuc.sh release/gate/SHA256SUMS release/gate/VERSION'.
#   6. A nuc.toml whose minimum is newer than release/gate/VERSION, or a nuc.toml without that file.
#
# Four scripts are not read. scripts/check-build.sh and scripts/check-parity.sh are the tests of
# the build and run it; they judge nothing but the build. This check and its self-test write the
# patterns above into fixtures and run none of them. Full-line comments are dropped, so a gate may
# explain itself; a trailing comment is read as content. The files are read line by line, so a
# command assembled from variables is not seen; that is review.
#
# Pure bash + coreutils/grep. Usage:
#     scripts/check-gates.sh [--root <dir>]
# The root defaults to this repository; scripts/test-check-gates.sh passes fixtures.
# Exit code 0 when the check passes, 1 otherwise, 2 when the root does not exist or on an argument
# it does not read.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/bash.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/bash.sh"

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/check.sh"
# shellcheck source-path=SCRIPTDIR source=lib/pin.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/pin.sh"
# shellcheck source-path=SCRIPTDIR source=lib/root.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/root.sh"
usage="scripts/check-gates.sh [--root <dir>]"
enter_root "${BASH_SOURCE[0]}" "$usage" "$@"
rest_args "${BASH_SOURCE[0]}" "$usage" 0 "${ARGS[@]}"

NOT_READ=(scripts/check-build.sh scripts/check-parity.sh scripts/check-gates.sh scripts/test-check-gates.sh)

INSTALL='release/install-nuc.sh release/gate/SHA256SUMS release/gate/VERSION'
HOOK_TESTS='scripts/test-(git-hooks|ready-for-main|normalize-remap)\.sh'
declare -A pinned=()
while IFS= read -r f; do pinned["$f"]=1; done < <(pinned_files "$ROOT")

gates=()
[[ -f "$ROOT/scripts/check-all.sh" ]] && gates+=("$ROOT/scripts/check-all.sh")
[[ -f "$ROOT/release/install-nuc.sh" ]] && gates+=("$ROOT/release/install-nuc.sh")
shopt -s nullglob
for hook in "$ROOT/.githooks"/*; do [[ -f "$hook" ]] && gates+=("$hook"); done
shopt -u nullglob
[[ -f "$ROOT/.github/workflows/checks.yml" ]] && gates+=("$ROOT/.github/workflows/checks.yml")
if [[ -d "$ROOT/scripts" ]]; then
  while IFS= read -r -d '' script; do
    rel="${script#"$ROOT"/}"
    [[ "$rel" == scripts/check-all.sh ]] || _contains "$rel" "${NOT_READ[@]}" || gates+=("$script")
  done < <(find "$ROOT/scripts" -type f -name '*.sh' -print0 | sort -z)
fi

CARGO='(^|[^A-Za-z0-9_-])cargo([[:space:]]+[+-][^[:space:]]*)*[[:space:]]+'
CARGO_RUN="${CARGO}(run|r)([[:space:]]|\$)"
CARGO_INSTALL="${CARGO}install([[:space:]].*)?[[:space:]]--path([[:space:]=]|\$)"
TARGET='(^|[^A-Za-z0-9_.-])(\./)?target/|(^|[/"])(debug|release)/nuc(\.exe)?([^A-Za-z0-9_.-]|$)'
LAUNCHER='(env([[:space:]]+(-[^[:space:]]*|[A-Za-z_][A-Za-z0-9_]*=[^[:space:]]*))*|exec|command|time|xargs)'
NUC="(^|run:|[;&|(\`]|(^|[[:space:]])${LAUNCHER})[[:space:]]*(\\\$\\([[:space:]]*)?([^[:space:]]*/)?nuc(\\.exe)?([[:space:]]|\$)"

for gate in "${gates[@]}"; do
  rel="${gate#"$ROOT"/}"
  while IFS= read -r hit; do
    n="${hit%%:*}" text="${hit#*:}"
    [[ "$text" =~ $CARGO_RUN ]] && add_error "$rel:$n: 'cargo run' runs the working tree's build"
    [[ "$text" =~ $CARGO_INSTALL ]] && add_error "$rel:$n: 'cargo install --path' installs the working tree's build"
    [[ "$text" =~ $TARGET ]] && add_error "$rel:$n: a path under a target directory is the working tree's build"
    if [[ "$text" =~ $NUC && ! -v pinned["$rel"] ]] \
       && ! [[ "$rel" == scripts/test-git-hooks.sh && "$text" =~ ^have=\"\$\(nuc\ --version\  ]]; then
      add_error "$rel:$n: 'nuc' run by a gate outside the hooks, which run a released nuc"
    fi
  done < <(grep -nvE '^[[:space:]]*#' "$gate")
done

# Rule 5: a job that runs a self-test that runs a hook installs the released nuc before it.
workflow="$ROOT/.github/workflows/checks.yml"
if [[ -f "$workflow" ]]; then
  job="" installed=0
  while IFS= read -r hit; do
    n="${hit%%:*}" text="${hit#*:}"
    if [[ "$text" =~ ^\ \ ([A-Za-z0-9_-]+):[[:space:]]*$ ]]; then
      job="${BASH_REMATCH[1]}" installed=0
    elif [[ "$text" == *"$INSTALL"* ]]; then
      installed=1
    elif [[ -n "$job" && "$text" =~ $HOOK_TESTS ]] && ((!installed)); then
      add_error ".github/workflows/checks.yml:$n: the job '$job' runs ${BASH_REMATCH[0]}, which runs a hook, without installing the released nuc before it ($INSTALL)"
    fi
  done < <(grep -nvE '^[[:space:]]*#' "$workflow")
fi

# Rule 6: the release CI installs is no older than the minimum nuc.toml names.
version_re='^([0-9]+)\.([0-9]+)\.([0-9]+)$'
if [[ -f "$ROOT/nuc.toml" ]]; then
  minimum="$(sed -nE 's/^[[:space:]]*minimum[[:space:]]*=[[:space:]]*"([^"]*)".*/\1/p' "$ROOT/nuc.toml" | head -n 1)"
  gate_version="$(tr -d '[:space:]' < "$ROOT/release/gate/VERSION" 2>/dev/null)"
  if [[ ! -f "$ROOT/release/gate/VERSION" ]]; then
    add_error "release/gate/VERSION: not found, though nuc.toml names a minimum the hooks need — CI installs the release it names"
  elif [[ "$gate_version" =~ $version_re && "$minimum" =~ $version_re ]]; then
    read -r -a g <<< "$(sed -E "s/$version_re/\1 \2 \3/" <<< "$gate_version")"
    read -r -a m <<< "$(sed -E "s/$version_re/\1 \2 \3/" <<< "$minimum")"
    for i in 0 1 2; do
      ((10#${g[i]} > 10#${m[i]})) && break
      if ((10#${g[i]} < 10#${m[i]})); then
        add_error "release/gate/VERSION: $gate_version is older than the minimum $minimum of nuc.toml — the hooks in CI would refuse"
        break
      fi
    done
  else
    add_error "release/gate/VERSION or nuc.toml: no version X.Y.Z to compare ('$gate_version', '$minimum')"
  fi
fi

if ((${#errors[@]})); then
  echo "Gates check: ${#errors[@]} finding(s) — the gates run no build of the working tree:" >&2
  printf '  - %s\n' "${errors[@]}" >&2
  exit 1
fi
echo "Gates check passed (${#gates[@]} gates, none runs a build of the working tree)."
