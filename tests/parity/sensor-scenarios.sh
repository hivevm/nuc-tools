# shellcheck shell=bash
# The git states the sensors run on beside the trees of the corpus
# (API-git-only-sensors-under-sense), sourced by scripts/lib/sensor-parity.sh. Each function gets
# a fixture repository on main whose README carries the release line of the template tree it runs
# for and whose nuc.toml turns on no rule, nothing committed yet; it builds the state through
# sp_git. tests/parity/sensors.tsv names the runs.

# overview <dir> <line> — docs/ARCHITECTURE.md with the line, or none for an empty one.
overview() {
  mkdir -p "$1/docs"
  printf '# Architecture\n\n%s\n\nThe rest.\n' "$2" > "$1/docs/ARCHITECTURE.md"
}

# change <dir> <path> <message> — one commit that writes the path.
change() {
  mkdir -p "$(dirname "$1/$2")"
  echo "$3" >> "$1/$2"
  sp_git "$1" add -A
  sp_git "$1" commit -q -m "$3"
}

# changes_since <dir> — after the overview's commit: changes in src/ twice, scripts/, the README, a
# directory that is gone again by the end, docs/ (not counted), and a merge (not counted).
changes_since() {
  change "$1" src/a.rs one
  change "$1" src/b.rs two
  change "$1" scripts/x.sh three
  change "$1" README.md four
  change "$1" gone/x five
  change "$1" docs/more.md six
  sp_git "$1" switch -q -c side
  change "$1" src/side.rs seven
  sp_git "$1" switch -q main
  change "$1" tools/t eight
  sp_git "$1" merge -q --no-ff -m merge side
  sp_git "$1" rm -q -r gone
  sp_git "$1" commit -q -m "nine"
}

revision_no_history() { overview "$1" '**Last design revision:** none yet, due after 1 changes.'; }

revision_not_due() {
  overview "$1" '**Last design revision:** 2026-01-01, due after 50 changes.'
  change "$1" README.md start
  changes_since "$1"
}

revision_due() {
  change "$1" src/old.rs before
  overview "$1" '**Last design revision:** 2026-01-01, due after 3 changes.'
  change "$1" README.md start
  changes_since "$1"
}

# The second revision of a day writes its ordinal, which releases before v0.6.2 cannot read.
revision_due_ordinal() {
  overview "$1" '**Last design revision:** 2026-01-01, due after 3 changes.'
  change "$1" README.md start
  change "$1" src/a.rs between
  overview "$1" '**Last design revision:** 2026-01-01 (2), due after 2 changes.'
  change "$1" README.md again
  changes_since "$1"
}

revision_none_yet() {
  overview "$1" '**Last design revision:** none yet, due after 4 changes.'
  change "$1" README.md start
  changes_since "$1"
}

# A date not committed yet counts nothing.
revision_uncommitted() {
  overview "$1" '**Last design revision:** none yet, due after 4 changes.'
  change "$1" README.md start
  changes_since "$1"
  overview "$1" '**Last design revision:** 2026-02-02, due after 4 changes.'
}

revision_no_line() {
  overview "$1" 'No line.'
  change "$1" README.md start
}

revision_no_overview() { change "$1" README.md start; }

revision_unreadable() {
  overview "$1" '**Last design revision:** soon, due after some changes.'
  change "$1" README.md start
}
