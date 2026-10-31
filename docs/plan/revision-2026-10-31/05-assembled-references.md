# Ticket: Assemble the references in Rust test literals in one place

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

This repository's own `check-docs.sh` reads every text file, the Rust tests included, so a test
literal that names an ADR, a draft, or a section of `AGENTS.md` it does not mean fails the gate
(ADR-0018 keeps the bash as the gate). The port answered that five times over, each its own way:
a helper `ADR-<key>` in `docs_adr_refs`, `docs_adr_link_targets`, and `docs_adr_supersessions`, a
draft heading in `docs_adr_numbering`, `missing_reference` in `tests/check.rs`, and two `concat!`
splits in `docs_section_refs`. The gate caught each case, so no further check is due; what is due
is one way to write such a literal. Give the unit tests and the integration tests one helper each
side can reach, as the template's self-tests do with `adr_id`, and use it everywhere.

## Acceptance criteria

- [ ] Given the tests, when one names an ADR identifier, a draft identifier, or a section
  reference it does not mean, then it builds it through the one helper, and no `concat!` split
  or local assembling function is left.
- [ ] Given the gate, when `scripts/check-all.sh` runs, then it passes with the tests unchanged in
  what they assert.

## Read first

- ADR-0018; ADR-0011 for where the integration tests' helpers live.
- `scripts/test-check-traceability.sh`, whose `adr_id` and `mark` show the template's form.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
