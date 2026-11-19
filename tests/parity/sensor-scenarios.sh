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

# tests_on_main <dir> — main with test files of several toolchains and paths git quotes.
tests_on_main() {
  change "$1" src/lib.rs lib
  change "$1" tests/a_test.rs a
  change "$1" tests/moved_test.rs moved
  change "$1" tests/kept_test.rs kept
  change "$1" spec/x.spec.js x
  change "$1" "test dir/my test.py" py
  change "$1" $'t\xc3\xabst/\xc3\xbc.rs' umlaut
  change "$1" src/Widget.java java
}

# tests_branch <dir> — a branch off main that deletes, moves, and adds markers in each toolchain,
# in files of another toolchain, in a file without an extension, in an upper-case extension, and in
# a path the sensor leaves out.
tests_branch() {
  tests_on_main "$1"
  sp_git "$1" switch -q -c feature
  sp_git "$1" rm -q tests/a_test.rs
  sp_git "$1" rm -q $'t\xc3\xabst/\xc3\xbc.rs'
  mkdir -p "$1/src/moved"
  sp_git "$1" mv tests/moved_test.rs src/moved/kept.rs
  sp_git "$1" mv tests/kept_test.rs tests/renamed_test.rs
  sp_git "$1" commit -q -m moves
  change "$1" tests/b_test.rs $'fn x() {}\n#[ignore]\nfn fit() {}'
  change "$1" web/app.test.js $'it.only("x", f)\n  describe.skip(1)\nxit(2)\na.it.only(3)\nfit(4)'
  change "$1" web/UPPER.TS 'test.only(5)'
  change "$1" "test dir/my test.py" '@pytest.mark.skip'
  change "$1" pkg/x_test.go 'func T(t *testing.T) { t.SkipNow() }'
  change "$1" src/Widget.java $'@Disabled\nenabled = false'
  change "$1" src/W.cs 'Skip = "why"'
  change "$1" Makefile '#[ignore]'
  mkdir -p "$1/scripts"
  change "$1" scripts/sensor-tests-kept.sh '#[ignore] it.only('
}

tests_kept_branch() { tests_branch "$1"; }

# The same commits, the change named by its base and head.
tests_kept_named() { tests_branch "$1"; sp_git "$1" tag base main; }

# On main, the merge base is the head.
tests_kept_same() { tests_on_main "$1"; }

# A history without a main: nothing to compare.
tests_kept_no_main() { tests_on_main "$1"; sp_git "$1" branch -q -m trunk; }

# A base that names no commit.
tests_kept_bad_ref() { tests_on_main "$1"; }

# entry <dir> <name> <text> — one commit that writes a backlog entry.
entry() { change "$1" "docs/backlog/$2" "$3"; }

backlog_no_commit() { :; }

backlog_none() { change "$1" README.md start; }

# Entries on main, with and without a heading, one that is no Markdown; a branch that adds one, a
# branch main holds, a branch cut before main deleted an entry, and one that adds an entry main
# has as well.
backlog_branches() {
  entry "$1" one.md $'Intro\n# First entry\n# Later'
  entry "$1" two.md 'no heading'
  entry "$1" notes.txt '# not an entry'
  sp_git "$1" switch -q -c feat/old
  sp_git "$1" switch -q main
  sp_git "$1" rm -q docs/backlog/two.md
  sp_git "$1" commit -q -m gone
  sp_git "$1" switch -q -c feat/new
  entry "$1" three.md $'#  Spaced title\r'
  entry "$1" one-more.md '#'
  sp_git "$1" switch -q -c feat/same main
  entry "$1" one.md 'again'
  sp_git "$1" switch -q main
  sp_git "$1" branch -q merged
}

# origin/main stands behind main: it is the one read.
backlog_origin() {
  entry "$1" one.md '# One'
  sp_git "$1" update-ref refs/remotes/origin/main HEAD
  entry "$1" two.md '# Two'
}

# ticket <dir> <path> <blocked by> <mode> — a ticket file, not committed.
ticket() {
  mkdir -p "$(dirname "$1/$2")"
  printf '# Ticket\n\n## What to build\n\n<!-- a comment\nthat spans -->\n\n## Blocked by\n\n%s\n\n## Mode\n\n%s\n' "$3" "$4" > "$1/$2"
}

plan_none() { change "$1" README.md start; }

plan_tickets() {
  change "$1" README.md start
  ticket "$1" docs/plan/s/01-first.md '- None (can start immediately)' 'unattended: the test decides'
  ticket "$1" docs/plan/s/02-second.md '- docs/plan/s/01-first.md' 'with the human'
  ticket "$1" docs/plan/s/03-third.md $'- docs/plan/s/09-gone.md\n- docs/plan/s/03-third.md' '<!-- x --> unattended'
  ticket "$1" docs/plan/s/04-claimed.md '- None' 'with the human'
  ticket "$1" docs/plan/t/01-remote.md '* docs/plan/s/02-second.md and docs/plan/s/01-first.md' '  unattended'
  ticket "$1" docs/plan/t/02-empty.md '' ''
  printf 'not a ticket\n' > "$1/docs/plan/s/README.md"
  printf 'not a ticket\n' > "$1/docs/plan/s/1-x.md"
  mkdir -p "$1/docs/plan/s/deeper"
  ticket "$1" docs/plan/s/deeper/05-nested.md '- None' 'x'
  ln -s 01-first.md "$1/docs/plan/s/06-link.md"
  sp_git "$1" branch -q ticket/s/04-claimed
  sp_git "$1" update-ref refs/remotes/origin/ticket/t/01-remote HEAD
}

# A ticket whose blocker is a link, which the sensor cannot place.
plan_odd() {
  change "$1" README.md start
  ticket "$1" docs/plan/s/01-first.md '- [second](02-second.md)' 'x'
  ticket "$1" docs/plan/s/02-second.md '- None' 'x'
}

plan_no_mode() {
  change "$1" README.md start
  printf '## Blocked by\n\nNone\n' > "$1/docs/plan/01-flat.md"
  mkdir -p "$1/docs/plan/s"
  printf '## Blocked by\n\nNone\n' > "$1/docs/plan/s/01-a.md"
}
