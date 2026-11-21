# shellcheck shell=bash
# The git states the sensors run on beside the trees of the corpus
# (API-sensor-cores-ask-the-shell), sourced by scripts/lib/sensor-parity.sh. Each function gets
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

# The identifier of the fixture's base ADR, assembled so that no reader of references reads one.
BIND_NUC="NUC""-base-rule"

# bind_adr <dir> <name> <status> <applies to> [<body>] — an ADR with a header, and the body after
# it, sections of one paragraph each by default.
bind_adr() {
  mkdir -p "$1/docs/adr"
  printf '# %s: Fixture\n\n- **Status:** %s\n- **Date:** 2026-01-01\n- **Applies to:** %s\n\n%s\n' \
    "${2%.md}" "$3" "$4" \
    "${5:-$'## Context\n\nWhy it came up.\n\n## Decision\n\nWe will do it.\n\n## Alternatives considered\n\n- None.\n\n## Enforcement\n\nA sensor lists it.'}" \
    > "$1/docs/adr/$2"
}

# bind_checks <dir> <a> <b> <one> <two> — a script with checks named by rule key and by number,
# their bodies as given, and an item of the header that carries old keys.
bind_checks() {
  mkdir -p "$1/scripts"
  printf '#!/usr/bin/env bash\n#\n#   t/a\n#   t/b (was: t/old, t/older)\nset -u\n\ncheck_a() {  # t/a\n  %s\n}\n\nhelper() {\n  :\n}\n\ncheck_b() {  # t/b\n  %s\n}\n\ncheck_one() {  # check 1\n  %s\n}\n\ncheck_two() {  # check 2\n  %s\n}\n' \
    "$2" "$3" "$4" "$5" > "$1/scripts/c.sh"
}

# bind_main <dir> — main with the ADRs of both records, each kind a release reads: the numbered
# ones and drafts, and those named by category; an index with a Binding table; and the files they
# name. Then the branch work.
# shellcheck disable=SC2016 # the backticks are Markdown, not a command substitution
bind_main() {
  local d="$1" kind
  for kind in 0 FIX; do
    n() { [[ "$kind" == 0 ]] && printf '%04d-%s.md' "$1" "$2" || printf 'FIX-%s-rule.md' "$2"; }
    bind_adr "$d" "$(n 1 plan)" "🟢 accepted" 'the plan under `docs/plan/` and `README.md`'
    bind_adr "$d" "$(n 2 gone)" "⚪ superseded" 'everything under `src/`'
    bind_adr "$d" "$(n 3 code)" "🟡 proposed" 'the code under `src/`, and the `pull request` template'
    bind_adr "$d" "$(n 4 graph)" "🟢 accepted" 'the graph of `crates/engine/src/graph.rs`, the root `crates/engine/src/lib.rs`, and `build.rs`'
    bind_adr "$d" "$(n 5 numbers)" "🟢 accepted" 'checks 2 of `scripts/c.sh`'
    bind_adr "$d" "$(n 6 keys)" "🟢 accepted" 'the rules `t/c`, `t/older`, and `t/x` of `scripts/c.sh`'
    bind_adr "$d" "$(n 7 names)" "🟢 accepted" 'the `Makefile`, the hooks under `.githooks`, `docs/über.md`, `odd"name.txt`, and the `plan-feature` skill'
    bind_adr "$d" "$(n 8 fenced)" "🟢 accepted" '`tools/`, `lib/x.sh`, and `Makefile`' \
      $'## Decision\n\nWe will fence.\n\n```md\n## Notes\n\nInside the fence.\n```\n\nAfter the fence.\n\n## Enforcement\n\nIt holds.\n\n## Decision\n\nA second one.'
    bind_adr "$d" "$(n 9 rejected)" "🔴 rejected" '`Makefile`'
    bind_adr "$d" "$(n 10 first)" "🟢 accepted" 'checks 1 and 3 of `scripts/c.sh`'
    bind_adr "$d" "$(n 11 akey)" "🟢 accepted" 'the rule `t/a` of `scripts/c.sh`'
  done
  bind_adr "$d" draft-new.md "🟡 proposed" 'the tools under `tools/` and the rule `t/b` of `scripts/c.sh`'
  bind_adr "$d" "$BIND_NUC.md" "🟡 proposed" 'every ADR, `scripts/base.sh`'
  printf '# ADRs\n\n## Mechanics\n\nHow the record is kept.\n\n### Binding\n\n| ADR | Title | Applies to |\n|---|---|---|\n| [0001](0001-plan.md) | Plan | the plan under `docs/plan/` and `README.md` |\n| [0003](0003-code.md) | Code |  the code under `src/`  |\n| [0008](0008-fenced.md) | Fenced | `tools/`, `lib/x.sh`, and `Makefile` |\n| [0099](0099-gone.md) | Gone | a row only |\n\n## Later\n\n| [0098](x.md) | After | after the table |\n' \
    > "$d/docs/adr/README.md"
  mkdir -p "$d/docs/plan" "$d/src" "$d/crates/engine/src/lib" "$d/build" "$d/.githooks" "$d/tools"
  printf 'plan\n' > "$d/docs/plan/README.md"
  printf 'code\n' > "$d/src/app.sh"
  printf 'old\n' > "$d/src/old.sh"
  printf 'x\n' > "$d/crates/engine/src/lib/part.rs"
  printf 'x\n' > "$d/build/asset.txt"
  printf 'all:\n' > "$d/Makefile"
  printf 'x\n' > "$d/.githooks/pre-push"
  printf 'ü\n' > "$d/docs/über.md"
  bind_checks "$d" : : : :
  sp_git "$d" add -A
  sp_git "$d" commit -q -m base
  sp_git "$d" switch -q -c work
}

