#!/usr/bin/env bash
#
# Self-test of the release tooling under release/ and the Feature's installer. Verifies:
# ADR-draft-distribution
#
# The cases use stub binaries served through file://, no network and no build:
# release/sums.sh writes one line per binary; release/install-nuc.sh installs the binary whose
# checksum SHA256SUMS names and refuses one that differs, one SHA256SUMS does not name, a commit
# without SHA256SUMS, and a VERSION that is no X.Y.Z, leaving nothing behind; release/pin.sh
# writes the release into a copy of the Action's and the Feature's files and refuses a version
# that is no X.Y.Z; and the Feature's install.sh refuses to run without a pinned release. That
# the installers reject a real binary of each target that differs is the release-test workflow's.
#
# Inputs: release/ feature/ action.yml scripts/lib/
#
# Usage:
#     scripts/test-release.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
INSTALL="$ROOT/release/install-nuc.sh"
TARGET=x86_64-unknown-linux-musl

# expect <pass|fail> <name> <needle> <command...> — run it; its output contains the needle.
expect() {
  local want="$1" name="$2" needle="$3" out rc
  shift 3
  out="$("$@" 2>&1)"; rc=$?
  if [[ ( "$want" == pass && $rc -eq 0 || "$want" == fail && $rc -ne 0 ) && "$out" == *"$needle"* ]]; then
    passed=$((passed + 1))
    return
  fi
  failed=$((failed + 1))
  echo "FAIL: $name (expected $want with \"$needle\", exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
}

# release <name> <version> — a release directory with stub binaries under v<version>/, its
# SHA256SUMS and VERSION beside them, printed as its path.
release() {
  local dir="$work/$1" t
  mkdir -p "$dir/v$2"
  for t in x86_64-unknown-linux-musl aarch64-unknown-linux-musl aarch64-apple-darwin; do
    printf '#!/bin/sh\necho "stub nuc %s %s"\n' "$2" "$t" > "$dir/v$2/nuc-$t"
  done
  printf 'MZ stub\n' > "$dir/v$2/nuc-x86_64-pc-windows-msvc.exe"
  bash "$ROOT/release/sums.sh" "$dir/v$2" > /dev/null
  cp "$dir/v$2/SHA256SUMS" "$dir/SHA256SUMS"
  echo "$2" > "$dir/VERSION"
  echo "$dir"
}

# install <release directory> <installer arguments...> — the installer, fetching from the
# directory through file://; on Windows the Git Bash path becomes a URL through its drive letter.
# shellcheck disable=SC2317 # run through expect
install() {
  local dir="$1" base="$1"; shift
  command -v cygpath > /dev/null && base="/$(cygpath -m "$dir")"
  env NUC_RELEASE_BASE="file://$base" NUC_TARGET="${NUC_TARGET:-$TARGET}" sh "$INSTALL" "$@"
}

# --- cases: the checksums -----------------------------------------------------------------

d="$(release sums 1.2.3)"
lines="$(wc -l < "$d/SHA256SUMS")"
check "sums.sh writes one line per binary" test "$lines" -eq 4
check "sums.sh names the Windows binary with its .exe" grep -q '  nuc-x86_64-pc-windows-msvc.exe$' "$d/SHA256SUMS"
expect fail "sums.sh refuses a directory without a binary" "no nuc-* binary" bash "$ROOT/release/sums.sh" "$work"

# --- cases: the installer -----------------------------------------------------------------

d="$(release good 1.2.3)"
expect pass "the binary SHA256SUMS names is installed" "nuc 1.2.3 installed" \
  install "$d" "$d/SHA256SUMS" "$d/VERSION" "$work/bin-good"
expect pass "the installed binary runs" "stub nuc 1.2.3 $TARGET" "$work/bin-good/nuc"
NUC_TARGET=x86_64-pc-windows-msvc expect pass "on Windows the binary keeps its .exe" "nuc.exe" \
  install "$d" "$d/SHA256SUMS" "$d/VERSION" "$work/bin-windows"

