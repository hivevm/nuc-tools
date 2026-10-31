# Ticket: Cite the criteria the port serves from the tests that decide them

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The closed spec served G-1, G-6, Q-1, Q-2, Q-7, and Q-11, and the traceability check lists all
of them as without a test. Some are decided already and only lack the marker: the refusals of
`tests/check.rs` and `tests/contract.rs` decide classes of Q-1; `nuc explain` and the line form of
a finding decide G-6; the parity harness's self-test decides that a difference fails, the core of
Q-2. Add `Verifies:` markers where a test decides the criterion as its text says, and name in the
hand-off each criterion that stays pending and why, G-1 among them, which the corpus decides only
for the trees it pins.

## Acceptance criteria

- [ ] Given `scripts/check-traceability.sh`, when it runs, then G-6, Q-1, and Q-2 are no longer
  listed as without a test, each cited by a test that decides it as the specification words it.
- [ ] Given a criterion no test decides whole, when the ticket closes, then it carries no marker
  and the hand-off says what is missing.

## Read first

- ADR-0002 and its Enforcement.
- `docs/SPECIFICATION.md`, G-1, G-6, Q-1, Q-2.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
