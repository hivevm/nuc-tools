#!/usr/bin/env bash
#
# Writes SHA256SUMS for the release binaries in a directory: one line '<sha256>  <name>' per
# nuc-* file, sorted by name (ADR-draft-distribution).
#
# Usage:
#     release/sums.sh <directory>
# Exit code 0 when the file is written, 1 when the directory holds no binary.

set -euo pipefail

(($# == 1)) || { echo "usage: release/sums.sh <directory>" >&2; exit 1; }
cd "$1"
shopt -s nullglob
binaries=(nuc-*)
((${#binaries[@]})) || { echo "release/sums.sh: no nuc-* binary in $1" >&2; exit 1; }
if command -v sha256sum > /dev/null; then
  sha256sum "${binaries[@]}" | sort -k2 > SHA256SUMS
else
  shasum -a 256 "${binaries[@]}" | sort -k2 > SHA256SUMS
fi
cat SHA256SUMS
