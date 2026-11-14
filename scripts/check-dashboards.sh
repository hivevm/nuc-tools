#!/usr/bin/env bash
#
# Dashboard check for the development observability stack (.devcontainer/README.md).
#
# The overview dashboard filters every panel by the 'project' and 'session' variables. A filter
# left out shows a panel unfiltered beside filtered ones, and an unanchored match on a name also
# matches every name that contains it, so choosing 'nuc' would show 'nuc-tools' too. This check
# reads every dashboard under .devcontainer/grafana/dashboards/ and fails on:
#
#   1. Anchoring: a reference to '${project' or '${session' in a query that is not the whole-name
#      match "'^(${project:regex})$'" or "'^(${session:regex})$'".
#   2. Coverage: a panel query, a 'rawSql' after the dashboard's '"panels":' key, that does not
#      carry both filters. The variables' own queries, before that key, are exempt: the project
#      list is unfiltered, and the session list is filtered by project alone.
#
# A query is the value of one '"rawSql":' line, as Grafana writes a dashboard; the files are
# generated and no JSON parser is guaranteed on PATH.
#
# Pure bash + coreutils/grep. Usage:
#     scripts/check-dashboards.sh [repository root]
# The root defaults to this repository; scripts/test-check-dashboards.sh passes fixtures.
# Exit code 0 when the check passes, 1 otherwise.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/check.sh"

ROOT="$(cd "${1:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd)"
DIR=".devcontainer/grafana/dashboards"

shopt -s nullglob
dashboards=("$ROOT/$DIR"/*.json)
queries=0
for file in "${dashboards[@]}"; do
  name="$DIR/${file##*/}"
  in_panels=0
  line_no=0
  while IFS= read -r line; do
    line_no=$((line_no + 1))
    [[ "$line" =~ ^[[:space:]]*\"panels\":[[:space:]]*\[ ]] && in_panels=1
    [[ "$line" == *'"rawSql":'* ]] || continue
    queries=$((queries + 1))
    for var in project session; do
      anchored="'^(\${$var:regex})\$'"
      all="$(grep -o "\${$var" <<< "$line" | wc -l)"
      whole="$(grep -oF "$anchored" <<< "$line" | wc -l)"
      ((all == whole)) \
        || add_error "$name:$line_no: a '\${$var' that is not the whole-name match $anchored — an unanchored match also selects every name that contains the chosen one"
      ((in_panels && whole == 0)) \
        && add_error "$name:$line_no: a panel query without the '$var' filter — the panel ignores the choice the others follow"
    done
  done < "$file"
done

if ((${#errors[@]})); then
  echo "Dashboard check FAILED:" >&2
  printf '  - %s\n' "${errors[@]}" >&2
  exit 1
fi
echo "Dashboard check passed (${#dashboards[@]} dashboards, $queries queries, every panel filtered by project and session on the whole name)."
exit 0
