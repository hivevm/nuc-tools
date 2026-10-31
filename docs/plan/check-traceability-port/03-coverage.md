# Ticket: Port the coverage of the accepted ADRs, and name the pending criteria

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`nuc check` runs a rule `traceability/coverage`. It reports what parts 2 and 4 of
`check-traceability.sh` report:
- an accepted ADR that no marker cites, unless its Enforcement section declares it not
  mechanically decidable;
- such a declaration that gives no reason.

Proposed ADRs are out of the rule's scope, as they are out of the bash's.

The criteria no marker cites go into the summary `nuc check` writes to standard error, as the
bash's closing line names them. They are never a finding and never change the exit status.

## Acceptance criteria

- [ ] Given each case of `test-check-traceability.sh` for the coverage and the exemption, when
  the rule runs, then it reports the same finding or none.
- [ ] Given an accepted ADR that only a proposed ADR's test cites, or that only a Markdown file
  names, when the rule runs, then it is reported as uncited.
- [ ] Given criteria no marker cites, when `nuc check` runs with the rule on, then the summary
  names them in the order of the specification, and the exit status is the one without them.
- [ ] Given the corpus and two variants (a marker removed so that an accepted ADR is uncited, and
  an exemption stripped of its reason), when the harness runs, then it reports no difference.

## Read first

- **ADRs:** ADR-0009, ADR-0019 (what goes to standard error), ADR-0022, ADR-0023.
- **The bash:** parts 2 and 4 of the template's `scripts/check-traceability.sh`, and their
  cases in `scripts/test-check-traceability.sh`.
- **How the ADR rules of the area `docs`** read the status line and a section of an ADR.

## Blocked by

- docs/plan/check-traceability-port/02-citations.md

## Mode

with the human. ADR-0007 keeps every ticket with the human, one at a time.
