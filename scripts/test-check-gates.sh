#!/usr/bin/env bash
#
# Self-test of scripts/check-gates.sh.
# Verifies: SEC-gates-run-released-nuc
#
# Each case builds a fixture under a temporary directory — a check-all.sh, a hook, and a
# checks.yml that run the bash checks and scripts/check-build.sh — runs the check against it, and
# asserts the exit code and, for a failing case, the message that names the violation. The
# baseline passes; the cases that fail have one gate, or a script under scripts/ a gate runs, run
# a build of the working tree; those that pass show what is no build or not read. The rest hold
# the hooks on a released nuc: a pinned hook and the version query of the hooks' self-test may run
# nuc, the installer is read as a gate, a job that runs a hook installs nuc from release/gate/
# first, and release/gate/VERSION is no older than the minimum of nuc.toml.
#
# Inputs: scripts/check-gates.sh scripts/lib/
#
# Usage:
#     scripts/test-check-gates.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
CHECK="$ROOT/scripts/check-gates.sh"

# gates <name> — a fixture whose gates pass, printed as its path.
gates() {
  local dir="$work/$1"
  mkdir -p "$dir/scripts" "$dir/.githooks" "$dir/.github/workflows"
  {
    echo '#!/usr/bin/env bash'
    echo '# A comment may name cargo run, target/release/nuc, and nuc check.'
    echo 'run "Documentation" scripts/check-docs.sh'
    echo 'run "Build, test, and lint" scripts/check-build.sh'
  } > "$dir/scripts/check-all.sh"
  {
    echo '#!/usr/bin/env bash'
    echo 'exec bash scripts/check-all.sh --changed'
  } > "$dir/.githooks/pre-push"
  {
    echo 'jobs:'
    echo '  build:'
    echo '    steps:'
    echo '      - uses: hivevm/nuc-tools@0123456789abcdef0123456789abcdef01234567 # v1.0.0'
    echo '      - name: Run the nuc-tools build and the targets of the tests'
    echo '        run: bash scripts/check-build.sh'
    echo '      - run: cargo install --locked cargo-deny@0.20.2'
  } > "$dir/.github/workflows/checks.yml"
  echo "$dir"
}

# expect <pass|fail> <name> <dir> [needle] — run the check; when a needle is given, its output
# must contain it.
expect() {
  local want="$1" name="$2" dir="$3" needle="${4:-}"
  local out rc
  out="$(bash "$CHECK" --root "$dir" 2>&1)"; rc=$?
  if [[ "$want" == "pass" && $rc -eq 0 && ( -z "$needle" || "$out" == *"$needle"* ) ]] \
     || [[ "$want" == "fail" && $rc -ne 0 && ( -z "$needle" || "$out" == *"$needle"* ) ]]; then
    passed=$((passed + 1))
    return
  fi
  failed=$((failed + 1))
  echo "FAIL: $name (expected $want${needle:+ with needle \"$needle\"}, exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
}

# expect_refused <name> [args...] — run the check with the arguments; it must stop with exit 2 and
# name the argument it does not read: a root given as a first argument is refused, not read as
# the root (NUC-checks-are-bash §9).
expect_refused() {
  local name="$1" out rc
  shift
  out="$(bash "$CHECK" "$@" 2>&1)"; rc=$?
  if ((rc == 2)) && [[ "$out" == *"unknown argument '$1'"* ]]; then
    passed=$((passed + 1))
    return
  fi
  failed=$((failed + 1))
  echo "FAIL: $name (expected exit 2 naming '$1', exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
}

# --- cases ---------------------------------------------------------------------------------

d="$(gates baseline)"
expect pass "gates that run only the bash checks and the build's own tests pass" "$d" "3 gates"

d="$(gates cargo-run)"
echo '        run: cargo run --locked -- check' >> "$d/.github/workflows/checks.yml"
expect fail "'cargo run' in a job" "$d" "checks.yml:8: 'cargo run'"

d="$(gates cargo-toolchain-run)"
echo 'cargo +stable -q run -- check' >> "$d/scripts/check-all.sh"
expect fail "'cargo run' with a toolchain and a flag before the command" "$d" "check-all.sh:5: 'cargo run'"

