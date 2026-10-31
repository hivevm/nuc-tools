# Ticket: Decide which file an uncited ADR's finding names

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`traceability/coverage` reports an accepted ADR no test cites at the file `docs/adr`, a
directory, because the bash message names no file and the parity table can only place such a
message at a fixed path, as it does for `docs/adr-numbering`. G-6 asks a finding to name its
`file:line`, and ADR-0019 writes `<file>[:<line>]`; an editor that jumps to the location opens a
directory. Naming the ADR's file, as the exemption without a reason already does, reads better
and needs a mode of `tests/parity/normalize.tsv` that derives the path from the ADR's identifier
in the message.

Decide with the human between keeping `docs/adr` and naming the ADR's file. If the second, add
the table mode, its case in the harness's self-test, and move the finding.

## Acceptance criteria

- [ ] Given the human's decision, when it is taken, then the finding names the chosen path, a
  unit test and `tests/check.rs` pin it, and the parity harness shows no difference with the
  `coverage-uncited` variant on every tree.
- [ ] Given the choice to keep `docs/adr`, when it is taken, then the hand-off records why, and
  nothing else changes.

## Read first

- ADR-0019 (the line form), ADR-0022 (the table), ADR-0023.
- `compare` in `scripts/check-parity.sh` and the places of `tests/parity/normalize.tsv`.

## Blocked by

- None (can start immediately)

## Mode

with the human — the output a derived project reads is the human's to settle.
