#!/usr/bin/env bash
#
# macOS floor check for the workflows of this repository.
#
# A GitHub-hosted macOS runner ships bash 3.2, and a script that sources lib/bash.sh stops below
# bash 4.4 (NUC-checks-are-bash §1), so a step of a macOS job that runs one fails. This check reads
# every workflow under .github/workflows/ and fails on a step of a job that names a macOS runner,
# in 'runs-on' or in its matrix, that names a script scripts/<path>.sh, run through bash or by its
# own shebang, by a relative path or in quotes, where that script sources lib/bash.sh or
# lib/selftest.sh, itself or through a library under scripts/lib/ it sources, unless the step's
# 'if:' skips macOS: 'runner.os != 'macOS'', or 'runner.os == 'Linux'' or '== 'Windows''. A
# finding names the workflow, the job, and the script.
#
# The workflows are read line by line, not as YAML, as scripts/check-workflows.sh reads them:
# full-line comments are dropped, a job is a key at the indentation of the first key under the
# top-level 'jobs:', and a step runs from its '- ' to the next at its indentation, its 'run:' block
# included. The check errs towards failing where it cannot tell:
#   - a job names a macOS runner where any line of it outside its steps holds 'macos', a name or
#     a variable included;
#   - an 'if:' with '||' skips nothing, since its other side may run the step on macOS; one with
#     '&&' skips macOS where a side does; another form of the condition, a matrix value, a negation,
#     or an 'if:' over several lines, skips nothing;
#   - any line of a step that names a script counts, its name included.
# A flow mapping, a job that calls a reusable workflow, a job-level 'if:', and a script that runs
# another through bash, not by sourcing it, are not read. A script is known to source the floor by
# a line that is no comment and starts with 'source' or '.', as the rule shell/bash-floor of
# scripts/check-shell.sh holds it by running the script.
#
# Pure bash + coreutils/grep/awk. Usage:
#     scripts/check-macos-floor.sh [--root <dir>]
# The root defaults to this repository; scripts/test-check-macos-floor.sh passes fixtures.
# Exit code 0 when the check passes, 1 otherwise, and 2 on an argument it does not read or a root
# that is no directory.

set -uo pipefail

# shellcheck source-path=SCRIPTDIR source=lib/bash.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/bash.sh"

# shellcheck source-path=SCRIPTDIR source=lib/check.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/check.sh"

# shellcheck source-path=SCRIPTDIR source=lib/root.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/root.sh"
usage="scripts/check-macos-floor.sh [--root <dir>]"
enter_root "${BASH_SOURCE[0]}" "$usage" "$@"
rest_args "${BASH_SOURCE[0]}" "$usage" 0 "${ARGS[@]}"
WORKFLOWS=".github/workflows"

shopt -s nullglob
workflows=("$ROOT/$WORKFLOWS"/*.yml "$ROOT/$WORKFLOWS"/*.yaml)
shopt -u nullglob

# macos_steps <file> — one line per step of a macOS job: its line, its job, its if, and the
# scripts it runs, separated by spaces, the four fields by the unit separator, since a tab is
# whitespace to read and two of them around an empty if would fold into one.
macos_steps() {
  awk '
    function indent(s) { match(s, /^ */); return RLENGTH }
    function close_step() {
      if (step_line) { n++; line_of[n] = step_line; if_of[n] = step_if; run_of[n] = step_run }
      step_line = 0; step_if = ""; step_run = ""
    }
    function close_job(   i) {
      close_step()
      if (job != "" && mac)
        for (i = 1; i <= n; i++) printf "%s\037%s\037%s\037%s\n", line_of[i], job, if_of[i], run_of[i]
      job = ""; mac = 0; n = 0; in_steps = 0
    }
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
        next
      }
      if (in_steps && i <= steps_indent && !(i == steps_indent && $0 ~ /^ *- /)) {
        close_step(); in_steps = 0
      }
      if (!in_steps) {
        if (tolower($0) ~ /macos/) mac = 1
        if ($0 ~ /^ *steps:[[:space:]]*$/) { in_steps = 1; steps_indent = i; step_indent = -1 }
        next
      }
      if ($0 ~ /^ *- / && (step_indent < 0 || i == step_indent)) {
        close_step(); step_indent = i; step_line = NR
      }
      text = $0; sub(/^ *(- )?/, "", text)
      if (text ~ /^if:/) { sub(/^if:[[:space:]]*/, "", text); step_if = text }
      rest = $0
      while (match(rest, /(\.\/)?scripts\/[A-Za-z0-9_.\/-]+\.sh/)) {
        script = substr(rest, RSTART, RLENGTH); sub(/^\.\//, "", script)
        step_run = step_run (step_run == "" ? "" : " ") script
        rest = substr(rest, RSTART + RLENGTH)
      }
    }
    END { close_job() }
  ' "$1"
}

# skips_macos <if> — whether a step's condition skips macOS.
skips_macos() {
  local cond="${1,,}" q="[\"']"
  # One side of an '||' may run the step on macOS whatever the other says.
  [[ "$cond" == *'||'* ]] && return 1
  [[ "$cond" =~ runner\.os[[:space:]]*!=[[:space:]]*${q}macos${q} ]] && return 0
  [[ "$cond" =~ runner\.os[[:space:]]*==[[:space:]]*${q}(linux|windows)${q} ]] && return 0
  return 1
}

# floored <path> [depth] — whether the script sources the floor, itself or through a library
# under scripts/lib/ it sources.
floored() {
  local path="$1" depth="${2:-0}" line lib
  [[ -f "$ROOT/$path" ]] && ((depth < 6)) || return 1
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*(source|\.)[[:space:]] ]] || continue
    [[ "$line" == *lib/bash.sh* || "$line" == *lib/selftest.sh* ]] && return 0
    [[ "$path" == scripts/lib/* && "$line" == */bash.sh* ]] && return 0
    if [[ "$line" =~ lib/([A-Za-z0-9_-]+)\.sh ]]; then
      lib="scripts/lib/${BASH_REMATCH[1]}.sh"
      floored "$lib" $((depth + 1)) && return 0
    elif [[ "$path" == scripts/lib/* && "$line" =~ /([A-Za-z0-9_-]+)\.sh ]]; then
      floored "scripts/lib/${BASH_REMATCH[1]}.sh" $((depth + 1)) && return 0
    fi
  done < "$ROOT/$path"
  return 1
}

steps=0
for file in "${workflows[@]}"; do
  rel="${file#"$ROOT"/}"
  while IFS=$'\x1f' read -r line job cond scripts; do
    steps=$((steps + 1))
    [[ -n "$scripts" ]] || continue
    skips_macos "$cond" && continue
    for script in $scripts; do
      floored "$script" \
        && add_error "$rel:$line: the job '$job' runs on macOS, and its step runs $script, which stops below bash 4.4 there — skip the step with if: runner.os != 'macOS' [macos/step:floor]"
    done
  done < <(macos_steps "$file")
done

if ((${#errors[@]})); then
  echo "macOS floor check FAILED:" >&2
  printf '  - %s\n' "${errors[@]}" >&2
  exit 1
fi
echo "macOS floor check passed (${#workflows[@]} workflows, $steps steps of macOS jobs, none behind the bash floor) [macos/check:passed]"
exit 0
