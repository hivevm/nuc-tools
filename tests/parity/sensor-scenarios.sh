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

# workflow <dir> <name> <text> — a workflow file, not committed.
workflow() {
  mkdir -p "$1/.github/workflows"
  printf '%s\n' "$3" > "$1/.github/workflows/$2"
}

refs_none() { change "$1" README.md start; }

# Every form and every finding, in a .yml and a .yaml, a hidden workflow and a directory the glob
# lists, a comment, quotes, a carriage return, and a byte that is no UTF-8.
refs_mixed() {
  workflow "$1" b.yml $'jobs:\n  x:\n    steps:\n      - uses: actions/checkout@v4\n      - uses: "a/b@1.2.3"  # pinned\n      - uses: c/d@0123456789abcdef0123456789abcdef01234567\n      - uses: e/f@main\n      - uses: ./local\n      - uses: docker://alpine\n      - uses: docker://host:5000/img:1\n      - uses: g/h\n      # - uses: i/j@main\n      - uses: \'k/l@v2\'\r\n        run: |\n          uses: m/n@\xff\n'
  workflow "$1" a.yaml $'steps:\n  - uses: o/p@v1#x'
  workflow "$1" .hidden.yml $'- uses: q/r@main'
  mkdir -p "$1/.github/workflows/dir.yml"
}

# One form alone, a major tag, with a Dependabot entry for the actions.
refs_major() {
  workflow "$1" ci.yml $'- uses: a/b@v4\n- uses: c/d@3'
  printf 'version: 2\nupdates:\n  - package-ecosystem: "github-actions"\n' > "$1/.github/dependabot.yml"
}

refs_sha() { workflow "$1" ci.yml $'- uses: a/b@0123456789abcdef0123456789abcdef01234567'; }

# The identifier of the fixture's NUC- ADR, assembled so that no reader of references reads one.
PLAIN_NUC="NUC""-rule-y"

# plain_main <dir> — main with AGENTS.md and an ADR of each name a release reads, each section with
# one short paragraph, as the template's self-test of the sensor builds it; then the branch work.
plain_main() {
  mkdir -p "$1/docs/adr"
  printf '# Agent Guide\n\n## 1. Rules\n\n- The human decides.\n- Old line that says should.\n' > "$1/AGENTS.md"
  local name
  for name in FIX-rule-x "$PLAIN_NUC" 0001-old draft-new; do
    printf '# %s: X\n\n- **Status:** accepted\n\n## Context\n\nThe human decides.\n\n## Decision\n\nThe human decides.\n\n## Enforcement\n\nThe human decides.\n' \
      "$name" > "$1/docs/adr/$name.md"
  done
  sp_git "$1" add -A
  sp_git "$1" commit -q -m start
  sp_git "$1" switch -q -c work
}

# plain_after <dir> <file> <heading> <text> — the text as a paragraph right after the heading, not
# committed.
plain_after() {
  T="$4" awk -v h="$3" '{ print } $0 == h { print ""; print ENVIRON["T"] }' "$1/$2" > "$1/$2.new"
  mv "$1/$2.new" "$1/$2"
}

# plain_glossary <dir> — a glossary whose entries list words under _Avoid:_ in each form the
# readers of the releases take: a qualifier, a wrapped item, items they cannot read, and a sub-list.
# shellcheck disable=SC2016 # the backticks are Markdown, not a command substitution
plain_glossary() {
  mkdir -p "$1/docs"
  printf '%s\n' '# Glossary' '' 'One term per concept; list under _Avoid:_ the synonyms it displaces.' '' \
    '- **Git hook** — a script git runs.' '  _Avoid:_ hook script, trigger.' \
    '- **Node shape** — the form in which a node is drawn.' '  _Avoid:_ *shape* alone, figure.' \
    '- **Loop** — an edge from a node back to itself.' '  _Avoid:_ self-edge in prose, self-loop.' \
    '- **Culling** — leaving out what lies outside the viewport.' \
    '  _Avoid:_ frustum culling, outside the wording of the constraint that requires it.' \
    '- **Chapter** — a part of the book.' '  _Avoid:_ include' '  file, part,' '  ci/cd.' \
    '- **User interface** — what a human sees.' '  _Avoid:_ UI, daemon; `pin`, -flag, Größe.' \
    '- **Ticket** — one slice of work.' '  _Avoid:_ story,' '  - kept apart from boards,' '    lanes.' \
    '- **Spec** — the destination of a feature.' '  It is disposable.' '  _Avoid:_ epic, <placeholder>.' \
    > "$1/docs/GLOSSARY.md"
}

