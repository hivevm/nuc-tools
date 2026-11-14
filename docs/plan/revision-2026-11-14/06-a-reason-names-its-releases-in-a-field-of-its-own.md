# Ticket: A reason of the release table names its releases in a field of its own

<!-- A ticket of the design revision of 2026-11-14, in the shape of the plan-feature skill's
     ticket template, without a spec. Claim it by creating the branch
     ticket/revision-2026-11-14/<NN>-<slug> BEFORE any work, by the steps of the work-tickets
     skill. The branch that lands it deletes this file. Its sections are the parts a ticket
     carries, as docs/adr/NUC-large-work-is-planned.md decides (clause 3). -->

## What to build

A row of `tests/parity/releases.tsv` names the releases its reason covers in a field of its own,
"-" for every release, so that no code reads a list of releases out of the reason's prose. Today a
reason that opens with `vX.Y.Z[,vX.Y.Z...]:` covers only those releases, a grammar stated in the
table's header comment and parsed by `covered` in `tests/releases.rs`. Pull request #80 took five
rounds of review, four of them on that parser: a reason that exempted the whole row (3ba72e7), a
malformed prefix that fell back to covering every release (522363c), a release-shaped opening that
did not parse and still covered every release (d7773be), and letters stuck to a release read as
prose (2647bff). Each round found a new input that fell back to the widest cover. A field of its
own has nothing to fall back from: a release that does not parse is refused.

## Acceptance criteria

- [ ] Given the table, when it is read, then each row has its covered releases in a field of its
      own, and no reason opens with a list of releases.
- [ ] Given a covered-releases field that names a release that does not parse, when
      `tests/releases.rs` runs, then it fails and names the row.
- [ ] Given the rows of today, when they are converted, then each covers the same releases as
      before, which the hand-off lists row by row.
- [ ] Given `scripts/check-parity.sh`, which reads the first four fields of the table, when the
      new field is added, then it still reads those four and the parity self-test passes.

## Read first

- Touches: `tests/parity/releases.tsv`, `tests/releases.rs`, `scripts/check-parity.sh`,
  `scripts/test-check-parity.sh`
- API-version-coupling-hook-defaults, which binds the table; the Review
  section of the hand-off of #80 for the four cases.

## Blocked by

- None (can start immediately)

## Mode

with the human — DEV-one-agent-one-ticket allows no unattended ticket, and the table is bound by
API-version-coupling-hook-defaults.
