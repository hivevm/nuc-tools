# Ticket: The releases check holds the libraries a sensor sources

## What to build

API-sensor-cores-ask-the-shell names the releases at which each sensor changed, and "a change of
the sensor's script or of a library it sources counts". The parity harness compares a sensor's
row of `tests/parity/releases.tsv` with the trees of the corpus through `check_body` of
`scripts/check-parity.sh`. It compares the sensor's script alone, comment lines left out. A change
to `scripts/lib/git.sh`, `scripts/lib/root.sh`, `scripts/lib/bash.sh`, or `scripts/lib/adr.sh`
that a sensor sources shows in no comparison. The sessions of tickets 02, 05, and 07 of
`sensors-under-nuc-sense` checked those libraries by hand. They wrote what they found into the
reasons of the rows `sense/tests-kept`, `sense/plain-rules`, and `sense/review-binding`, and the
session of ticket 07 said in its hand-off that it had checked each library by hand. A template
release that changes only a library would pass the harness unseen.

Make the releases check compare, for a row `sense/<name>`, the script with each library it
sources, as the tree of each release has them. A release at which only a library changed must
then stand in the row, or in its reason.

## Acceptance criteria

- [ ] Given a copy of a template tree whose `scripts/lib/git.sh` changed in a line that is not a
  comment, and a `sense/` row whose sensor sources it, when the harness's releases check runs and
  the row lacks that release, then the check fails and names the library.
- [ ] Given the corpus as pinned, when the harness runs, then the releases check passes, with
  the rows corrected where a library changed at a release the row lacks.

## Read first

- Touches: `scripts/check-parity.sh`, `scripts/test-check-parity.sh`, `tests/parity/releases.tsv`,
  `tests/releases.rs`
- API-sensor-cores-ask-the-shell; TEST-parity-harness; the convention "A self-test asserts what
  it builds" in `docs/CONVENTIONS.md`

## Blocked by

- None (can start immediately)

## Mode

with the human: DEV-one-agent-one-ticket allows no unattended work, and a row the check corrects
may show a release at which a sensor of `nuc` reports otherwise than its bash.