# plain_added <dir> — on the branch, rules added in every form the readers of the releases tell
# apart, committed: long sentences, soft modal verbs, and avoided words in prose, in italics, in
# code spans, wrapped, in fences, in quotes, in identifiers, and across a paragraph.
# shellcheck disable=SC2016 # the backticks and the dollar sign are Markdown and the rule's text
plain_added() {
  local long="The session that reads this rule before it starts any work must find every single word of it in the glossary and must also know who acts here."
  plain_glossary "$1"
  printf '# ADRs\n\n## Categories\n\n- **UI** — K.\n' > "$1/docs/adr/README.md"
  plain_after "$1" AGENTS.md "## 1. Rules" "$(printf -- '- The session should read the glossary.\n- **Plain.** Say *must*; _should_ leaves the reader guessing.\n- **Cache.** A session writes with *write\n  mode*, and it could flush before the *next* run.\n- **Stop.** Run `make it so it\n  should stop` first.\n- The run_should_stop flag might be set.\n> A quote that should not count.\n- Compute a * b from $_ENV, and then\n  run a hook script.\n- Each Trigger runs.')"
  plain_after "$1" docs/adr/FIX-rule-x.md "## Decision" "$(printf '%s\n%s\n\n%s\n\n%s\n\n%s\n%s\n\n%s\n\n%s\n\n%s' \
    "$long" "${long%% before it*}" "before it${long#* before it}" \
    "${long/ any work / any [\`docs/GLOSSARY.md\`](docs/GLOSSARY.md). Then it }" \
    "It runs a hook" "  script before each commit, and the shape of the node shape." \
    "Each glyph, self-edge, frustum culling, include file, story, and lanes; the UI and [UI-some-view](UI-some-view.md) and NUC-container-hides-host-daemon and xNUC-a-b-daemon run." \
    "It runs on each hook" "Script authors read it. A card, an epic, Größe, and a pin.")"
  plain_after "$1" docs/adr/FIX-rule-x.md "## Enforcement" "$(printf '````md\n```sh\nit should run\n````\n\nIt might run.\n\n```md\n## Notes\n```\n\nIt ideally runs.')"
  plain_after "$1" docs/adr/FIX-rule-x.md "## Context" "$long It should run."
  plain_after "$1" "docs/adr/$PLAIN_NUC.md" "## Decision" "Each trigger should run."
  plain_after "$1" docs/adr/0001-old.md "## Decision" "The session should read it, a hook script."
  plain_after "$1" docs/adr/draft-new.md "## Decision" "The check might run."
  printf '%s should\n' "$long" > "$1/README-more.md"
  sp_git "$1" add -A
  sp_git "$1" commit -q -m rules
}

# The rules added on a branch, read against main; the base of zeros reads main's merge base.
plain_branch() { plain_main "$1"; plain_added "$1"; }

# A renamed ADR: the lines its rename changes are read from v0.9.0, the whole file before.
plain_rename() {
  plain_main "$1"
  plain_after "$1" docs/adr/FIX-rule-x.md "## Decision" "The session should read it."
  sp_git "$1" commit -q -am decision
  sp_git "$1" branch -q -f main HEAD
  sp_git "$1" mv docs/adr/FIX-rule-x.md docs/adr/FIX-rule-z.md
  sp_git "$1" mv "docs/adr/$PLAIN_NUC.md" docs/adr/FIX-rule-w.md
  plain_after "$1" docs/adr/FIX-rule-w.md "## Enforcement" "It could run."
  sp_git "$1" add -A
  sp_git "$1" commit -q -m rename
}

# The change in the working tree: the committed rules, an uncommitted edit, an untracked ADR, and a
# glossary only the working tree has.
plain_worktree() {
  plain_main "$1"
  plain_added "$1"
  sp_git "$1" rm -q --cached docs/GLOSSARY.md
  sp_git "$1" commit -q -m "no glossary"
  plain_after "$1" AGENTS.md "## 1. Rules" "- It runs a hook script, and it should."
  printf '# FIX-rule-y: Y\n\n## Decision\n\nWe might do it, with each trigger.\n' > "$1/docs/adr/FIX-rule-y.md"
}

# With a head, the glossary is the head's, not the working tree's.
plain_head() { plain_branch "$1"; rm "$1/docs/GLOSSARY.md"; }

# This repository's record on a branch off a main that has none of it: every line is added.
plain_record() {
  change "$1" README.md start
  sp_git "$1" switch -q -c work
  mkdir -p "$1/docs/adr"
  cp "$ROOT"/docs/adr/*.md "$1/docs/adr/"
  cp "$ROOT/AGENTS.md" "$1/AGENTS.md"
  cp "$ROOT/docs/GLOSSARY.md" "$1/docs/GLOSSARY.md"
  sp_git "$1" add -A
  sp_git "$1" commit -q -m record
}

# A change outside the rules: nothing to report.
plain_outside() { plain_main "$1"; change "$1" src/x.rs should; }

# A history without a main: nothing to compare.
plain_no_main() { plain_main "$1"; sp_git "$1" branch -q -D main; }

# A filter on a file outside the rules, as Git LFS names one, the file's stat changed: the bash's
# git runs its clean command, a harmless cat here, and nuc's runs none, with the same report.
plain_filtered() {
  plain_main "$1"
  printf '*.png filter=bait\n' > "$1/.gitattributes"
  printf 'image\n' > "$1/a.png"
  sp_git "$1" add -A
  sp_git "$1" commit -q -m image
  sp_git "$1" config filter.bait.clean cat
  touch -d '+1 min' "$1/a.png"
  plain_after "$1" "docs/adr/FIX-rule-x.md" "## Decision" "It should run a hook script."
}

# A branch that adds an ADR beside an image under a filter, committed and checked out clean, as CI
# runs the gate: no path is deleted, so no rename can pair otherwise, and both report alike.
plain_filtered_added() {
  plain_main "$1"
  printf '*.png filter=bait\n' > "$1/.gitattributes"
  printf '# FIX-rule-n: N\n\n## Decision\n\nIt should run.\n' > "$1/docs/adr/FIX-rule-n.md"
  printf 'image\n' > "$1/d.png"
  sp_git "$1" add -A
  sp_git "$1" commit -q -m "adr and image"
  sp_git "$1" config filter.bait.clean cat
}