d="$(release tampered 1.2.3)"
echo 'echo replaced' >> "$d/v1.2.3/nuc-$TARGET"
expect fail "a binary whose checksum differs is refused" "not installed" \
  install "$d" "$d/SHA256SUMS" "$d/VERSION" "$work/bin-tampered"
check "nothing of the refused binary is left" test -z "$(ls -A "$work/bin-tampered" 2>/dev/null)"

d="$(release unnamed 1.2.3)"
grep -v "nuc-$TARGET\$" "$d/SHA256SUMS" > "$d/SHA256SUMS.new" && mv "$d/SHA256SUMS.new" "$d/SHA256SUMS"
expect fail "a binary SHA256SUMS does not name is refused" "SHA256SUMS names no nuc-$TARGET" \
  install "$d" "$d/SHA256SUMS" "$d/VERSION" "$work/bin-unnamed"

d="$(release missing 1.2.3)"
expect fail "a commit without SHA256SUMS installs nothing" "no release commit" \
  install "$d" "$d/no-SHA256SUMS" "$d/VERSION" "$work/bin-missing"

d="$(release bad-version 1.2.3)"
echo "latest" > "$d/VERSION"
expect fail "a VERSION that is no X.Y.Z is refused" "not X.Y.Z" \
  install "$d" "$d/SHA256SUMS" "$d/VERSION" "$work/bin-bad-version"

d="$(release absent 1.2.3)"
rm "$d/v1.2.3/nuc-$TARGET"
expect fail "a binary the release lacks is refused" "could not fetch" \
  install "$d" "$d/SHA256SUMS" "$d/VERSION" "$work/bin-absent"

NUC_TARGET=sparc-sun-solaris expect fail "a target without a binary is refused" "SHA256SUMS names no" \
  install "$(release solaris 1.2.3)" "$work/solaris/SHA256SUMS" "$work/solaris/VERSION" "$work/bin-solaris"

# --- cases: pinning a release into the Action and the Feature ------------------------------

copy="$work/repo"
mkdir -p "$copy/release" "$copy/feature/src"
cp "$ROOT/release/install-nuc.sh" "$copy/release/"
cp -r "$ROOT/feature/src/nuc" "$copy/feature/src/"
d="$(release pin 2.0.1)"
expect pass "pin.sh writes the release" "release 2.0.1 pinned" bash "$ROOT/release/pin.sh" 2.0.1 "$d/SHA256SUMS" "$copy"
check "the Action carries the checksums" cmp -s "$d/SHA256SUMS" "$copy/release/pinned/SHA256SUMS"
check "the Action carries the version" test "$(cat "$copy/release/pinned/VERSION")" = 2.0.1
check "the Feature carries the checksums" cmp -s "$d/SHA256SUMS" "$copy/feature/src/nuc/SHA256SUMS"
check "the Feature carries the installer" cmp -s "$INSTALL" "$copy/feature/src/nuc/install-nuc.sh"
check "the Feature's version is the release" grep -q '^  "version": "2.0.1",$' "$copy/feature/src/nuc/devcontainer-feature.json"
expect pass "the pinned Action installs its release" "nuc 2.0.1 installed" \
  install "$d" "$copy/release/pinned/SHA256SUMS" "$copy/release/pinned/VERSION" "$work/bin-pinned"
expect fail "pin.sh refuses a version that is no X.Y.Z" "no version X.Y.Z" \
  bash "$ROOT/release/pin.sh" v2.0.1 "$d/SHA256SUMS" "$copy"

expect fail "the Feature refuses to install without a pinned release" "carries no release" \
  sh "$ROOT/feature/src/nuc/install.sh"
check "the Action names the pinned files it installs from" grep -q 'release/pinned/SHA256SUMS' "$ROOT/action.yml"
# shellcheck disable=SC2016 # the expression is the Action's
check "the Action passes its arguments through the environment" grep -q 'NUC_ARGS: \${{ inputs.args }}' "$ROOT/action.yml"

summary "Release tooling self-test"
