# shellcheck shell=bash
# The comparison of the sensors (API-git-only-sensors-under-sense), sourced by
# scripts/check-parity.sh.
#
# Each row of tests/parity/sensors.tsv names a sensor nuc ports, the git state it runs on, and its
# arguments. The state 'tree' is the tree of the corpus, or the variant, as the comparison has it:
# the sensor runs there in every comparison, on each tree whose scripts/ holds the sensor, the
# release that has it. Any other state names a function of tests/parity/sensor-scenarios.sh, which
# builds a fixture repository through git; it runs on each template tree of the corpus whose
# scripts/ holds the sensor, the fixture's README carrying the tree's release line, which the
# bash ignores.
#
# Both sides run with the environment cleared to PATH, HOME, and LC_ALL=C: the tree's bash sensor,
# given the fixture as its root as the release reads it, '--root <dir>' before the arguments where
# the sensor calls enter_root, and the last argument after them before; and
# 'nuc sense <name> --repo <dir>' with the same arguments. Compared: the standard
# output, byte for byte, and the exit status. An exit that differs is a difference unless a row of
# tests/parity/sensor-exits.tsv lists it, by the sensor, the releases it holds on, the two exits,
# and the cause. The standard error is not compared: its wording is each side's own.
#
# PARITY_SENSOR_TABLE, PARITY_SENSOR_EXITS, and PARITY_SENSOR_SCENARIOS name other files, for
# scripts/test-check-parity.sh.
#
# Usage: sensor_tree <nuc> <label> <dir> <out prefix>, sensor_scenarios <nuc> <work dir> <tree>...,
# and sensor_compare <label> <sensor> <release> <bash out> <bash exit> <nuc out> <nuc exit>; each
# returns 1 on any difference. load_sensor_tables returns 2 on a malformed table.

SENSOR_TABLE="${PARITY_SENSOR_TABLE:-$ROOT/tests/parity/sensors.tsv}"
SENSOR_EXITS="${PARITY_SENSOR_EXITS:-$ROOT/tests/parity/sensor-exits.tsv}"
SENSOR_SCENARIOS="${PARITY_SENSOR_SCENARIOS:-$ROOT/tests/parity/sensor-scenarios.sh}"

# The rows of the two tables.
st_sensors=() st_states=() st_args=()
se_sensors=() se_releases=() se_bash=() se_nuc=()

