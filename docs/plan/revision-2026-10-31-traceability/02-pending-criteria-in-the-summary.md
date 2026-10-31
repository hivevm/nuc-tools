# Ticket: Decide how a rule reports what is not a finding

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

ADR-0009 has a rule take values and return findings. The criteria no test cites are no finding
(ADR-0023), so the port has the shell's `summary` test whether the rule with the ID
`traceability/coverage` is selected and then call `traceability_coverage::pending` by name. The
shell now knows one rule by its ID, and a rule produces a second kind of output outside its
`Rule`. It also names the pending criteria when the run fails, where the bash's closing line
names them only on a pass; that goes beyond the bash on the standard error only, which the
harness does not compare.

Decide with the human between keeping it as it is, written down in the conventions, and giving
`Rule` a way to return notes for the summary that the shell prints for any rule, with or without
the bash's restriction to a pass. The second widens what a rule returns, as
docs/plan/revision-2026-10-31/13-rule-refusals-beside-rules.md asks for refusals, so the two are
best decided together, and it may need a `proposed` ADR on the shape ADR-0009 decides.

## Acceptance criteria

- [ ] Given the human's decision, when it is taken, then it is recorded as a convention or a
  `proposed` ADR, and the summary follows it.
- [ ] Given criteria no marker cites, when `nuc check` runs with `traceability/coverage` on, then
  the summary still names them in the order of the specification, and the exit code is the one
  without them, as `tests/check.rs` pins today.

## Read first

- ADR-0009, ADR-0019 (the summary on the standard error), ADR-0023 (criteria are pending work).
- `summary` in `src/shell/mod.rs` and `pending` in `src/rules/traceability_coverage.rs`.

## Blocked by

- None (can start immediately)

## Mode

with the human — it may change the shape ADR-0009 decides, and the choice is the human's.
