#!/usr/bin/env bash
#
# Self-test of scripts/check-hooks.sh.
#
# Each case copies the repository's own hooks, their blob list, and its nuc.toml into a throwaway
# git repository, changes one thing, runs the check against it, and asserts the exit code and the
# key of the line that names the break. The copy passes. A hook edited by hand, one the list does
# not pin, one the list pins and the tree lacks, a file of the project's own or a path outside the
# hooks pinned, a list without a release or with a line it cannot read, and no list at all fail on
# the pin (clause 2). No nuc.toml, a minimum older than the
# release, none at all, and a rules list that lacks a hook/ rule, or holds it only in a comment,
# fail on nuc.toml (clause 3); a minimum newer than the release, a rules list on one line, further
# rules, and a '#' inside a string pass. A hook checked
# out with CRLF line endings under core.autocrlf passes as git stores it.
#
# Verifies: NUC-hooks-run-nuc
# Inputs: scripts/check-hooks.sh scripts/lib/ .githooks/ nuc.toml
#
# Usage:
#     scripts/test-check-hooks.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
CHECK="$ROOT/scripts/check-hooks.sh"

# repo <name> — a fixture repository with the repository's hooks, blob list, and nuc.toml, printed
# as its path.
repo() {
  local dir="$work/$1"
  mkdir -p "$dir"
  git -C "${dir:?}" init -q -b main
  cp -R "$ROOT/.githooks" "$dir/"
  cp "$ROOT/nuc.toml" "$dir/"
  echo "$dir"
}

# expect <exit code> <name> <dir> [key] — run the check against the fixture; a line of its output
# ends in the key, when one is given.
expect() {
  local want="$1" name="$2" dir="$3" key="${4:-}" out rc
  out="$(bash "$CHECK" --root "${dir:?}" 2>&1)"; rc=$?
  if ((rc == want)) && { [[ -z "$key" ]] || grep -qE "\[$key\]\$" <<< "$out"; }; then
    passed=$((passed + 1))
    return
  fi
  failed=$((failed + 1))
  echo "FAIL: $name (expected exit $want${key:+ and [$key]}, exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
}

# toml <dir> <minimum> <rules line...> — write nuc.toml with the minimum and the lines of its rules.
toml() {
  local dir="$1" minimum="$2"; shift 2
  { echo "minimum = \"$minimum\""; printf '%s\n' "$@"; } > "$dir/nuc.toml"
}

# rules_of <dir> — the hook/ rules the fixture's blob list names, quoted and comma-separated.
rules_of() {
  awk '$1 == "rule" { printf "%s\"%s\"", (n++ ? ", " : ""), $2 }' "$1/.githooks/nuc.blobs"
}

# release_of <dir> — the release the fixture's blob list names, without its 'v'.
release_of() { awk '$1 == "release" { sub(/^v/, "", $2); print $2 }' "$1/.githooks/nuc.blobs"; }

# --- clause 2, the pin ----------------------------------------------------------------------

d="$(repo copy)"
expect 0 "the repository's hooks, blob list, and nuc.toml pass" "$d" "hooks/check:passed"

d="$(repo edited)"
echo "# edited by hand" >> "$d/.githooks/pre-push"
expect 1 "a hook edited by hand" "$d" "hooks/pin:differs"
check "the finding names the hook" \
  grep -q '^  - .githooks/pre-push: ' <<< "$(bash "$CHECK" --root "$d" 2>&1)"

d="$(repo unlisted)"
grep -v ' pre-commit$' "$d/.githooks/nuc.blobs" > "$d/list" && mv "$d/list" "$d/.githooks/nuc.blobs"
expect 1 "a hook the list does not pin" "$d" "hooks/pin:unlisted"

d="$(repo missing)"
rm "$d/.githooks/reference-transaction"
expect 1 "a hook the list pins and the tree lacks" "$d" "hooks/pin:missing"

d="$(repo no-release)"
grep -v '^release ' "$d/.githooks/nuc.blobs" > "$d/list" && mv "$d/list" "$d/.githooks/nuc.blobs"
expect 1 "a list without a release" "$d" "hooks/pin:release"

d="$(repo bad-entry)"
echo "pre-push 1234" >> "$d/.githooks/nuc.blobs"
expect 1 "a line the list cannot read" "$d" "hooks/pin:entry"

