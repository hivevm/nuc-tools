#!/usr/bin/env bash
#
# Writes the release into the files the Action and the Feature install from
# (ADR-0014): SHA256SUMS and VERSION under release/pinned/ for the Action, and the
# installer, SHA256SUMS, and VERSION beside the Feature's install.sh, its devcontainer-feature.json
# set to the version. The release workflow commits the result as the release commit.
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

feature="$ROOT/feature/src/nuc"
json="$feature/devcontainer-feature.json"
grep -qE '^  "version": "[^"]*",$' "$json" \
  || { echo "release/pin.sh: $json has no '  \"version\": \"…\",' line" >&2; exit 1; }

mkdir -p "$ROOT/release/pinned"
for dir in "$ROOT/release/pinned" "$feature"; do
  cp "$sums" "$dir/SHA256SUMS"
  echo "$version" > "$dir/VERSION"
done
cp "$ROOT/release/install-nuc.sh" "$feature/install-nuc.sh"
sed -i.bak -E 's/^  "version": "[^"]*",$/  "version": "'"$version"'",/' "$json" && rm -f "$json.bak"
echo "release/pin.sh: release $version pinned in release/pinned/ and feature/src/nuc/"
