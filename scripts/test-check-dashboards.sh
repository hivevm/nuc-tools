#!/usr/bin/env bash
#
# Self-test of scripts/check-dashboards.sh.
#
# Each case writes a dashboard under a temporary directory — two variable queries, the session's
# filtered by project, and one panel query filtered by both on the whole name — runs the check
# against it, and asserts the exit code and, for a failing case, the message that names the
# violation. The baseline passes; every other case leaves out a filter or anchors one less.
#
# Inputs: scripts/check-dashboards.sh scripts/lib/
#
# Usage:
#     scripts/test-check-dashboards.sh
# Exit code 0 when every case passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/selftest.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/selftest.sh"
CHECK="$ROOT/scripts/check-dashboards.sh"

# shellcheck disable=SC2016  # the variables are Grafana's, not the shell's
PROJECT="match(Attributes['vcs.repository.name'], '^(\${project:regex})\$')"
# shellcheck disable=SC2016
SESSION="match(Attributes['session.id'], '^(\${session:regex})\$')"
LIST_PROJECTS="SELECT DISTINCT Attributes['vcs.repository.name'] FROM otel.otel_metrics_sum"

# dashboard <name> <session list query> <panel query> — a fixture root with one dashboard.
dashboard() {
  local dir="$work/$1"
  mkdir -p "$dir/.devcontainer/grafana/dashboards"
  cat > "$dir/.devcontainer/grafana/dashboards/overview.json" <<EOF
{
  "templating": {
    "list": [
      { "name": "project", "query": {
          "rawSql": "$LIST_PROJECTS"
      } },
      { "name": "session", "query": {
          "rawSql": "$2"
      } }
    ]
  },
  "panels": [
    { "id": 1, "targets": [ {
        "rawSql": "$3"
    } ] }
  ]
}
EOF
  echo "$dir"
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

SESSIONS="SELECT DISTINCT Attributes['session.id'] FROM otel.otel_metrics_sum WHERE $PROJECT"
PANEL="SELECT sum(Value) FROM otel.otel_metrics_sum WHERE $PROJECT AND $SESSION"

d="$(dashboard baseline "$SESSIONS" "$PANEL")"
expect pass "a panel filtered by both on the whole name, the variables by less" "$d" \
  "1 dashboards, 3 queries"

# shellcheck disable=SC2016
d="$(dashboard project-unanchored "$SESSIONS" "${PANEL/"'^(\${project:regex})\$'"/"'\${project:regex}'"}")"
expect fail "a panel's project filter without anchors" "$d" "a '\${project' that is not the whole-name match"

# shellcheck disable=SC2016
d="$(dashboard session-prefix "$SESSIONS" "${PANEL/"'^(\${session:regex})\$'"/"'^(\${session:regex})'"}")"
expect fail "a panel's session filter anchored at the start only" "$d" "a '\${session' that is not the whole-name match"

# shellcheck disable=SC2016
d="$(dashboard variable-unanchored "${SESSIONS/"'^(\${project:regex})\$'"/"'\${project:regex}'"}" "$PANEL")"
expect fail "a variable query's filter without anchors" "$d" "a '\${project' that is not the whole-name match"

d="$(dashboard panel-without-session "$SESSIONS" "${PANEL/" AND $SESSION"/}")"
expect fail "a panel query without the session filter" "$d" "a panel query without the 'session' filter"

d="$(dashboard panel-without-project "$SESSIONS" "${PANEL/"$PROJECT AND "/}")"
expect fail "a panel query without the project filter" "$d" "a panel query without the 'project' filter"

d="$(dashboard panels-first "$SESSIONS" "$PANEL")"
f="$d/.devcontainer/grafana/dashboards/overview.json"
{ echo '{'; sed -n '/^  "panels"/,/^  \]/p' "$f"; echo '  ,'; sed -n '/^  "templating"/,/^  },/p' "$f" \
  | sed '$s/,$//'; echo '}'; } > "$f.new" && mv "$f.new" "$f"
expect pass "the panels before the variables, as Grafana exports them" "$d" "1 dashboards, 3 queries"

d="$work/none"; mkdir -p "$d"
expect pass "a repository without dashboards" "$d" "0 dashboards"

# --- summary -------------------------------------------------------------------------------

summary "Dashboard self-test"
