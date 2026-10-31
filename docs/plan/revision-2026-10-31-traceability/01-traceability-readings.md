# Ticket: Read the criteria and the citations outside the rule modules

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The three rules of the area `traceability` call each other: `traceability/citations` reads the
criteria through `traceability_criteria::read`, and `traceability/coverage` reads them the same
way and the cited identifiers through `traceability_citations::citations`. The reading of the
specification's criteria and the reading of the markers are each one concept the three rules
share, kept in the module of one of them. The rules of the record share theirs in `src/adr.rs`,
outside `src/rules/`, as the overview describes. Move the two readings into one module outside
`src/rules/`, named for the traceability of the record, so that each rule module holds its rule
and calls no other rule. It changes no behaviour.

## Acceptance criteria

- [ ] Given the suite as it stands, when the readings have moved, then every test passes
  unchanged and the parity harness shows no difference.
- [ ] Given `src/rules/`, when the ticket lands, then no rule module uses another rule module.
- [ ] Given `docs/ARCHITECTURE.md`, when the module lands, then the Rules block names it beside
  `src/adr.rs`.

## Read first

- ADR-0009 (the shape), ADR-0023 (the decision the rules hold).
- `src/rules/traceability_criteria.rs`, `traceability_citations.rs`, `traceability_coverage.rs`.
- docs/plan/revision-2026-10-31/03-bash-byte-readers.md, which moves the byte readers these
  readings use.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
