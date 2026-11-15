# Ticket: Every finding names its key, and the rules run against the template's tables

## What to build

The tracer bullet. Every finding of every rule `nuc` has carries the reason the bash of v0.9.1
names for it, and the output shows the key `<rule>:<reason>` in text, JSON, and SARIF as the ADR
accepted from ticket 01 says. The fixture tables `check-docs-record.txt`,
`check-traceability.txt`, and `check-shell.txt` of template v0.11.0 are copied into the tests with
the blob of each. A runner builds each case's tree from the table's steps through the real git
CLI, runs the rules as v0.11.0, and compares keys and subjects. Every case that does not agree
yet is listed as pending with its rule; that list is what the later tickets empty.

## Acceptance criteria

- [ ] Given a tree with a finding of any rule, when `nuc check` runs in each format, then each
      finding shows the key the bash of v0.9.1 names for the same finding.
- [ ] Given a copied table whose bytes differ from the blob recorded for it, when the tests run,
      then a test fails and names the table.
- [ ] Given the copied tables, when the runner runs, then each case either agrees or is listed as
      pending with its rule, and a listed case that agrees fails the test, so the list cannot go
      stale.
- [ ] Given the cases of `shell/bash-floor` and `shell/root`, when the runner runs, then they are
      listed with the reason of the ADR from ticket 01, not as pending.
- [ ] Given the parity corpus, when the parity harness runs, then it agrees as before, reading the
      key where the bash names one.

## Read first

- Touches: `src/finding.rs`, `src/format.rs`, `src/rules/`, `tests/formats.rs`, `tests/`
  (a new runner and the copied tables), `tests/parity/normalize.tsv`, `scripts/check-parity.sh`
- The ADRs accepted from ticket 01 on the key and on the tables.
- The golden path: tests/bash_cases.rs and its list tests/bash_cases.tsv, which hold the unit tests
  to the copied self-test cases of v0.8.1; the fixture helpers of tests/support/.
- The template's tables and the base fixture each describes in its header, at v0.11.0.

## Blocked by

- docs/plan/port-v0-9-to-v0-11/01-propose-the-adrs-of-the-port.md

## Mode

with the human: it changes the output contract and the parity harness, the fail-closed path that
DEV-one-agent-one-ticket keeps with the maintainer.
