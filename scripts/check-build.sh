#!/usr/bin/env bash
#
# Build, test, and lint check for this repository (OPS-rust-stable-behind-scripts).
#
# Runs the commands the README names under Build, Test & Run, in this order: the format check,
# clippy with every warning an error, the build, and the tests, each with the lock file as it
# stands. It stops at the first that fails. The toolchain is the one rust-toolchain.toml pins,
# which rustup installs on the first call; without cargo on PATH the check fails, it never skips.
# Cargo writes only under target/, which git ignores and no other check reads, so check-all.sh runs
# it beside the others.
#
# Usage:
#     scripts/check-build.sh [--root <dir>]        (or: bash scripts/check-build.sh)
# Exit code 0 when every command passes, 1 otherwise, 2 when the root does not exist or on an
# argument it does not read. It sources no bash floor: the portability job runs it on macOS, whose
# bash is 3.2, so the empty arguments expand in the form bash 3.2 reads under 'set -u'.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/root.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/root.sh"
usage="scripts/check-build.sh [--root <dir>]"
enter_root "${BASH_SOURCE[0]}" "$usage" "$@"
rest_args "${BASH_SOURCE[0]}" "$usage" 0 ${ARGS[@]+"${ARGS[@]}"}

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
