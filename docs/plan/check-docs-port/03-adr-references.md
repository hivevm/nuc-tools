# Ticket: Port the ADR reference, link agreement, and supersession checks

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`nuc check` reports what checks 5, 6, and 8 of `check-docs.sh` report: every `ADR-NNNN` and
`ADR-draft-<slug>` reference names an ADR that exists; a link whose text cites an ADR points at
that ADR's file; and a *Supersedes* field and the status line of each ADR it names agree.
Superseded and rejected ADRs are skipped where the bash skips them.

## Acceptance criteria

- [ ] Given each case of `test-check-docs.sh` for checks 5, 6, and 8, when the rule runs, then it
  reports the same finding or none.
- [ ] Given the corpus and a variant per check (a reference to a number that does not exist, a
  link to another ADR's file, a Supersedes field without its counterpart), when the harness
  runs, then it reports no difference.

## Read first

- ADR-0009 (the shape), ADR-0010 (git, where the rule reads it), ADR-0013, ADR-0019.
- The golden path and the test pattern in `docs/ARCHITECTURE.md`; the conventions on the shell
  reading for the rules and on porting the bash with its quirks.
- The checks this ticket ports in the template's `scripts/check-docs.sh` and their cases in
  `scripts/test-check-docs.sh`, at v0.5.0.

## Blocked by

- docs/plan/check-docs-port/02-adr-index.md

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
