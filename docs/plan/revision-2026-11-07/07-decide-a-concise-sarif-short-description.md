# Ticket: Decide whether SARIF's short description takes only the first sentence of a reason

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

ADR-0027 gives a rule's whole `nuc explain` reason, two to four sentences, as its SARIF
`shortDescription`, its `fullDescription`, and its `help`. GitHub's documentation calls the short
description "a concise description of the rule" and shows it where a reader scans a list of
alerts; the full reason there repeats the full description word for word. The doubt was carried
in the pull request of ADR-0027 and not decided.

This is a change of an accepted ADR, so it is the human's to decide first:

1. With the human, decide whether `shortDescription` takes the first sentence of the reason (up
   to the first `. ` followed by a capital letter, or the whole reason where there is none),
   or stays as it is. If it stays, delete this ticket; nothing else changes.
2. If it changes: propose, through `propose-adr`, an ADR that supersedes ADR-0027 and restates it
   with the one clause changed, in commits of ADRs only, and hand those to the human before code.
3. After the human's review, implement it in `src/format.rs`, with the unit test and the SARIF
   expectations of `tests/formats.rs` moved to the first sentence, and move the citations of
   ADR-0027 to the new number.

## Acceptance criteria

- [ ] Given the human's decision, when it is to keep the whole reason, then this file is deleted
  and nothing else changes.
- [ ] Given a decision to shorten it, when `nuc check --format sarif` runs on the fixtures of
  `tests/formats.rs`, then each rule's `shortDescription` is the first sentence of its reason
  and its `fullDescription` and `help` the whole reason, and the superseding ADR is accepted by
  the human before the change merges.

## Read first

- Touches: `docs/adr/`, `src/format.rs`, `tests/formats.rs`
- ADR-0027 in full; the record's mechanics in `docs/adr/README.md`.
- The table of `reportingDescriptor` properties on GitHub's *SARIF support for code scanning*.

## Blocked by

- docs/plan/revision-2026-11-07/06-every-reason-fits-github-sarif-limits.md

## Mode

with the human — it changes an accepted ADR, and ADR-0007 allows no unattended ticket.
