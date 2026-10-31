#!/usr/bin/env bash
#
# Self-test of scripts/check-workflows.sh. Verifies: ADR-draft-no-stored-secrets
#
# Each case builds a fixture under a temporary directory — one workflow whose two jobs each set
# their permissions and use only secrets.GITHUB_TOKEN — runs the check against it, and asserts the
# exit code and, for a failing case, the message that names the violation. The baseline passes;
# every other case adds a stored secret, hands secrets on, or drops a job's permissions.
#
# Inputs: scripts/check-workflows.sh scripts/lib/
#
# Usage:
#     scripts/test-check-workflows.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
CHECK="$ROOT/scripts/check-workflows.sh"

# workflow <dir> [extra step lines...] — write a workflow with the jobs 'one' and 'two', each
# with its permissions; the extra lines are added as steps of 'two'.
# shellcheck disable=SC2016 # the expressions are the workflow's
workflow() {
  local dir="$1"; shift
  mkdir -p "$dir/.github/workflows"
  {
    echo 'name: Fixture'
    echo '# A comment naming secrets.SOME_TOKEN is no reference.'
    echo 'on: [push]'
    echo 'permissions: {}'
    echo 'jobs:'
    echo '  one:'
    echo '    runs-on: ubuntu-latest'
    echo '    permissions:'
    echo '      contents: read'
    echo '    steps:'
    echo '      - name: Check that no credentials leak'
    echo '        run: echo "$TOKEN"'
    echo '        env:'
    echo '          TOKEN: ${{ secrets.GITHUB_TOKEN }}'
    echo ''
    echo '  two:'
    echo '    # A comment between the job and its keys.'
    echo '    runs-on: ubuntu-latest'
    echo '    permissions:'
    echo '      contents: read'
    echo '      id-token: write'
    echo '    steps:'
    echo '      - run: echo two'
    printf '      %s\n' "$@"
  } > "$dir/.github/workflows/fixture.yml"
}

# expect <pass|fail> <name> <dir> [needle] — run the check; when a needle is given, its output
# must contain it.
expect() {
  local want="$1" name="$2" dir="$3" needle="${4:-}"
  local out rc
  out="$(bash "$CHECK" "$dir" 2>&1)"; rc=$?
  if [[ "$want" == "pass" && $rc -eq 0 && ( -z "$needle" || "$out" == *"$needle"* ) ]] \
     || [[ "$want" == "fail" && $rc -ne 0 && ( -z "$needle" || "$out" == *"$needle"* ) ]]; then
    passed=$((passed + 1))
    return
  fi
  failed=$((failed + 1))
  echo "FAIL: $name (expected $want${needle:+ with needle \"$needle\"}, exit $rc)" >&2
  while IFS= read -r l; do printf '    %s\n' "$l"; done <<< "$out" >&2
}

# --- cases ---------------------------------------------------------------------------------

d="$work/baseline"
workflow "$d"
expect pass "jobs with their permissions and only secrets.GITHUB_TOKEN pass" "$d" \
  "1 workflows, 1 secret references"

d="$work/stored-secret"
# shellcheck disable=SC2016 # the expression is the workflow's
workflow "$d" '- run: echo "$T"' '  env:' '    T: ${{ secrets.RELEASE_TOKEN }}'
expect fail "a stored secret" "$d" "'secrets.RELEASE_TOKEN' reads the secrets context"

d="$work/indexed-secret"
# shellcheck disable=SC2016 # the expression is the workflow's
workflow "$d" "- run: echo \"\${{ secrets['NPM'] }}\""
expect fail "a secret by index" "$d" "'secrets['NPM']' reads the secrets context"

d="$work/whole-context"
# shellcheck disable=SC2016 # the expression is the workflow's
workflow "$d" '- run: echo "${{ toJSON(secrets) }}"'
expect fail "the whole secrets context" "$d" "'secrets' reads the secrets context"

d="$work/token-and-secret"
# shellcheck disable=SC2016 # the expression is the workflow's
workflow "$d" '- run: echo "${{ secrets.GITHUB_TOKEN }} ${{ secrets.OTHER }}"'
expect fail "a stored secret beside GITHUB_TOKEN on one line" "$d" "'secrets.OTHER'"

d="$work/inside-a-function"
# shellcheck disable=SC2016 # the expression is the workflow's
workflow "$d" '- run: echo "$T"' '  env:' "    T: \${{ format('{0}', secrets.DEPLOY_KEY) }}"
expect fail "a secret inside an expression with braces before it" "$d" "'secrets.DEPLOY_KEY'"

d="$work/across-lines"
# shellcheck disable=SC2016 # the expression is the workflow's
workflow "$d" '- run: echo "$T"' '  env:' '    T: ${{' '      secrets.DEPLOY_KEY' '      }}'
expect fail "a secret in an expression across lines" "$d" "'secrets.DEPLOY_KEY'"

d="$work/in-a-name"
workflow "$d" '- name: Rotate the secrets'
expect fail "the word in a step name, which the check does not tell from a reference" "$d" "'secrets'"

d="$work/inherit"
workflow "$d"
{
  echo '  call:'
  echo '    permissions:'
  echo '      contents: read'
  echo '    uses: ./.github/workflows/other.yml'
  echo '    secrets: inherit'
} >> "$d/.github/workflows/fixture.yml"
expect fail "secrets handed to a reusable workflow" "$d" "a 'secrets:' key hands secrets"

d="$work/no-permissions"
workflow "$d"
{
  echo '  three:'
  echo '    runs-on: ubuntu-latest'
  echo '    steps:'
  echo '      - run: echo three'
} >> "$d/.github/workflows/fixture.yml"
expect fail "a job without permissions" "$d" "job 'three' sets no permissions of its own"

d="$work/nested-permissions"
workflow "$d"
{
  echo '  four:'
  echo '    runs-on: ubuntu-latest'
  echo '    steps:'
  echo '      - uses: ./local'
  echo '        with:'
  echo '          permissions: write'
} >> "$d/.github/workflows/fixture.yml"
expect fail "a 'permissions' key below the job's own keys is not the job's" "$d" \
  "job 'four' sets no permissions"

d="$work/after-jobs"
workflow "$d"
printf 'env:\n  permissions: none\n' >> "$d/.github/workflows/fixture.yml"
expect pass "a top-level key after the jobs ends them" "$d"

d="$work/no-workflows"
mkdir -p "$d"
expect pass "a repository without workflows has nothing to check" "$d" "0 workflows"

expect fail "a root that does not exist" "$work/missing" "is not a directory"

summary "Workflow credentials check self-test"
