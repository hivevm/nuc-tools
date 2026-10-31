#!/usr/bin/env bash
#
# Writes SHA256SUMS for the release binaries in a directory: one line '<sha256>  <name>' per
# nuc-* file, sorted by name (ADR-0014).
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
# sha256sum in Git Bash marks a binary-mode line with '*'; the file is the same on every platform.
if command -v sha256sum > /dev/null; then
  sha256sum "${binaries[@]}"
else
  shasum -a 256 "${binaries[@]}"
fi | sed -E 's/^([0-9a-f]{64}) [ *]/\1  /' | LC_ALL=C sort -k2 > SHA256SUMS
cat SHA256SUMS
