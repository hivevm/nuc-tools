# TEST-template-table-cases: The rules are tested against copies of the template's fixture tables of the newest release `nuc` ports, each pinned by its blob, with every case that does not agree listed as pending on a ticket, as not ported, or as decided by an ADR in force

- **Status:** 🟢 accepted
- **Date:** 2026-11-15
- **Deciders:** Markus Brigl
- **Applies to:** the copies of the template's fixture tables under `tests/tables/`, their blob list `tests/tables.blobs`, the list of cases that do not agree `tests/tables.cases`, the runner `tests/tables.rs`, and the check of the blobs in `scripts/check-parity.sh`

## Context

G-1 and Q-2 ask that a rule report what the bash check of the release a tree carries reports. The
parity harness decides that on the trees of its corpus and the variants derived from them
([TEST-parity-harness](TEST-parity-harness.md)), so a case that no tree shows goes undecided. The
convention "A port copies the bash, quirks included" of `docs/CONVENTIONS.md` fills that gap up to
template release v0.8.1: a rule's unit tests copy the cases of the bash self-tests, and
`tests/bash_cases.rs` holds them to the list of those cases.

From release v0.9.1 on, the template keeps the cases of its checks in fixture tables under
`scripts/fixtures/`: `check-docs-record.txt` with 208 cases at v0.11.0, `check-traceability.txt`,
and `check-shell.txt`. Each case builds a tree from steps on a base fixture its table's header
spells out, and expects an exit and the key and subjects of each finding, never wording. Every
finding of `nuc` names the same key
([API-command-line-with-measure](API-command-line-with-measure.md)), so `nuc` can run the
template's own cases. A table changes with each release, and the cases of one release describe
that release's checks.

The tests build their repositories through the real git CLI under a cleared environment
([TEST-fixtures-with-own-home](TEST-fixtures-with-own-home.md)). Q-11 keeps the checks off the
network, and the tests run offline but for the parity harness, which fetches its corpus into a
cache. Q-7 forbids `nuc` to run repository content; the steps of `check-shell.txt` write scripts,
which a rule of `nuc` reads and never runs. An ADR identifier inside a copy of a template release is
not a citation of this record
([TEST-copies-keep-old-identifiers](TEST-copies-keep-old-identifiers.md)).

Five cases of `check-docs-record.txt` at v0.11.0 cannot agree, and no ticket ports them. 'no README
to carry the Template release line' expects a finding where `nuc` refuses a tree without a README.
'a release line that says 'not forked'' expects `docs/base-blobs` on a tree whose README names
v0.8.1, which `nuc` checks as v0.8.1, a release without that check. Three cases of
`docs/template-release` expect its checks of v0.10.0 on a tree whose README names v0.1.0 or v0.2.0,
which `nuc` checks as that release, before those checks. All five follow
[API-version-coupling-hook-defaults](API-version-coupling-hook-defaults.md): a rule whose check
changed checks a tree as the release its README names, and a tree whose README names none is not
decided. The template's bash knows only its own release. Neither case is a gap a ticket closes, nor
a rule `nuc` leaves to the bash.

## Decision

We will test the rules against copies of the template's fixture tables of the newest release `nuc`
ports, each pinned by its blob at that release's tag, run by a runner that builds each case's tree
and compares keys and subjects, with every case that does not agree listed as pending on the ticket
that ports it, as not ported with its reason, or as decided by the ADR in force that decides it
otherwise, with its reason.

- **The copies:** `tests/tables/` holds the tables of the template's checks that `nuc` ports, as
  they stand at the tag of the newest release `nuc` ports, byte for byte. Today these are
  `check-docs-record.txt`, `check-traceability.txt`, and `check-shell.txt` of v0.11.0. A port of a
  later release replaces them in the change that adds the release; the tables of an older release
  are not kept.
- **The pin:** `tests/tables.blobs` names, per line, the release, the blob of a table at its tag,
  and its file under `tests/tables/`. The runner fails on a copy whose blob differs from its line,
  and on a copy or a line without the other. `scripts/check-parity.sh` reads the release's tag from
  the template's repository in its cache, fetching the tag where the cache lacks it, and fails where
  the tag holds another blob at that path, or where it cannot read the tag. It needs no tree of that
  release pinned in the corpus, since the copies may lead the pins while a port is under way.
- **The runner:** for each case, it builds the tree the table's header and the case's steps
  describe through the real git CLI, runs the rules of `nuc` as the release of the copies, and
  compares the exit and each expected key and subject, by the reading the table's header gives. It
  runs no step that the table's header does not describe, and no script a step writes.