# bind_changes <dir> — on the branch, committed: a path under a directory, a module of a Rust file,
# files under the directories of a library root and a build script, a file that only begins like
# one named, five files under one directory, paths git quotes, a deletion, and edits inside a check
# of each kind.
bind_changes() {
  local d="$1" i
  printf 'more\n' >> "$d/docs/plan/README.md"
  mkdir -p "$d/crates/engine/src/graph"
  printf 'mod\n' > "$d/crates/engine/src/graph/hit.rs"
  printf 'more\n' >> "$d/crates/engine/src/lib/part.rs"
  printf 'more\n' >> "$d/build/asset.txt"
  printf 'x\n' > "$d/README.md.bak"
  for i in 1 2 3 4 5; do printf '%s\n' "$i" > "$d/src/f$i.sh"; done
  printf 'more\n' >> "$d/docs/über.md"
  printf 'x\n' > "$d/odd\"name.txt"
  printf 'x\n' > "$d/tab	name.txt"
  sp_git "$d" rm -q src/old.sh
  bind_checks "$d" : true : true
  sp_git "$d" add -A
  sp_git "$d" commit -q -m changes
}

# The committed change, and in the working tree an edit between two checks, an ADR flipped, and an
# untracked file.
# shellcheck disable=SC2016 # the backticks are Markdown, not a command substitution
bind_branch() {
  bind_main "$1"
  bind_changes "$1"
  printf 'more\n' >> "$1/Makefile"
  printf 'new\n' > "$1/tools/new.sh"
  bind_adr "$1" FIX-plan-rule.md "🔴 rejected" 'the plan under `docs/plan/` and `README.md`'
  bind_adr "$1" 0001-plan.md "🔴 rejected" 'the plan under `docs/plan/` and `README.md`'
}

# An edit inside the check of the rule t/b alone, in the working tree: it binds the ADR that names
# its old key, and from v0.10.0 no ADR that names only another rule; v0.9.x reads it outside every
# numbered check.
bind_inside_key() {
  bind_main "$1"
  bind_checks "$1" : true : :
  printf 'more\n' >> "$1/.githooks/pre-push"
}

