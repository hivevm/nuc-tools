#!/usr/bin/env bash
#
# The one ShellCheck the shell lint runs with. The Dev Container installs it on creation
# (postCreateCommand of devcontainer.json), and the shell job of .github/workflows/checks.yml
# before it lints, so a script clean in the container is clean in CI. The version and the
# checksums of its release archives are written here and nowhere else: a change of version is a
# change of this file.
#
# The archive comes from the release page of koalaman/shellcheck and is refused unless its SHA-256
# is the one below; nothing is installed then. The checksums were read from the release's own
# asset digests and agreed with the archives downloaded on 2026-11-15.
#
# Usage:
#     .devcontainer/install-shellcheck.sh <dir>     install shellcheck into <dir>, through sudo
#                                                   where <dir> is not writable
#     .devcontainer/install-shellcheck.sh --verify  check that the shellcheck on PATH is this one
#     .devcontainer/install-shellcheck.sh --version print the pinned version
# Exit code 0 on success, 1 on a refused archive, a failed download, or another version on PATH,
# and 2 on an argument it does not read or a machine it has no archive for.
#
# SHELLCHECK_BASE_URL replaces the release page, for the self-test; the checksum still decides.
#
# Self-test: scripts/test-install-shellcheck.sh

set -euo pipefail

VERSION=0.11.0
SHA256_x86_64=8c3be12b05d5c177a04c29e3c78ce89ac86f1595681cab149b65b97c4e227198
SHA256_aarch64=12b331c1d2db6b9eb13cfca64306b1b157a86eb69db83023e261eaa7e7c14588

usage="usage: install-shellcheck.sh <dir> | --verify | --version"

case "${1:-}" in
  --version)
    echo "$VERSION"
    exit 0
    ;;
  --verify)
    found="$(shellcheck --version 2>/dev/null | sed -n 's/^version: //p')" || found=""
    if [[ "$found" != "$VERSION" ]]; then
      echo "install-shellcheck: shellcheck on PATH is '${found:-absent}', not the pinned $VERSION" >&2
      exit 1
    fi
    echo "install-shellcheck: shellcheck $VERSION"
    exit 0
    ;;
  "" | -*)
    echo "install-shellcheck: $usage" >&2
    exit 2
    ;;
esac
[[ $# -eq 1 ]] || { echo "install-shellcheck: $usage" >&2; exit 2; }
dir="$1"

arch="$(uname -m)"
case "$arch" in
  x86_64 | amd64) arch=x86_64; sha="$SHA256_x86_64" ;;
  aarch64 | arm64) arch=aarch64; sha="$SHA256_aarch64" ;;
  *)
    echo "install-shellcheck: no pinned archive for the machine '$arch'" >&2
    exit 2
    ;;
esac

base="${SHELLCHECK_BASE_URL:-https://github.com/koalaman/shellcheck/releases/download}"
name="shellcheck-v$VERSION.linux.$arch.tar.xz"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

if ! curl -fsSL --retry 3 -o "$tmp/$name" "$base/v$VERSION/$name"; then
  echo "install-shellcheck: could not download $base/v$VERSION/$name" >&2
  exit 1
fi
got="$(sha256sum "$tmp/$name")"
got="${got%% *}"
if [[ "$got" != "$sha" ]]; then
  echo "install-shellcheck: $name has the SHA-256 $got, not the pinned $sha; nothing installed" >&2
  exit 1
fi

tar -xJf "$tmp/$name" -C "$tmp"
mkdir -p "$dir" 2>/dev/null || sudo mkdir -p "$dir"
if [[ -w "$dir" ]]; then
  install -m 0755 "$tmp/shellcheck-v$VERSION/shellcheck" "$dir/shellcheck"
else
  sudo install -m 0755 "$tmp/shellcheck-v$VERSION/shellcheck" "$dir/shellcheck"
fi
echo "install-shellcheck: shellcheck $VERSION in $dir"