d="$(repo foreign)"
echo "$(git -C "$d" hash-object "$d/.githooks/pre-push") own" >> "$d/.githooks/nuc.blobs"
cp "$d/.githooks/pre-push" "$d/.githooks/own"
expect 1 "a list that pins a file of the project's own, though its blob holds" "$d" "hooks/pin:foreign"
d="$(repo outside)"
echo "$(git -C "$d" hash-object "$d/.githooks/pre-push") ../pre-push" >> "$d/.githooks/nuc.blobs"
expect 1 "a list that pins a path outside the hooks" "$d" "hooks/pin:foreign"

d="$(repo no-list)"
rm "$d/.githooks/nuc.blobs"
expect 1 "no blob list" "$d" "hooks/pin:absent"

# git stores a file checked out with CRLF under core.autocrlf as LF, so the blob it hashes holds.
# The library of the self-tests turns core.autocrlf off for every git; this case sets no such
# variable, so the fixture's own setting holds, as Git for Windows sets it.
d="$(repo crlf)"
git -C "${d:?}" config core.autocrlf true
awk '{ printf "%s\r\n", $0 }' "$d/.githooks/pre-commit" > "$d/crlf" && cat "$d/crlf" > "$d/.githooks/pre-commit"
rm "$d/crlf"
GIT_CONFIG_COUNT=0 expect 0 "a hook checked out with CRLF under core.autocrlf passes as git stores it" "$d" "hooks/check:passed"

# --- clause 3, nuc.toml ---------------------------------------------------------------------

d="$(repo no-toml)"
rm "$d/nuc.toml"
expect 1 "no nuc.toml" "$d" "hooks/toml:absent"

d="$(repo old)"
toml "$d" 0.1.0 "rules = [$(rules_of "$d")]"
expect 1 "a minimum older than the release of the hooks" "$d" "hooks/toml:minimum"

d="$(repo no-minimum)"
toml "$d" "" "rules = [$(rules_of "$d")]"
expect 1 "a nuc.toml without a minimum" "$d" "hooks/toml:minimum"

d="$(repo newer)"
IFS=. read -r major minor patch <<< "$(release_of "$d")"
toml "$d" "$major.$minor.$((patch + 1))" "rules = [$(rules_of "$d")]"
expect 0 "a minimum newer than the release, on one line of rules" "$d" "hooks/check:passed"
toml "$d" "$major.$((minor + 1)).0" "rules = [$(rules_of "$d"), \"docs/prose-width\"]"
expect 0 "further rules besides the hook/ rules" "$d" "hooks/check:passed"
toml "$d" "$((major + 1)).0.0" "rules = [$(rules_of "$d")]"
expect 0 "a minimum of a newer major" "$d" "hooks/check:passed"

d="$(repo lacks-rule)"
toml "$d" "$(release_of "$d")" "rules = [$(rules_of "$d" | sed 's/"hook\/push-checks"//; s/, ,/,/; s/^, //; s/, $//')]"
expect 1 "a rules list that lacks a hook/ rule" "$d" "hooks/toml:rule"
check "the finding names the rule" \
  grep -q "'hook/push-checks'" <<< "$(bash "$CHECK" --root "$d" 2>&1)"

d="$(repo trailing-comment)"
toml "$d" "$(release_of "$d")" "rules = [" \
  "$(rules_of "$d" | sed 's/"hook\/push-checks"//; s/, ,/,/; s/^, //; s/, $//'), # \"hook/push-checks\"," "]"
expect 1 "a hook/ rule only in a comment after a rule" "$d" "hooks/toml:rule"
d="$(repo hash-in-string)"
toml "$d" "$(release_of "$d")" "rules = [$(rules_of "$d"), \"docs/x#y\"] # a comment"
expect 0 "a '#' inside a string starts no comment" "$d" "hooks/check:passed"

d="$(repo commented-rule)"
toml "$d" "$(release_of "$d")" "rules = [" "  # \"hook/push-checks\"," \
  "$(rules_of "$d" | sed 's/"hook\/push-checks"//; s/, ,/,/; s/^, //; s/, $//')" "]"
expect 1 "a hook/ rule only in a comment of the rules list" "$d" "hooks/toml:rule"

# --- summary -------------------------------------------------------------------------------

summary "Hooks pin check self-test"
