# Ticket: Port the prose width check

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`nuc check` reports what check 15 of `check-docs.sh` reports: no line of Markdown prose over 100
columns, with fenced code, front matter, tables, headings, ADR header fields, lines that cannot
break, superseded and rejected ADRs, and files marked generated exempt, and indented code read as
prose, as the bash reads them.

## Acceptance criteria

- [ ] Given each case of `test-check-docs.sh` for check 15, when the rule runs, then it reports
  the same finding or none.
- [ ] Given a line of multi-byte characters, when the rule counts its columns, then it counts them
  as the bash does in the locale the harness sets.
- [ ] Given the corpus and a variant (a prose line made 101 columns), when the harness runs, then it
  reports no difference.

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
