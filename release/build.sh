#!/usr/bin/env bash
#
# Builds the release binary of nuc for one target into a directory, named as the release names
# it: nuc-<target>, with .exe on Windows (ADR-draft-distribution). The toolchain is the one
# rust-toolchain.toml pins; the target is added to it.
#
# Usage:
#     release/build.sh <target> <output directory>
# Exit code 0 when the binary is built, 1 otherwise.

set -euo pipefail

(($# == 2)) || { echo "usage: release/build.sh <target> <output directory>" >&2; exit 1; }
target="$1" out="$2"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

case "$target" in
  x86_64-unknown-linux-musl | aarch64-unknown-linux-musl | aarch64-apple-darwin | x86_64-pc-windows-msvc) ;;
  *) echo "release/build.sh: '$target' is no release target" >&2; exit 1 ;;
esac

cd "$ROOT"
rustup target add "$target"
cargo build --locked --release --target "$target"
exe=""
[[ "$target" == *windows* ]] && exe=".exe"
mkdir -p "$out"
cp "target/$target/release/nuc$exe" "$out/nuc-$target$exe"
echo "release/build.sh: $out/nuc-$target$exe"
