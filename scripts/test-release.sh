#!/usr/bin/env bash
#
# Self-test of the release tooling under release/ and of the release as the interface
# hivevm/features takes it from. Verifies: ADR-0025
#
# The cases use stub binaries served through file://, no network and no build:
# release/sums.sh writes one line per binary; release/install-nuc.sh installs the binary whose
# checksum SHA256SUMS names and refuses one that differs, one SHA256SUMS does not name, a commit
# without SHA256SUMS, and a VERSION that is no X.Y.Z, leaving nothing behind; release/pin.sh
# writes the release into a copy of the Action's files and refuses a version that is no X.Y.Z;
# release/version.sh refuses a tag other than the crate's version; release/move-major.sh moves the
# major tag to a later release and keeps it from an earlier one. The interface holds: the release
# workflow and build.sh build exactly its four targets, named nuc-<target>, SHA256SUMS holds one
# line '<sha256>  <asset>' each, and the installer, with uname stubbed, takes the asset of each
# platform; and no Dev Container Feature lies in or is published from this repository. That the
# installer rejects a real binary of each target that differs is the release-test workflow's.
#
# feature/ stands among the inputs so that a branch that brings it back runs this test.
# Inputs: release/ feature/ action.yml .github/workflows/ scripts/lib/
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
# shellcheck disable=SC2317,SC2329 # run through expect
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
# The interface hivevm/features relies on (ADR-0025): these targets, their asset names, and this
# form of SHA256SUMS, until a major release.
targets="aarch64-apple-darwin aarch64-unknown-linux-musl x86_64-pc-windows-msvc x86_64-unknown-linux-musl"
built="$(sed -n 's/^ *- { target: \([^,]*\),.*/\1/p' "$ROOT/.github/workflows/release.yml" | sort | tr '\n' ' ')"
check "the release workflow builds exactly the targets of the interface" test "$built" = "$targets "
cased="$(sed -n 's/^ *\([a-z0-9_ |-]*\)) ;;$/\1/p' "$ROOT/release/build.sh" | tr -d ' ' | tr '|' '\n' | sort | tr '\n' ' ')"
check "build.sh takes exactly the targets of the interface" test "$cased" = "$targets "
# shellcheck disable=SC2016 # the expression is build.sh's
check "build.sh names each binary nuc-<target>" grep -q ' "$out/nuc-$target$exe"$' "$ROOT/release/build.sh"
for target in $targets; do
  asset="nuc-$target"
  case "$target" in *windows*) asset="$asset.exe" ;; esac
  check "SHA256SUMS names $asset in the form '<sha256>  <asset>'" grep -Eq "^[0-9a-f]{64}  $asset\$" "$d/SHA256SUMS"
done

# The installer maps the platform uname names to its target, with uname stubbed on the PATH.
stub="$work/uname-stub"
mkdir -p "$stub"
# shellcheck disable=SC2016 # the stub expands them when it runs
printf '#!/bin/sh\ncase "$1" in -s) echo "$STUB_OS" ;; -m) echo "$STUB_ARCH" ;; esac\n' > "$stub/uname"
chmod +x "$stub/uname"
d="$(release mapping 1.2.3)"
base="$d"
command -v cygpath > /dev/null && base="/$(cygpath -m "$d")"
for platform in Linux/x86_64:x86_64-unknown-linux-musl Linux/aarch64:aarch64-unknown-linux-musl \
  Darwin/arm64:aarch64-apple-darwin; do
  os="${platform%%/*}" rest="${platform#*/}" arch="${rest%%:*}" target="${rest#*:}"
  dest="$work/bin-map-$target"
  expect pass "the installer takes $target on $os $arch" "nuc 1.2.3 installed" \
    env PATH="$stub:$PATH" STUB_OS="$os" STUB_ARCH="$arch" NUC_RELEASE_BASE="file://$base" \
    sh "$INSTALL" "$d/SHA256SUMS" "$d/VERSION" "$dest"
  expect pass "the binary installed on $os $arch is the one for $target" "stub nuc 1.2.3 $target" "$dest/nuc"
done
expect pass "the installer takes the Windows binary on MINGW64_NT x86_64" "nuc 1.2.3 installed" \
  env PATH="$stub:$PATH" STUB_OS=MINGW64_NT-10.0 STUB_ARCH=x86_64 NUC_RELEASE_BASE="file://$base" \
  sh "$INSTALL" "$d/SHA256SUMS" "$d/VERSION" "$work/bin-map-windows"
check "the Windows binary is installed as nuc.exe" test -f "$work/bin-map-windows/nuc.exe"
expect fail "sums.sh refuses a directory without a binary" "no nuc-* binary" bash "$ROOT/release/sums.sh" "$work"

# --- cases: the installer -----------------------------------------------------------------

d="$(release good 1.2.3)"
expect pass "the binary SHA256SUMS names is installed" "nuc 1.2.3 installed" \
  install "$d" "$d/SHA256SUMS" "$d/VERSION" "$work/bin-good"
expect pass "the installed binary runs" "stub nuc 1.2.3 $TARGET" "$work/bin-good/nuc"
expect pass "a destination with a backslash, as on Windows, installs the same" "nuc 1.2.3 installed" \
  install "$d" "$d/SHA256SUMS" "$d/VERSION" "$work/bin\\slash"
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

