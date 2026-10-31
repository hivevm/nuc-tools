#!/usr/bin/env bash
#
# Workflow credentials check for this repository (ADR-draft-no-stored-secrets).
#
# The workflows authenticate only with GITHUB_TOKEN and OIDC, each job with the permissions it
# names. This check reads every workflow under .github/workflows/ and fails on what would break
# that:
#
#   1. Secrets: a reference to the 'secrets' context inside a '${{ ... }}' expression other than
#      'secrets.GITHUB_TOKEN' — 'secrets.<NAME>', 'secrets[...]', or the whole context, as in
#      'toJSON(secrets)' — and a 'secrets:' key, which hands secrets to a reusable workflow.
#   2. Permissions: a job without a 'permissions' key of its own. A job inherits the workflow's
#      permissions otherwise, and what each job needs is the job's to name.
#
# The workflows are read line by line, not as YAML: full-line comments are dropped, a job is a key
# at the indentation of the first key under the top-level 'jobs:', and its 'permissions' key sits
# at the indentation of the job's first key. A flow mapping ('jobs: { ... }') is not read. That no
# secret is stored in the repository's settings is outside the tree; this check holds the decision
# where the tree can.
#
# Pure bash + coreutils/grep/awk. Usage:
#     scripts/check-workflows.sh [repository root]
# The root defaults to this repository; scripts/test-check-workflows.sh passes fixtures.
# Exit code 0 when the check passes, 1 otherwise, 2 when the root does not exist.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/check.sh"

ROOT="$(cd "${1:-$(dirname "${BASH_SOURCE[0]}")/..}" 2>/dev/null && pwd)" \
  || { echo "Workflow credentials check: '${1:-}' is not a directory." >&2; exit 2; }
WORKFLOWS=".github/workflows"

shopt -s nullglob
workflows=("$ROOT/$WORKFLOWS"/*.yml "$ROOT/$WORKFLOWS"/*.yaml)
shopt -u nullglob

# jobs_without_permissions <file> — '<line>:<job>' for every job without a 'permissions' key.
jobs_without_permissions() {
  awk '
    function indent(s) { match(s, /^ */); return RLENGTH }
    function close_job() { if (job != "" && !has) print job_line ":" job; job = ""; has = 0 }
    /^[[:space:]]*(#|$)/ { next }
    /^jobs:[[:space:]]*$/ { in_jobs = 1; job_indent = -1; next }
    in_jobs && /^[^[:space:]]/ { close_job(); in_jobs = 0 }
    !in_jobs { next }
    {
      i = indent($0)
      if (job_indent < 0) job_indent = i
      if (i == job_indent) {
        close_job()
        job = $0; sub(/^ */, "", job); sub(/:.*$/, "", job)
        job_line = NR; key_indent = -1
        next
      }
      if (job != "" && i > job_indent) {
        if (key_indent < 0) key_indent = i
        if (i == key_indent && $0 ~ /^ *permissions:/) has = 1
      }
    }
    END { close_job() }
  ' "$1"
}

secret_refs=0
for wf in "${workflows[@]}"; do
  rel="${wf#"$ROOT"/}"
  while IFS= read -r hit; do
    n="${hit%%:*}" text="${hit#*:}"
    if [[ "$text" =~ ^[[:space:]]*(-[[:space:]]+)?secrets: ]]; then
      add_error "$rel:$n: a 'secrets:' key hands secrets to a reusable workflow"
    fi
    while IFS= read -r expr; do
      [[ -n "$expr" ]] || continue
      while IFS= read -r ref; do
        [[ -n "$ref" ]] || continue
        secret_refs=$((secret_refs + 1))
        [[ "$ref" == "secrets.GITHUB_TOKEN" ]] \
          || add_error "$rel:$n: '$ref' is a stored secret; only secrets.GITHUB_TOKEN is used"
      done < <(grep -oE '(^|[^A-Za-z0-9_.-])secrets(\.[A-Za-z0-9_-]+|\[[^]]*\])?' <<< "$expr" \
        | sed -E 's/^[^s]//')
    done < <(grep -oE '\$\{\{[^}]*\}\}' <<< "$text")
  done < <(grep -nvE '^[[:space:]]*#' "$wf" | grep -E 'secrets')

  while IFS= read -r miss; do
    [[ -n "$miss" ]] && add_error "$rel:${miss%%:*}: job '${miss#*:}' sets no permissions of its own"
  done < <(jobs_without_permissions "$wf")
done

if ((${#errors[@]})); then
  echo "Workflow credentials check: ${#errors[@]} finding(s):" >&2
  printf '  - %s\n' "${errors[@]}" >&2
  exit 1
fi
echo "Workflow credentials check passed (${#workflows[@]} workflows, $secret_refs secret references, every job with its own permissions)."
