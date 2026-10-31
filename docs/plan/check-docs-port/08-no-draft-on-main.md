# Ticket: Port the check of drafts on main

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`nuc check` reports what check 16 of `check-docs.sh` reports: a change that lands on `main`
carries no draft ADR, read from the branch the change lands on and the draft state of its pull
request as the bash reads them from the environment, which the shell reads and passes as values.

## Acceptance criteria

- [ ] Given each case of `test-check-docs.sh` for check 16, when the rule runs on the values the
  shell read, then it reports the same finding or none.
- [ ] Given the environment of a pull request into `main` marked ready and a tree with a draft,
  when the harness runs bash and `nuc` with it, then it reports no difference.

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