# load_sensor_tables — read the sensors and the exits that differ, and source the scenarios.
load_sensor_tables() {
  local row sensor state args extra releases bash_rc nuc_rc cause
  st_sensors=() st_states=() st_args=() se_sensors=() se_releases=() se_bash=() se_nuc=()
  [[ -r "$SENSOR_TABLE" && -r "$SENSOR_EXITS" && -r "$SENSOR_SCENARIOS" ]] \
    || { echo "Parity check: the tables of the sensors are unreadable" >&2; return 2; }
  while IFS= read -r row || [[ -n "$row" ]]; do
    [[ "$row" =~ ^[[:space:]]*(#|$) ]] && continue
    IFS=$'\t' read -r sensor state args extra <<< "$row"
    if ! [[ "$sensor" =~ ^[a-z][a-z0-9-]*$ && "$state" =~ ^[a-z_]+$ && -n "$args" && -z "$extra" ]]; then
      echo "Parity check: $SENSOR_TABLE: a row needs a sensor, a state, and arguments or '-': '$row'" >&2
      return 2
    fi
    st_sensors+=("$sensor") st_states+=("$state") st_args+=("$args")
  done < "$SENSOR_TABLE"
  while IFS= read -r row || [[ -n "$row" ]]; do
    [[ "$row" =~ ^[[:space:]]*(#|$) ]] && continue
    IFS=$'\t' read -r sensor releases bash_rc nuc_rc cause <<< "$row"
    if ! [[ "$sensor" =~ ^[a-z][a-z0-9-]*$ && "$releases" =~ ^(-|v[0-9.]+(,v[0-9.]+)*)$ \
      && "$bash_rc" =~ ^[0-9]+$ && "$nuc_rc" =~ ^[0-9]+$ && -n "$cause" ]]; then
      echo "Parity check: $SENSOR_EXITS: a row needs a sensor, releases or '-', the bash's exit, nuc's, and the cause: '$row'" >&2
      return 2
    fi
    se_sensors+=("$sensor") se_releases+=("$releases") se_bash+=("$bash_rc") se_nuc+=("$nuc_rc")
  done < "$SENSOR_EXITS"
  # shellcheck source=/dev/null # the scenarios are the table's, named by a variable
  source "$SENSOR_SCENARIOS" || return 2
  local i
  for i in "${!st_states[@]}"; do
    [[ "${st_states[i]}" == tree ]] && continue
    declare -F "${st_states[i]}" > /dev/null || {
      echo "Parity check: $SENSOR_TABLE: the state '${st_states[i]}' is no function of $SENSOR_SCENARIOS" >&2
      return 2
    }
  done
}

# exit_listed <sensor> <release> <bash exit> <nuc exit> — whether a row lists the pair of exits.
exit_listed() {
  local i
  for i in "${!se_sensors[@]}"; do
    [[ "${se_sensors[i]}" == "$1" && "${se_bash[i]}" == "$3" && "${se_nuc[i]}" == "$4" ]] || continue
    [[ "${se_releases[i]}" == - || ",${se_releases[i]}," == *",$2,"* ]] && return 0
  done
  return 1
}

# sensor_compare <label> <sensor> <release> <bash out> <bash exit> <nuc out> <nuc exit>
sensor_compare() {
  local label="$1" sensor="$2" release="$3" diffs=()
  if ! cmp -s "$4" "$6"; then
    diffs+=("the standard output differs:")
    while IFS= read -r line; do diffs+=("  $line"); done < <(diff "$4" "$6" | head -n 12)
  fi
  if [[ "$5" != "$7" ]] && ! exit_listed "$sensor" "$release" "$5" "$7"; then
    diffs+=("bash exits $5 and nuc $7, which no row of ${SENSOR_EXITS#"$ROOT/"} lists for $release")
  fi
  if ((${#diffs[@]})); then
    echo "Parity check: $label, sensor $sensor — ${#diffs[@]} difference(s):" >&2
    printf '  - %s\n' "${diffs[@]}" >&2
    return 1
  fi
}

# sensor_takes_root <script> — whether the bash sensor reads its root from '--root <dir>' before
# its arguments, as enter_root does; else it reads it from its last argument.
sensor_takes_root() { grep -q 'enter_root' "$1"; }

# sensor_pair <nuc> <label> <script> <sensor> <args> <release> <dir> <out prefix> — run both sides
# on the dir and compare them.
sensor_pair() {
  local nuc="$1" label="$2" script="$3" sensor="$4" args=() root_args=() brc nrc
  [[ "$5" == - ]] || read -r -a args <<< "$5"
  if sensor_takes_root "$script"; then
    root_args=(--root "$7" "${args[@]}")
  else
    root_args=("${args[@]}" "$7")
  fi
  env -i PATH="$PATH" HOME="$HOME" LC_ALL=C bash "$script" "${root_args[@]}" \
    > "$8.sb" 2> /dev/null < /dev/null
  brc=$?
  env -i PATH="$PATH" HOME="$HOME" LC_ALL=C "$nuc" sense "$sensor" --repo "$7" "${args[@]}" \
    > "$8.sn" 2> "$8.sn.err" < /dev/null
  nrc=$?
  sensor_compare "$label" "$sensor" "${6:--}" "$8.sb" "$brc" "$8.sn" "$nrc"
}

# sensor_tree <nuc> <label> <dir> <out prefix> — the rows of state 'tree' on the tree or variant.
sensor_tree() {
  local nuc="$1" label="$2" dir="$3" out="$4" i script release rc=0
  release="$(release_of "$dir/README.md")"
  for i in "${!st_sensors[@]}"; do
    [[ "${st_states[i]}" == tree ]] || continue
    script="$dir/scripts/sensor-${st_sensors[i]}.sh"
    [[ -f "$script" ]] || continue
    sensor_pair "$nuc" "$label" "$script" "${st_sensors[i]}" "${st_args[i]}" "$release" "$dir" \
      "$out.sensor-$i" || rc=1
  done
  return "$rc"
}

# sp_git <dir> <args...> — git in a fixture, with an identity and without hooks.
sp_git() {
  local dir="$1"
  shift
  git -C "${dir:?}" -c user.name=t -c user.email=t@example.invalid -c core.hooksPath=/dev/null "$@"
}

# sp_repo <dir> <release> — a fixture repository on main whose README carries the release line and
# whose nuc.toml turns on no rule, nothing committed yet.
sp_repo() {
  mkdir -p "$1"
  git -C "$1" init -q -b main
  printf '# Fixture\n\n- **Template release:** %s\n' "${2:-unreleased}" > "$1/README.md"
  printf 'minimum = "0.0.0"\nrules = []\n' > "$1/nuc.toml"
  printf 'nuc.toml\n' >> "$1/.git/info/exclude"
}

# sensor_scenarios <nuc> <work dir> <tree>... — the rows of the other states, on each template
# tree that has the sensor.
sensor_scenarios() {
  local nuc="$1" work="$2" tree i script release fixture rc=0 n=0
  shift 2
  for tree in "$@"; do
    release="$(release_of "$tree/README.md")"
    for i in "${!st_sensors[@]}"; do
      [[ "${st_states[i]}" == tree ]] && continue
      script="$tree/scripts/sensor-${st_sensors[i]}.sh"
      [[ -f "$script" ]] || continue
      fixture="$work/sensor-scenarios/${release:-unreleased}-$i"
      rm -rf "$fixture"
      if ! { sp_repo "$fixture" "$release" && "${st_states[i]}" "$fixture"; } > /dev/null 2>&1; then
        echo "Parity check: ${release:-unreleased}, sensor ${st_sensors[i]} — the state ${st_states[i]} could not be built" >&2
        rc=1
        continue
      fi
      sensor_pair "$nuc" "${release:-unreleased}, state ${st_states[i]}" "$script" "${st_sensors[i]}" \
        "${st_args[i]}" "$release" "$fixture" "$fixture" || rc=1
      n=$((n + 1))
      rm -rf "$fixture" "$fixture".sb "$fixture".sn "$fixture".sn.err
    done
  done
  ((rc)) || echo "Parity check: the sensor scenarios — $n run(s), the same from bash and nuc."
  return "$rc"
}
