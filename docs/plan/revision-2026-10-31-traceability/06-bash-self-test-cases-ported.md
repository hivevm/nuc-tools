# Ticket: Hold each rule's unit tests to the cases of the bash self-test

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The conventions have a rule's unit tests copy the cases of the bash self-test, and every ticket
of the port asked for it. The review of `traceability/citations` still found two cases left out:
an invalid identifier beside valid ones, and a G-n filed under Quality Goals. Nothing but a
reviewer compares the two lists. Decide with the human how a check holds them together, then
build it. One way: each copied case names the bash case it copies, by the name the bash
self-test gives it, and a test reads the self-test of the carried release and fails on a case
of the ported part no unit test names. Another: the parity harness runs the fixtures the bash
self-test builds. Either reads the template's self-tests, which this repository carries.

## Acceptance criteria

- [ ] Given the human's choice, when the check is built, then it fails on a ported rule whose
  unit tests leave out a case of its part of the bash self-test, and passes on every rule as the
  suite stands.
- [ ] Given the conventions, when the check lands, then the entry on copying the bash's cases
  names the check, or gives way to it.

## Read first

- ADR-0023, ADR-0018 (the gates run the bash), ADR-0022.
- The conventions on porting the bash with its quirks.
- `scripts/test-check-docs.sh` and `scripts/test-check-traceability.sh`, how they name their
  cases.

## Blocked by

- None (can start immediately)

## Mode

with the human — how the check reads the template's self-tests is the human's to choose.
