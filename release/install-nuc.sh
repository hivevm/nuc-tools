#!/bin/sh
#
# Installs the nuc of one release, verified against the checksums the caller carries
# (ADR-draft-distribution). The GitHub Action and the Dev Container Feature run it with the
# SHA256SUMS and the VERSION their release commit holds; a binary whose checksum differs is
# deleted and the installation fails.
#
# POSIX sh, curl or wget, and sha256sum or shasum, so that it runs in a container without bash.
# Usage:
#     install-nuc.sh <SHA256SUMS> <VERSION file> <destination directory>
# NUC_RELEASE_BASE replaces https://github.com/hivevm/nuc-tools/releases/download, the base the
# binary is fetched from, as scripts/test-release.sh does with a file:// URL; NUC_TARGET replaces
# the target read from uname. Neither can pass a binary the checksums do not name.
# Exit code 0 when nuc is installed, 1 otherwise.

set -eu

fail() { echo "install-nuc: $1" >&2; exit 1; }

[ $# -eq 3 ] || fail "usage: install-nuc.sh <SHA256SUMS> <VERSION file> <destination directory>"
sums="$1" version_file="$2" dest="$3"
if [ ! -f "$sums" ] || [ ! -f "$version_file" ]; then
  fail "no SHA256SUMS or VERSION beside the installer — this commit is no release commit; reference the major tag vN of a release"
fi
version="$(tr -d '[:space:]' < "$version_file")"
printf '%s\n' "$version" | grep -Eqx '(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)' \
  || fail "VERSION holds '$version', not X.Y.Z"

target="${NUC_TARGET:-}"
if [ -z "$target" ]; then
  os="$(uname -s)" arch="$(uname -m)"
  case "$os/$arch" in
    Linux/x86_64 | Linux/amd64) target=x86_64-unknown-linux-musl ;;
    Linux/aarch64 | Linux/arm64) target=aarch64-unknown-linux-musl ;;
    Darwin/arm64) target=aarch64-apple-darwin ;;
    MINGW*/x86_64 | MSYS*/x86_64 | CYGWIN*/x86_64) target=x86_64-pc-windows-msvc ;;
    *) fail "no release binary for $os on $arch" ;;
  esac
fi
asset="nuc-$target"
binary=nuc
case "$target" in *windows*) asset="$asset.exe" binary=nuc.exe ;; esac

expected="$(awk -v name="$asset" '$2 == name || $2 == "*" name { print $1 }' "$sums")"
[ -n "$expected" ] || fail "SHA256SUMS names no $asset"

base="${NUC_RELEASE_BASE:-https://github.com/hivevm/nuc-tools/releases/download}"
url="$base/v$version/$asset"
mkdir -p "$dest"
partial="$dest/.$binary.partial"
trap 'rm -f "$partial"' EXIT
if command -v curl > /dev/null 2>&1; then
  curl -fsSL --proto '=https,file' -o "$partial" "$url" || fail "could not fetch $url"
elif command -v wget > /dev/null 2>&1; then
  wget -q -O "$partial" "$url" || fail "could not fetch $url"
else
  fail "neither curl nor wget is on the PATH"
fi

# Through stdin, so that no path reaches the output: a backslash in one, as on Windows, makes
# sha256sum escape the line with a leading backslash.
if command -v sha256sum > /dev/null 2>&1; then
  actual="$(sha256sum < "$partial" | awk '{ print $1 }')"
elif command -v shasum > /dev/null 2>&1; then
  actual="$(shasum -a 256 < "$partial" | awk '{ print $1 }')"
else
  fail "neither sha256sum nor shasum is on the PATH"
fi
[ "$actual" = "$expected" ] || fail "$asset has the checksum $actual, not the $expected of release $version — not installed"

chmod 755 "$partial"
mv "$partial" "$dest/$binary"
echo "install-nuc: nuc $version installed as $dest/$binary"
