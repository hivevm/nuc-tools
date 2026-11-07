#!/usr/bin/env bash
#
# Writes the release into the files the Action installs from (ADR-0025): SHA256SUMS and VERSION
# under release/pinned/. The release workflow commits the result as the release commit.
#
# Usage:
#     release/pin.sh <version X.Y.Z> <SHA256SUMS> [repository root]
# Exit code 0 when every file is written, 1 otherwise.

set -euo pipefail

(($# == 2 || $# == 3)) || { echo "usage: release/pin.sh <version> <SHA256SUMS> [root]" >&2; exit 1; }
version="$1" sums="$2"
ROOT="$(cd "${3:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd)"
[[ "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] \
  || { echo "release/pin.sh: '$version' is no version X.Y.Z" >&2; exit 1; }
[[ -s "$sums" ]] || { echo "release/pin.sh: '$sums' is empty or missing" >&2; exit 1; }

mkdir -p "$ROOT/release/pinned"
cp "$sums" "$ROOT/release/pinned/SHA256SUMS"
echo "$version" > "$ROOT/release/pinned/VERSION"
echo "release/pin.sh: release $version pinned in release/pinned/"
