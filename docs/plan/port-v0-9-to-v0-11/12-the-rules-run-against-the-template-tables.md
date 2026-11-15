# Ticket: The rules run against the template's fixture tables

## What to build

Cut from ticket 02, whose key part landed: every finding already names the key the bash names.

The fixture tables `check-docs-record.txt`, `check-traceability.txt`, and `check-shell.txt` of
template v0.11.0 are copied into `tests/tables/`, byte for byte, with the blob of each in
`tests/tables.blobs`. A runner builds each case's tree from its table's base fixture and steps
through the real git CLI, runs the rules as the release of the copies, and compares the exit and
each expected key and subject, as the table's header reads them. Every case that does not agree
yet is listed in `tests/tables.cases` as pending on the ticket that ports it, or as not ported
under ARCH-rules-need-only-git; that list is what the later tickets empty. The parity harness
checks each blob against the template's tag.

## Acceptance criteria

- [ ] Given a copied table whose bytes differ from the blob recorded for it, or a copy or line
      without the other, when the tests run, then a test fails and names the table.
- [ ] Given the copied tables, when the runner runs, then each case either agrees or is listed in
      `tests/tables.cases`; a listed case that agrees, a line that names no case, and a pending
      case whose ticket file is gone each fail the test.
- [ ] Given the cases of `shell/bash-floor` and `shell/root`, when the runner runs, then they are
      listed as not ported under ARCH-rules-need-only-git, not as pending.
- [ ] Given a blob of `tests/tables.blobs` that the template's tag does not hold at that path, or a
      tag the harness cannot read, when `scripts/check-parity.sh` runs, then it fails; its
      self-test decides both.
- [ ] Given the runner and its lists, when this lands, then TEST-cases-from-template-tables is
      flipped to accepted.

## Decided

- The human accepted TEST-cases-from-template-tables on 2026-11-15. It stays `proposed` until this
  ticket implements it, and the change that adds `tests/tables.rs` with its `Verifies:` marker flips
  it, its header line only (NUC-decisions-are-proposed-then-accepted §5).
- The human chose on 2026-11-15 to cut this from ticket 02, which had grown past one session.

## Read first

- Touches: `tests/tables/`, `tests/tables.blobs`, `tests/tables.cases`, `tests/tables.rs`,
  `tests/support/`, `scripts/check-parity.sh`, `scripts/test-check-parity.sh`, `docs/adr/`
- TEST-cases-from-template-tables, ARCH-rules-need-only-git, TEST-fixtures-with-own-home,
  TEST-copies-keep-old-identifiers.
- The golden path: tests/bash_cases.rs and its list tests/bash_cases.tsv; the fixture helpers of
  tests/support/.
- The headers of the three tables at v0.11.0, read with `git show` from a clone of the template,
  never run: each spells out its base fixture and its steps. The record table alone has about 30
  kinds of step, git's branches and merges among them.

## Blocked by

- None (can start immediately)

## Mode

with the human: it adds the tests that every later ticket is held to, and touches the parity
harness, which DEV-one-agent-one-ticket keeps with the maintainer.
