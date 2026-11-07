# Ticket: The formats carry the notes end to end

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

ADR-0027 puts the rules' notes into the JSON document's `"notes"` and keeps them out of the SARIF
log, on the standard error in every format. The unit tests of `src/format.rs` show a note in the
JSON, but every fixture of `tests/formats.rs` runs with no note, so no test shows through the
binary that the notes the shell gathers reach the JSON in the order the summary lists them, stay
out of the SARIF, and leave the standard error as the text leaves it.

Add a fixture to `tests/formats.rs` whose specification has a criterion no `Verifies:` marker
cites, so that `traceability/coverage` notes it, and assert all three.

## Acceptance criteria

- [ ] Given a fixture whose run gives at least one note, when `nuc check` runs with each format,
  then the JSON's `"notes"` holds the notes the text summary names, in its order, the SARIF holds
  none, and the exit code and standard error equal those of the text.

## Read first

- Touches: `tests/formats.rs`
- ADR-0027 (*JSON*, *SARIF*, *The same run*); ADR-0026 for the fixture.
- The *test pattern* of `docs/ARCHITECTURE.md`; the fixture helpers in `tests/support/`.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket.