d="$(gates target-path)"
echo 'run "nuc" target/release/nuc check' >> "$d/scripts/check-all.sh"
expect fail "a binary under target/ in check-all.sh" "$d" "a path under a target directory"

d="$(gates dot-target-path)"
echo './target/debug/nuc check' >> "$d/.githooks/pre-push"
expect fail "a binary under ./target/ in a hook" "$d" "pre-push:3: a path under a target directory"

d="$(gates nuc-command)"
echo 'nuc check || exit 1' >> "$d/.githooks/pre-push"
expect fail "'nuc' run by a hook" "$d" "pre-push:3: 'nuc' run by a gate"

d="$(gates nuc-run-key)"
echo '      - run: nuc check --format sarif' >> "$d/.github/workflows/checks.yml"
expect fail "'nuc' as the value of a run key" "$d" "'nuc' run by a gate"

d="$(gates nuc-after-and)"
# shellcheck disable=SC2016 # the variable is the gate's
echo 'cd "$ROOT" && nuc.exe check' >> "$d/.githooks/pre-push"
expect fail "'nuc' after '&&'" "$d" "'nuc' run by a gate"

d="$(gates nuc-substitution)"
# shellcheck disable=SC2016 # the substitution is the gate's
echo 'out="$(nuc check)"' >> "$d/scripts/check-all.sh"
expect fail "'nuc' in a command substitution" "$d" "'nuc' run by a gate"

d="$(gates nuc-in-prose)"
echo '      - name: Report what nuc would find' >> "$d/.github/workflows/checks.yml"
expect pass "'nuc' in the name of a step is no command" "$d"

d="$(gates script-run-by-check-all)"
echo 'run "Sub" scripts/check-sub.sh' >> "$d/scripts/check-all.sh"
printf '#!/usr/bin/env bash\ncargo run -- check\n' > "$d/scripts/check-sub.sh"
expect fail "'cargo run' in a script check-all.sh runs" "$d" "scripts/check-sub.sh:2: 'cargo run'"

d="$(gates script-in-lib)"
mkdir -p "$d/scripts/lib"
printf 'run_nuc() {\n  nuc check\n}\n' > "$d/scripts/lib/helpers.sh"
expect fail "'nuc' in a library a gate sources" "$d" "scripts/lib/helpers.sh:2: 'nuc' run by a gate"

d="$(gates cargo-alias)"
echo 'cargo r -- check' >> "$d/scripts/check-all.sh"
expect fail "the alias 'cargo r'" "$d" "'cargo run'"

d="$(gates cargo-install-path)"
# shellcheck disable=SC2016 # the variable is the gate's
echo 'cargo install --locked --path . --root "$work"' >> "$d/.githooks/pre-push"
expect fail "'cargo install --path'" "$d" "'cargo install --path'"

d="$(gates cargo-install-tool)"
echo 'cargo install --locked cargo-deny@0.20.2' >> "$d/.githooks/pre-push"
expect pass "'cargo install' of a tool from crates.io builds no working tree" "$d"

for launcher in "env -i PATH=/usr/bin" "exec" "command" "time" "xargs"; do
  d="$(gates "launcher-${launcher%% *}")"
  echo "$launcher nuc check" >> "$d/.githooks/pre-push"
  expect fail "'nuc' after '$launcher'" "$d" "'nuc' run by a gate"
done

d="$(gates other-target-dir)"
# shellcheck disable=SC2016 # the variable is the gate's
echo '"$CARGO_TARGET_DIR/debug/nuc" check' >> "$d/scripts/check-all.sh"
expect fail "a binary under another target directory" "$d" "a path under a target directory"

d="$(gates build-tests-not-read)"
printf '#!/usr/bin/env bash\ncargo test --locked\ncargo run -- check\n' > "$d/scripts/check-build.sh"
# shellcheck disable=SC2016 # the variable is the gate's
printf '#!/usr/bin/env bash\n"$ROOT/target/debug/nuc" check\n' > "$d/scripts/check-parity.sh"
expect pass "the two tests of the build run the build" "$d"