- **The cases that do not agree:** `tests/tables.cases` lists each case of a copy that does not
  agree, by its table and name, with its rule and one of:
  - `pending <ticket>`, the path of the ticket under `docs/plan/` that ports it;
  - `not ported <reason>`, for a case of a rule `nuc` leaves to the bash;
  - `decided by <ADR> <reason>`, for a case whose expectation an ADR in force decides otherwise for
    `nuc`: the ADR's identifier, then why it decides the case so.

  The runner fails on a case that neither agrees nor is listed, on a listed case that agrees, on a
  line that names no case of a copy, and on a pending case whose ticket file is gone. It fails on a
  decided case whose ADR has no file under `docs/adr/` or is not accepted. So no case is pending
  once its port has landed, and none stays decided once its ADR is no longer in force. A case that
  agrees for a reason other than the template's stands in no list.
- **The releases before:** the copied self-test cases of v0.8.1 and `tests/bash_cases.rs` stay for
  the releases up to v0.8.1, whose bash names no key. The releases between v0.8.1 and the newest
  are held by the parity harness on their pinned trees.

**Out of scope:** the tables of checks `nuc` does not port, such as `check-devcontainer.txt` and the
plain-rules sensor's table; the cases a bash self-test keeps inline; which rules `nuc` ports.

## Alternatives considered

- **The cases pending on a ticket for good** — no new status, but a pending case says a port
  will make it agree, and the runner would fail once the ticket's file is gone.
- **The cases as not ported** — no new status, but `not ported` says the rule is the bash's,
  while `nuc` ports `docs/template-release` and `docs/base-blobs` and decides these cases on
  purpose.
- **The copies edited, the cases deleted** — nothing to list, but a copy would no longer be
  the template's blob, and the pin is what keeps it so.
- **A decided case without a reason, the ADR alone** — shorter, but the line would not say which
  part of the ADR decides the case, and a reader would open the ADR to find it.
- **The ADR left to review, unchecked by the runner** — simpler, but a case would stay decided by an
  ADR that was superseded, which no one notices until the next port.
- **Cases that agree by chance listed too** — a record of each, but the runner would then accept a
  listed case that agrees, and lose the check that ends a pending case.

- **The tables of every release whose tables changed** — each release held to its own cases, but
  three copies of mostly the same cases to keep at v0.11.0, one more with each release, where the
  parity corpus already pins a tree of each release.
- **The tables read from the corpus's cache of the template** — no copy to keep, but the tests then
  need the network to fill the cache, and `cargo test` would depend on the harness having run.
- **The pin checked offline only** — no network, but whether the blob is the template's would rest
  on the review alone.
- **Pending cases without a ticket, each with a reason** — flexible, but a gap would stay unseen
  once the work that left it ends.
- **The cases copied into unit tests, as up to v0.8.1** — no runner to keep, but each case
  rewritten by hand, its key and subjects read off the table, for 250 cases and again with every
  release.
- **The parity harness alone** — no copy and no runner, but a case no tree of the corpus shows
  stays undecided, and a variant per case would grow the corpus run by the cases.

## Sources / Prior art

- The template's `scripts/fixtures/check-docs-record.txt`, `check-traceability.txt`, and
  `check-shell.txt` at v0.11.0, whose headers give the form of a case and the base fixture, and
  `scripts/test-check-docs.sh`, which reads the first.
- The tag message of the template's v0.9.1: the fixture tables expect keys and data, never wording.
- The five cases of `check-docs-record.txt` at v0.11.0 that cannot agree, and
  API-version-coupling-hook-defaults, which decides them.
- `git hash-object`, which computes the blob of a file as git stores it.

## Consequences

- Positive: each rule is held to the template's own cases, beyond what the corpus shows (G-1, Q-2),
  without a case rewritten by hand. A copy edited by hand or left behind its release is caught, and
  so is a gap that outlives its ticket. A case `nuc` decides otherwise on purpose names the ADR
  that decides it, and stays checked against that ADR's status. The tests stay offline (Q-11).
- Negative / trade-offs: the runner has to read every step the tables describe, and a release that
  adds a step needs the runner to learn it. The copies are files of the template inside this
  repository, replaced with each release. The pin's tie to the template is checked only where the
  parity harness runs. An older release's cases that a later release changed are held only by its
  tree in the corpus.
- Follow-ups: none.

## Enforcement

`tests/tables.rs` cites this ADR with `Verifies: TEST-template-table-cases`. It fails on a copy
whose blob differs from `tests/tables.blobs`, on a copy or a line of that list without the other, on
a case that neither agrees nor is listed in `tests/tables.cases`, on a listed case that agrees, on a
line of that list that names no case, and on a pending case whose ticket file does not exist. It
fails on a decided case whose ADR has no file under `docs/adr/` or whose status is not accepted.
`scripts/check-parity.sh` fails where a blob of `tests/tables.blobs` is not the blob of its table at
the release's tag of the template, and where it cannot read that tag; `scripts/test-check-parity.sh`
decides both.
