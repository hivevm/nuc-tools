# Ticket: Port the criteria part, and run both bash checks in the harness

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The tracer bullet through the new area `traceability`:

- `nuc check` runs a rule `traceability/criteria`. It reports what part 1 of
  `check-traceability.sh` reports:
  - a list item under Goals or Quality Goals of the specification without its bold identifier;
  - an identifier in the wrong section, or used twice;
  - a missing specification.
- `nuc explain` gives the rule's reason.
- The parity harness runs both `check-docs.sh` and `check-traceability.sh` of each tree. It
  compares their findings together with one `nuc check`, and a tree fails when either script
  fails.

## Acceptance criteria

- [ ] Given each case of `test-check-traceability.sh` for part 1, when the rule runs, then it
  reports the same finding or none.
- [ ] Given a tree with no specification, when `nuc check` runs with the rule on, then it reports
  the finding the bash reports.
- [ ] Given the corpus, when the harness runs, then it runs both bash checks on every tree and
  reports no difference.
- [ ] Given a variant that drops the bold identifier of a goal, when the harness runs, then the
  bash and `nuc` both report it, and the harness shows no difference.
- [ ] Given a tree where `check-traceability.sh` fails and `check-docs.sh` passes, when the
  harness's self-test runs, then the bash side counts as failed.

## Read first

- **ADRs:** ADR-0009 (the shape), ADR-0013, ADR-0019, ADR-0022 (the harness), ADR-0023 (the
  decision the check holds).
- **The golden path and the test pattern** in `docs/ARCHITECTURE.md`.
- **The conventions** on the shell reading for the rules, and on porting the bash with its
  quirks.
- **The bash:** the template's `scripts/check-traceability.sh` and
  `scripts/test-check-traceability.sh`. Both are the same at v0.5.0 and v0.6.1.
- **The harness:** `scripts/check-parity.sh` and its self-test, where they run `check-docs.sh`
  and read its `  - ` lines.

## Blocked by

- None (can start immediately)

## Mode

with the human. ADR-0007 keeps every ticket with the human, one at a time.
