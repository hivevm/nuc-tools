#!/usr/bin/env bash
#
# Moves the major tag vN to the release commit at HEAD, unless vN already names a later release
# (ADR-draft-distribution): a release of an older line, cut after a newer one, must not move the
# projects on @vN back. Prints 'kept', or 'moved <object>' with the object vN named before, empty
# when vN is new, so that the push can lease on it and a release that moved vN meanwhile is not
# overwritten.
#
# Usage:
#     release/move-major.sh <version X.Y.Z>     (in the repository, the release commit at HEAD)
# Exit code 0 when vN is moved or kept, 1 otherwise.

set -euo pipefail

(($# == 1)) || { echo "usage: release/move-major.sh <version>" >&2; exit 1; }
version="$1"
[[ "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] \
  || { echo "release/move-major.sh: '$version' is no version X.Y.Z" >&2; exit 1; }
major="v${version%%.*}"
before="$(git rev-parse -q --verify "refs/tags/$major" || true)"
if [[ -n "$before" ]]; then
  current="$(git show "$major:release/pinned/VERSION" 2>/dev/null | tr -d '[:space:]' || true)"
  if [[ -n "$current" && "$(printf '%s\n%s\n' "$current" "$version" | sort -V | tail -1)" != "$version" ]]; then
    echo "release/move-major.sh: $major names $current, later than $version — kept" >&2
    echo "kept"
    exit 0
  fi
fi
git tag -f "$major" HEAD > /dev/null
echo "release/move-major.sh: $major moved to $version" >&2
echo "moved $before"
