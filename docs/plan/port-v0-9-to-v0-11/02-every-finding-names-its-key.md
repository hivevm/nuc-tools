# Ticket: Every finding names its key, and the rules run against the template's tables

## What to build

The tracer bullet. Every finding of every rule `nuc` has carries the reason the bash of v0.9.1 names
for it, and the output shows the key `<rule>:<reason>` in text, JSON, and SARIF as
API-command-line-keyed-findings and API-output-formats-carry-reason say. The fixture tables
`check-docs-record.txt`, `check-traceability.txt`, and `check-shell.txt` of template v0.11.0 are
copied into the tests with the blob of each. A runner builds each case's tree from the table's steps
through the real git CLI, runs the rules as v0.11.0, and compares keys and subjects. Every case that
does not agree yet is listed as pending with its rule; that list is what the later tickets empty.

## Acceptance criteria

- [ ] Given a tree with a finding of any rule, when `nuc check` runs in each format, then each
      finding shows the key the bash of v0.9.1 names for the same finding.
- [ ] Given a copied table whose bytes differ from the blob recorded for it, when the tests run,
      then a test fails and names the table.
- [ ] Given the copied tables, when the runner runs, then each case either agrees or is listed as
      pending with its rule, and a listed case that agrees fails the test, so the list cannot go
      stale.
- [ ] Given the cases of `shell/bash-floor` and `shell/root`, when the runner runs, then they are
      listed as not ported under ARCH-rules-need-only-git, not as pending.
- [ ] Given the parity corpus, when the parity harness runs, then it agrees as before, reading the
      key where the bash names one.

## Decided

- The human accepted API-command-line-keyed-findings, API-output-formats-carry-reason, and
  TEST-cases-from-template-tables on 2026-11-15. They stay `proposed` until this ticket implements
  them: the change that moves the `Verifies:` markers of API-command-line-with-hook-signal and
  API-output-formats-short-description to their successors flips the three to accepted and the two
  old ones to superseded, header lines only
  (NUC-decisions-are-proposed-then-accepted §5).

## Read first

- Touches: `src/finding.rs`, `src/format.rs`, `src/rules/`, `tests/formats.rs`, `tests/contract.rs`,
  `tests/hooks.rs`, `docs/adr/`, `tests/` (a new runner and the copied tables),
  `tests/parity/normalize.tsv`, `scripts/check-parity.sh`
- API-command-line-keyed-findings, API-output-formats-carry-reason, and
  TEST-cases-from-template-tables.
- The golden path: tests/bash_cases.rs and its list tests/bash_cases.tsv, which hold the unit tests
  to the copied self-test cases of v0.8.1; the fixture helpers of tests/support/.
- The template's tables and the base fixture each describes in its header, at v0.11.0.

## Blocked by

- None (can start immediately)

## Mode

with the human: it changes the output contract and the parity harness, the fail-closed path that
DEV-one-agent-one-ticket keeps with the maintainer.
