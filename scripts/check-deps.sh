#!/usr/bin/env bash
#
# Dependency audit for this repository (ADR-draft-toolchain).
#
# Runs 'cargo deny check' against deny.toml — advisories, bans, licenses, and sources — and
# 'cargo audit' against Cargo.lock. Both fetch the RustSec advisory database, so the check needs
# the network; without it, or without either tool on PATH, it fails and never passes on a
# database it could not read (Q-1). An ignored advisory carries its reason in the file that
# ignores it.
#
# Usage:
#     scripts/check-deps.sh        (or: bash scripts/check-deps.sh)
# Exit code 0 when both pass, 1 otherwise.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

for tool in cargo-deny cargo-audit; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "Dependency audit FAILED: $tool not found on PATH — 'cargo install --locked $tool'." >&2
    exit 1
  fi
done

if ! cargo deny check; then
  echo "Dependency audit FAILED: cargo deny check." >&2
  exit 1
fi
if ! cargo audit; then
  echo "Dependency audit FAILED: cargo audit." >&2
  exit 1
fi

echo "Dependency audit passed (cargo deny, cargo audit)."