for bad in latest "1.2.3/../v1.2.3" "1x.2.3" "01.2.3" "1.2.3-rc1"; do
  d="$(release "bad-version-${bad//[^a-z0-9]/_}" 1.2.3)"
  echo "$bad" > "$d/VERSION"
  expect fail "a VERSION '$bad', which is no X.Y.Z, is refused" "not X.Y.Z" \
    install "$d" "$d/SHA256SUMS" "$d/VERSION" "$work/bin-bad-version"
done

d="$(release absent 1.2.3)"
rm "$d/v1.2.3/nuc-$TARGET"
expect fail "a binary the release lacks is refused" "could not fetch" \
  install "$d" "$d/SHA256SUMS" "$d/VERSION" "$work/bin-absent"

NUC_TARGET=sparc-sun-solaris expect fail "a target without a binary is refused" "SHA256SUMS names no" \
  install "$(release solaris 1.2.3)" "$work/solaris/SHA256SUMS" "$work/solaris/VERSION" "$work/bin-solaris"

# --- cases: pinning a release into the Action ----------------------------------------------

copy="$work/repo"
mkdir -p "$copy/release"
d="$(release pin 2.0.1)"
expect pass "pin.sh writes the release" "release 2.0.1 pinned" bash "$ROOT/release/pin.sh" 2.0.1 "$d/SHA256SUMS" "$copy"
check "the Action carries the checksums" cmp -s "$d/SHA256SUMS" "$copy/release/pinned/SHA256SUMS"
check "the Action carries the version" test "$(cat "$copy/release/pinned/VERSION")" = 2.0.1
expect pass "the pinned Action installs its release" "nuc 2.0.1 installed" \
  install "$d" "$copy/release/pinned/SHA256SUMS" "$copy/release/pinned/VERSION" "$work/bin-pinned"
expect fail "pin.sh refuses a version that is no X.Y.Z" "no version X.Y.Z" \
  bash "$ROOT/release/pin.sh" v2.0.1 "$d/SHA256SUMS" "$copy"

check "pin.sh writes nothing but release/pinned/" test "$(cd "$copy" && find . -type f | sort | tr '\n' ' ')" = \
  "./release/pinned/SHA256SUMS ./release/pinned/VERSION "
check "no Dev Container Feature lies in this repository" test ! -e "$ROOT/feature"
check "the workflows are there to search" test -d "$ROOT/.github/workflows"
# shellcheck disable=SC2016 # the expression is the inner shell's
check "no workflow publishes a Dev Container Feature" \
  sh -c '! grep -rqE "devcontainers/action|publish-features|features publish" "$1"' _ "$ROOT/.github/workflows"
check "the Action names the pinned files it installs from" grep -q 'release/pinned/SHA256SUMS' "$ROOT/action.yml"
# shellcheck disable=SC2016 # the expression is the Action's
check "the Action passes its arguments through the environment" grep -q 'NUC_ARGS: \${{ inputs.args }}' "$ROOT/action.yml"

# --- cases: the tag and the version of the crate -----------------------------------------

crate="$work/crate"
mkdir -p "$crate"
printf '[package]\nname = "x"\nversion = "1.4.0"\n\n[dependencies]\nversion = "9.9.9"\n' > "$crate/Cargo.toml"
expect pass "a tag of the crate's version passes and prints it" "1.4.0" bash "$ROOT/release/version.sh" v1.4.0 "$crate"
expect fail "a tag of another version is refused" "not the version 1.4.0" bash "$ROOT/release/version.sh" v1.4.1 "$crate"
for bad in 1.4.0 v01.4.0 v1.4 v1.4.0-rc1; do
  expect fail "a tag '$bad', which is no vX.Y.Z, is refused" "no tag vX.Y.Z" bash "$ROOT/release/version.sh" "$bad" "$crate"
done

# --- cases: moving the major tag ----------------------------------------------------------

# release_commit <repo> <version> — a commit carrying release/pinned/VERSION.
release_commit() {
  mkdir -p "$1/release/pinned"
  echo "$2" > "$1/release/pinned/VERSION"
  git -C "$1" add -A
  git -C "$1" -c user.name=F -c user.email=f@example.invalid -c core.hooksPath=/dev/null commit -q -m "pin $2"
}
repo="$work/tags"
mkdir -p "$repo" && git -C "$repo" init -q
move() { (cd "$repo" && bash "$ROOT/release/move-major.sh" "$1"); }
release_commit "$repo" 1.2.0
expect pass "a new major tag is moved, with nothing named before" "moved" move 1.2.0
check "the new major tag names the release commit" test "$(git -C "$repo" rev-parse v1)" = "$(git -C "$repo" rev-parse HEAD)"
first="$(git -C "$repo" rev-parse v1)"
release_commit "$repo" 1.3.0
check "a later release moves it and names what it named before" test "$(move 1.3.0 2>/dev/null)" = "moved $first"
later="$(git -C "$repo" rev-parse v1)"
release_commit "$repo" 1.2.4
expect pass "an earlier release keeps it" "kept" move 1.2.4
check "the kept major tag still names the later release" test "$(git -C "$repo" rev-parse v1)" = "$later"
release_commit "$repo" 2.0.0
expect pass "another major is a tag of its own" "moved" move 2.0.0
check "the first major is untouched by the second" test "$(git -C "$repo" rev-parse v1)" = "$later"
expect fail "move-major.sh refuses a version that is no X.Y.Z" "no version X.Y.Z" move v2.0.0

summary "Release tooling self-test"
