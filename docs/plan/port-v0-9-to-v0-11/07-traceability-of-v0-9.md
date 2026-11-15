# Ticket: Traceability as v0.9

## What to build

On a tree of v0.9.0 or later, the traceability rules check as the bash of v0.9.1:

- `traceability/citations`: a `Verifies:` marker names `<CAT>-<slug>`, `G-n`, or `Q-n`, and
  anything else is refused as malformed; no draft identifiers;
- `traceability/coverage`: identifiers in the new form;
- `traceability/sensor-tests`, a new rule: each `scripts/sensor-X.sh` has a
  `scripts/test-sensor-X.sh` that cites NUC-checks-are-bash.

## Acceptance criteria

- [ ] Given the cases of the copied `check-traceability.txt`, when the runner runs, then they agree
      and leave the pending list.
- [ ] Given a sensor without its test, when `nuc check` runs, then `traceability/sensor-tests`
      reports it with the key the bash names.
- [ ] Given a tree on v0.8.1, when `nuc check` runs, then the traceability findings are those of
      today.
- [ ] Given `traceability/sensor-tests`, when the contract test runs, then the ID is listed and
      ships off.

## Read first

- Touches: `src/traceability.rs`, `src/rules/traceability_citations.rs`,
  `src/rules/traceability_coverage.rs`, a new rule module, `src/rules/mod.rs`,
  `tests/contract.rule-ids`, `tests/parity/releases.tsv`
- The template's `scripts/check-traceability.sh`, diffed v0.8.1 to v0.9.1.
- Seam: the table runner.

## Blocked by

- docs/plan/port-v0-9-to-v0-11/03-adrs-named-by-category-and-slug.md

## Mode

with the human: the parser of markers in repository files, protected under
DEV-one-agent-one-ticket.
