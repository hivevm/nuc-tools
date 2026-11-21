# Ticket: `nuc upgrade` lists the citations a re-cut leaves behind and the releases it skipped

## What to build

`nuc upgrade B` adds two kinds of judgment. Each line of a project file outside the base that names
a `NUC-` identifier B no longer has is a finding at its `file:line`, so that the maintainer moves
it by the map in B's notes. A take-up across releases between A and B, read from the tags of the
template under `refs/template/` or named by the version gap, is one finding that names them,
since their notes may ask for more than files. Neither is written.

## Acceptance criteria

- [ ] Given a project ADR citing `NUC-x §2` and a B without `NUC-x`, when `nuc upgrade B` runs,
      then one finding names that file and line, and the file is unchanged.
- [ ] Given a citation of a `NUC-` identifier B still has, when it runs, then there is no finding.
- [ ] Given an identifier that names an ADR of a copied record, inside a file
      TEST-copies-keep-old-identifiers covers, when it runs, then the result is what the ADR of
      ticket 01 decides for it.
- [ ] Given A and B one release apart, when it runs, then there is no finding for skipped
      releases; given a release between them, then there is one, naming it.

## Read first

- Touches: `src/upgrade.rs`, `src/shell/upgrade.rs`, `src/adr.rs`, `tests/contract.rs`
- The ADR of ticket 01; `docs/adr/TEST-copies-keep-old-identifiers.md`.
- Golden path: how `src/adr.rs` and the rule `docs/adr-refs` find the identifiers a line cites.
- Seam: the plan function on values; one binary test for the whole run.

## Blocked by

- docs/plan/nuc-upgrade/03-three-way-by-blob.md

## Mode

with the human: it reads the project's files in the shell module of tickets 02 and 03, an effect
DEV-one-agent-one-ticket keeps with the maintainer.
