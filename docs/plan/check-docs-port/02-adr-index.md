# Ticket: Port the ADR index, numbering, and header checks

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`nuc check` reports what checks 1, 2, and 7 of `check-docs.sh` report: every ADR and draft listed
once in the index with its status and *Applies to* as the file states them, in the table its
status belongs to; the files numbered 0001..N with matching headings and well-formed drafts; and
each header with one legend emoji and its word, every field in the template's form, and no
leftover instruction paragraph. Each check is a rule `docs/…`, one per behaviour where the
releases of the corpus differ.

## Acceptance criteria

- [ ] Given each case of `test-check-docs.sh` for checks 1, 2, and 7, when the rule runs on its
  text, then it reports the same finding or none.
- [ ] Given the corpus and a variant per check (a row missing from the index, a gap in the
  numbering, a status with two emoji), when the harness runs, then it reports no difference.
- [ ] Given an ADR that cannot be read as UTF-8 text the way the bash reads it, when `nuc check`
  runs, then it refuses with exit 2 and names the file (Q-1).

## Read first

- ADR-0009 (the shape), ADR-0010 (git, where the rule reads it), ADR-0013, ADR-0019.
- The golden path and the test pattern in `docs/ARCHITECTURE.md`; the conventions on the shell
  reading for the rules and on porting the bash with its quirks.
- The checks this ticket ports in the template's `scripts/check-docs.sh` and their cases in
  `scripts/test-check-docs.sh`, at v0.5.0.
- Glossary: rule ID, finding.

## Blocked by

- docs/plan/check-docs-port/01-corpus-breadth.md

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
