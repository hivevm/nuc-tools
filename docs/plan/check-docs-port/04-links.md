# Ticket: Port the relative-link and section-reference checks

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`nuc check` reports what checks 3 and 4 of `check-docs.sh` report: every relative Markdown link
in a tracked Markdown file resolves, outside fenced code and placeholders; and every section
reference matches a numbered heading of `AGENTS.md`, whose headings run 1..N.

## Acceptance criteria

- [ ] Given each case of `test-check-docs.sh` for checks 3 and 4, when the rule runs, then it
  reports the same finding or none.
- [ ] Given a link into a submodule or a directory, and one with a title, when `nuc check` runs on
  a fixture repository, then it resolves them as the bash does.
- [ ] Given the corpus and a variant per check (a link to a deleted file, a reference to a section
  that does not exist), when the harness runs, then it reports no difference.
- [ ] Given the variant that deletes ADR-0003, the gap in the numbering tickets 02 and 03 left for
  the links it breaks, when the harness runs, then it reports no difference.

## Read first

- ADR-0009 (the shape), ADR-0010 (git, where the rule reads it), ADR-0013, ADR-0019.
- The golden path and the test pattern in `docs/ARCHITECTURE.md`; the conventions on the shell
  reading for the rules and on porting the bash with its quirks.
- The checks this ticket ports in the template's `scripts/check-docs.sh` and their cases in
  `scripts/test-check-docs.sh`, at v0.5.0.

## Blocked by

- docs/plan/check-docs-port/01-corpus-breadth.md

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
