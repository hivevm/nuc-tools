#!/usr/bin/env bash
#
# Checks that a release tag names the version the crate carries, and prints the version
# (ADR-0014): a tag vX.Y.Z over a Cargo.toml of another version would publish a
# nuc that reports, and is held to, a version other than its release's.
#
# Usage:
#     release/version.sh <tag> [repository root]
# Exit code 0 when the tag is vX.Y.Z of the crate's version, 1 otherwise.

set -euo pipefail

(($# == 1 || $# == 2)) || { echo "usage: release/version.sh <tag> [root]" >&2; exit 1; }
tag="$1"
ROOT="$(cd "${2:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd)"
num='(0|[1-9][0-9]*)'
[[ "$tag" =~ ^v$num\.$num\.$num$ ]] \
  || { echo "release/version.sh: '$tag' is no tag vX.Y.Z" >&2; exit 1; }
crate="$(awk -F'"' '/^\[/ { section = $0 } section == "[package]" && /^version = "/ { print $2; exit }' "$ROOT/Cargo.toml")"
[[ "${tag#v}" == "$crate" ]] \
  || { echo "release/version.sh: the tag $tag is not the version $crate of Cargo.toml — move the version first" >&2; exit 1; }
echo "$crate"