# An edit inside check 1 alone: in v0.9.x it binds the ADR that names checks 1 and 3, and not the
# one that names check 2; from v0.10.0 it lies outside every check of a rule key.
bind_inside_number() {
  bind_main "$1"
  bind_checks "$1" : : true :
}

# The last line deleted, its next line past the end of the file: every ADR that names it binds.
bind_tail() {
  bind_main "$1"
  sed '$d' "$1/scripts/c.sh" > "$1/c.sh.tmp" && cat "$1/c.sh.tmp" > "$1/scripts/c.sh" && rm "$1/c.sh.tmp"
}

# On main, nothing changed.
bind_same() { bind_main "$1"; sp_git "$1" switch -q main; }

# A history without a main: nothing to compare.
bind_no_main() { bind_main "$1"; sp_git "$1" branch -q -D main; }

# This repository's record and checks on a branch off a main that has them, the branch editing
# inside a rule's check, a source file, and the parity harness.
bind_record() {
  mkdir -p "$1/docs/adr" "$1/scripts" "$1/src/shell" "$1/tests/parity"
  cp "$ROOT"/docs/adr/*.md "$1/docs/adr/"
  cp "$ROOT/scripts/check-docs.sh" "$1/scripts/check-docs.sh"
  printf 'x\n' > "$1/src/shell/git.rs"
  printf 'x\n' > "$1/tests/parity/corpus.toml"
  sp_git "$1" add -A
  sp_git "$1" commit -q -m record
  sp_git "$1" switch -q -c work
  awk '/^check_relative_links\(\)/ { c = 1 } c && /^}/ { print "  : # edited"; c = 0 } { print }' \
    "$1/scripts/check-docs.sh" > "$1/check-docs.new"
  mv "$1/check-docs.new" "$1/scripts/check-docs.sh"
  printf 'y\n' >> "$1/src/shell/git.rs"
  printf 'y\n' >> "$1/tests/parity/corpus.toml"
  sp_git "$1" commit -q -am edits
}

# A filter on a file outside the record, as Git LFS names one, the file older than the index by
# whole seconds and its stat unchanged: neither stat-dirty nor racily clean, so the bash's git runs
# no clean command for it, and nuc refuses nothing, with the same report. nuc refuses a stat-dirty
# or racily clean one, which tests/contract.rs holds.
bind_filtered() {
  bind_main "$1"
  printf '*.png filter=bait\n' > "$1/.gitattributes"
  printf 'image\n' > "$1/a.png"
  touch -d '-1 min' "$1/a.png"
  sp_git "$1" add -A
  sp_git "$1" commit -q -m image
  sp_git "$1" config filter.bait.clean cat
  printf 'more\n' >> "$1/src/app.sh"
}

# bind_gitlink_staged <dir> [<ignore>] — on main, a submodule that is not checked out, an empty
# directory as git leaves it, its .gitmodules entry ignoring as given; then another commit of it
# staged, not committed.
bind_gitlink_staged() {
  local first
  bind_main "$1"
  sp_git "$1" switch -q main
  first="$(sp_git "$1" rev-parse HEAD)"
  mkdir -p "$1/tools/sub"
  printf '[submodule "tools/sub"]\n\tpath = tools/sub\n\turl = ./sub.git\n' > "$1/.gitmodules"
  [[ -z "${2:-}" ]] || printf '\tignore = %s\n' "$2" >> "$1/.gitmodules"
  sp_git "$1" add .gitmodules
  sp_git "$1" update-index --add --cacheinfo "160000,$first,tools/sub"
  sp_git "$1" commit -q -m submodule
  sp_git "$1" update-index --cacheinfo "160000,$(sp_git "$1" rev-parse HEAD),tools/sub"
}

# The staged commit of a submodule not checked out is a change; under ignore = all it is none, as
# the repository's setting has git read it.
bind_gitlink() { bind_gitlink_staged "$1"; }
bind_gitlink_ignored() { bind_gitlink_staged "$1" all; }

# rb_note <dir> <name> <text> — a hand-off under notes/, which no commit carries, its text as
# printf's %b reads it.
rb_note() {
  mkdir -p "$1/notes"
  printf '%b\n' "$3" > "$1/notes/$2"
}

# rb_adr <dir> <name> <status> <date> <body> — an ADR with a header and a body of one paragraph.
rb_adr() {
  mkdir -p "$1/docs/adr"
  printf '# %s: Fixture\n\n- **Status:** %s\n- **Date:** %s\n- **Deciders:** t\n\n%s\n' \
    "${2%.md}" "$3" "$4" "$5" > "$1/docs/adr/$2"
}

# rb_commit <dir> <message> — commit everything the working tree changed.
rb_commit() {
  sp_git "$1" add -A
  sp_git "$1" commit -q -m "$2"
}

# rb_main <dir> — main with an ADR of each record, numbered and named by category, and the index,
# tagged base; then the branch work, notes/ left untracked.
rb_main() {
  printf 'notes/\n' >> "$1/.git/info/exclude"
  rb_adr "$1" 0001-rule.md "🟡 proposed" 2026-01-01 "We will do it."
  rb_adr "$1" FIX-rule.md "🟡 proposed" 2026-01-01 "We will do it."
  printf '# ADRs\n\n| ADR | Title |\n|---|---|\n' > "$1/docs/adr/README.md"
  rb_commit "$1" base
  sp_git "$1" tag base
  sp_git "$1" switch -q -c work
}

# rb_branch <dir> — on the branch, linearly: the text of both ADRs with five other paths; their
# Status and Date lines with code; the index with code; ADRs whose names git quotes or awk splits,
# with paths git quotes; an ADR's text alone; and a fix. The hand-offs name the head in the ways a
# line may, an earlier commit with a comma after it, and what binds nothing, a tag at the head named
# like a hash, which git prefers to the object, among it.
rb_branch() {
  local d="$1" i head
  rb_main "$d"
  rb_adr "$d" 0001-rule.md "🟡 proposed" 2026-01-01 "We will do it, narrowed."
  rb_adr "$d" FIX-rule.md "🟡 proposed" 2026-01-01 "We will do it, narrowed."
  mkdir -p "$d/src"
  for i in a b c d e; do printf '%s\n' "$i" > "$d/src/$i.py"; done
  rb_commit "$d" "narrow and implement"
  rb_adr "$d" 0001-rule.md "🟢 accepted" 2026-02-01 "We will do it, narrowed."
  rb_adr "$d" FIX-rule.md "🟢 accepted" 2026-02-01 "We will do it, narrowed."
  printf 'more\n' >> "$d/src/a.py"
  rb_commit "$d" "accept"
  printf '| [0001](0001-rule.md) | Rule |\n' >> "$d/docs/adr/README.md"
  printf 'more\n' >> "$d/src/b.py"
  rb_commit "$d" "index"
  rb_adr "$d" "0002-a b.md" "🟡 proposed" 2026-01-01 "Spaced."
  rb_adr "$d" "FIX-a b.md" "🟡 proposed" 2026-01-01 "Spaced."
  rb_adr "$d" "0003-ü.md" "🟡 proposed" 2026-01-01 "Umlaut."
  rb_adr "$d" "FIX-ü.md" "🟡 proposed" 2026-01-01 "Umlaut."
  printf 'x\n' > "$d/src/ü.py"
  printf 'x\n' > "$d/src/tab	name.py"
  rb_commit "$d" "names — über"
  rb_adr "$d" FIX-rule.md "🟢 accepted" 2026-02-01 "We will do it, reworded."
  rb_commit "$d" "reword"
  printf 'fix\n' >> "$d/src/c.py"
  rb_commit "$d" "fix after review"
  head="$(sp_git "$d" rev-parse --short HEAD)"
  rb_note "$d" at-head.md "## Review\n\n- Reviewed at: \`$head\` (the head)"
  rb_note "$d" earlier.md "* Reviewed at: $(sp_git "$d" rev-parse HEAD~2), before the fixes"
  rb_note "$d" upper.md "Reviewed at: $(sp_git "$d" rev-parse HEAD | cut -c1-12 | tr a-f A-F)"
  rb_note "$d" crlf.md "Summary.\r\n\r\nReviewed at:\t$head\r"
  rb_note "$d" name.md "Reviewed at: work"
  rb_note "$d" base.md "Reviewed at: $(sp_git "$d" rev-parse --short base)"
  rb_note "$d" none.md "No review line.\nReviewed at:\nReviewed at: \`\`"
  rb_note "$d" bogus.md "Reviewed at: 0123456789abc"
  rb_note "$d" first.md "- Reviewed at: HEAD\n- Reviewed at: $head"
  sp_git "$d" tag abcdef0
  rb_note "$d" ref.md "Reviewed at: abcdef0"
}

# rb_merges <dir> — on the branch: a topic merged with a review at its tip, one reviewed before its
# last commit, which mixes ADR text with code, one without a line, a commit outside a merge, and
# main, which gained a commit, taken up by a merge.
rb_merges() {
  local d="$1"
  rb_main "$d"
  sp_git "$d" switch -q -c topic1
  change "$d" src/one.py one
  change "$d" src/one.py two
  sp_git "$d" switch -q work
  sp_git "$d" merge -q --no-ff -m "Merge topic1" \
    -m "## Review" -m "- Reviewed at: \`$(sp_git "$d" rev-parse --short topic1)\`" topic1
  sp_git "$d" switch -q -c topic2
  change "$d" src/two.py three
  rb_adr "$d" FIX-rule.md "🟡 proposed" 2026-01-01 "We will do it, mixed."
  printf 'four\n' >> "$d/src/two.py"
  rb_commit "$d" "mixed on the topic"
  sp_git "$d" switch -q work
  sp_git "$d" merge -q --no-ff -m "Merge topic2" \
    -m "Reviewed at: $(sp_git "$d" rev-parse --short topic2~1)" topic2
  sp_git "$d" switch -q -c topic3
  change "$d" src/three.py five
  sp_git "$d" switch -q work
  sp_git "$d" merge -q --no-ff -m "Merge topic3" topic3
  change "$d" src/direct.py six
  sp_git "$d" switch -q main
  change "$d" main.txt seven
  sp_git "$d" switch -q work
  sp_git "$d" merge -q --no-ff -m "Merge branch 'main' into work" main
  rb_note "$d" at-head.md "Reviewed at: $(sp_git "$d" rev-parse --short HEAD)"
  rb_note "$d" earlier.md "Reviewed at: $(sp_git "$d" rev-parse --short HEAD~2)"
  rb_note "$d" none.md "No review line."
}

# A history apart from main, its root commit the text of an ADR with code and the README, whose
# release line nuc reads: --root lists it.
rb_orphan() {
  rb_main "$1"
  sp_git "$1" switch -q --orphan orphan
  sp_git "$1" checkout main -- README.md
  rb_adr "$1" FIX-other.md "🟡 proposed" 2026-01-01 "Apart."
  printf 'x\n' > "$1/apart.py"
  rb_commit "$1" "apart"
  change "$1" apart.py more
  rb_note "$1" root.md "Reviewed at: $(sp_git "$1" rev-parse --short HEAD~1)"
}

# The branch at main, with a hand-off.
rb_none() { rb_main "$1"; rb_note "$1" h.md "Reviewed at: work"; }

# A history without a main: nothing to compare.
rb_no_main() { rb_none "$1"; sp_git "$1" branch -q -m main trunk; }
