#!/usr/bin/env bash
#
# Build, test, and lint check for this repository (ADR-0008).
#
# Runs the commands the README names under Build, Test & Run, in this order: the format check,
# clippy with every warning an error, the build, and the tests, each with the lock file as it
# stands. It stops at the first that fails. The toolchain is the one rust-toolchain.toml pins,
# which rustup installs on the first call; without cargo on PATH the check fails, it never skips.
# Cargo writes only under target/, which git ignores and no other check reads, so check-all.sh runs
# it beside the others.
#
# Usage:
#     scripts/check-build.sh        (or: bash scripts/check-build.sh)
# Exit code 0 when every command passes, 1 otherwise.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

if ! command -v cargo >/dev/null 2>&1; then
  echo "Build check FAILED: cargo not found on PATH — install rustup (https://rustup.rs); it reads rust-toolchain.toml." >&2
  exit 1
fi

step() {
  local label="$1"
  shift
  if ! "$@"; then
    echo "Build check FAILED: $label ($*)." >&2
    exit 1
  fi
}

step "format" cargo fmt --check
step "lint" cargo clippy --all-targets --locked -- -D warnings
step "build" cargo build --locked
step "tests" cargo test --locked

echo "Build check passed (format, lint, build, tests)."