# The hooks on a released nuc (SEC-gates-run-released-nuc §2, §3).
zero="$(printf '0%.0s' {1..40})"
d="$(gates pinned-hook)"
echo 'nuc hook pre-push "$@"' >> "$d/.githooks/pre-push"
printf 'release v1.0.0\n%s pre-push\n' "$zero" > "$d/.githooks/nuc.blobs"
expect pass "a hook the blob list pins runs nuc" "$d"
d="$(gates unpinned-hook)"
echo 'nuc hook pre-push "$@"' >> "$d/.githooks/pre-push"
printf 'release v1.0.0\n%s pre-commit\n' "$zero" > "$d/.githooks/nuc.blobs"
expect fail "a hook the blob list does not pin runs nuc" "$d" "pre-push:3: 'nuc' run by a gate"

d="$(gates version-query)"
# shellcheck disable=SC2016 # the substitution is the self-test's
printf '#!/usr/bin/env bash
have="$(nuc --version 2>/dev/null)"
' > "$d/scripts/test-git-hooks.sh"
expect pass "the version query of the hooks' self-test runs nuc" "$d"
# shellcheck disable=SC2016 # the substitution is the self-test's
printf '#!/usr/bin/env bash
have="$(nuc --version 2>/dev/null)"
nuc check
' > "$d/scripts/test-git-hooks.sh"
expect fail "another line of the hooks' self-test that runs nuc" "$d" "scripts/test-git-hooks.sh:3: 'nuc' run by a gate"

d="$(gates installer)"
mkdir -p "$d/release"
# shellcheck disable=SC2016 # the parameter is the installer's
printf '#!/bin/sh\ncp target/release/nuc "$3/nuc"\n' > "$d/release/install-nuc.sh"
expect fail "an installer that copies a build" "$d" "release/install-nuc.sh:2: a path under a target directory"

# shellcheck disable=SC2016 # the variable is the workflow's
install='      - run: sh release/install-nuc.sh release/gate/SHA256SUMS release/gate/VERSION "$RUNNER_TEMP/n"'
for t in test-git-hooks test-ready-for-main test-normalize-remap; do
  d="$(gates "uninstalled-$t")"
  echo "      - run: bash scripts/$t.sh" >> "$d/.github/workflows/checks.yml"
  expect fail "a job that runs scripts/$t.sh without installing nuc" "$d" \
    "the job 'build' runs scripts/$t.sh, which runs a hook, without installing the released nuc"
done
d="$(gates installed)"
printf '%s\n      - run: bash scripts/test-git-hooks.sh\n' "$install" >> "$d/.github/workflows/checks.yml"
expect pass "a job that installs nuc before it runs a hook" "$d"
d="$(gates installed-elsewhere)"
printf '%s\n  shell:\n    steps:\n      - run: bash scripts/test-git-hooks.sh\n' "$install" >> "$d/.github/workflows/checks.yml"
expect fail "nuc installed in another job" "$d" "the job 'shell' runs scripts/test-git-hooks.sh"
d="$(gates installed-after)"
printf '      - run: bash scripts/test-git-hooks.sh\n%s\n' "$install" >> "$d/.github/workflows/checks.yml"
expect fail "nuc installed after the hook ran" "$d" "the job 'build' runs scripts/test-git-hooks.sh"

d="$(gates gate-version)"
mkdir -p "$d/release/gate"
echo 0.2.2 > "$d/release/gate/VERSION"
printf 'minimum = "0.2.2"\n' > "$d/nuc.toml"
expect pass "a release CI installs as new as the minimum" "$d"
printf 'minimum = "0.2.10"\n' > "$d/nuc.toml"
expect fail "a release CI installs older than the minimum" "$d" "0.2.2 is older than the minimum 0.2.10"
printf 'minimum = "0.1.9"\n' > "$d/nuc.toml"
expect pass "a release CI installs newer than the minimum" "$d"
rm "$d/release/gate/VERSION"
expect fail "a nuc.toml without release/gate/VERSION" "$d" "release/gate/VERSION: not found"

d="$(gates no-gates)"
rm -r "$d/scripts" "$d/.githooks" "$d/.github"
expect pass "a tree without gates has nothing to check" "$d" "0 gates"

expect fail "a root that does not exist" "$work/missing" "is not a directory"

expect_refused "a root given as a first argument" "$work"
expect_refused "an argument the check does not read" --verbose

summary "Gates check self-test"
