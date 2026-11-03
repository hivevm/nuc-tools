# Ticket: Cite the ADRs in force, and fail on a citation of a superseded one

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Three supersessions in this count each left citations of the old ADR outside `docs/adr/`. The
convention *A rule changes everywhere it is stated* did not prevent that, and no check sees it.
`docs/adr-refs` only asks that the number exists, and `traceability/citations` only reads
`Verifies:` markers.

- `tests/functional_core.forbidden`, line 1, cites ADR-0009. 59cb87e moved the other citations to
  ADR-0025 and missed this one.
- `.github/workflows/checks.yml`, lines 19 and 240, cite ADR-0012 for the parity harness, two
  supersessions back. ADR-0022 decides it now.
- `.github/workflows/checks.yml`, lines 15, 74, and 158, and `.githooks/pre-push`, line 49, cite
  ADR-0002. That is the template's number for what ADR-0023 restates here. The take-up of v0.6.1
  kept the template's numbers in messages that the parity table matches, and moved the markers
  and the Markdown. Decide whether comments in files the template ships follow the messages or
  the markers.

Correct them. With the fix, ship the check the mistake asks for
([`AGENTS.md` §5](../../../AGENTS.md#5-quality-bar--definition-of-done)): it fails on a
citation of an ADR whose status is superseded or rejected, in a tracked file outside `docs/adr/`.
It exempts the template's numbers in the places a list beside it names, such as the messages of
`nuc` and of `tests/parity/normalize.tsv`, and the fixtures and variants that name a corpus
tree's files. Where it runs is the session's to propose: a test of the crate, or a check under
`scripts/` with its job and its line in `scripts/check-all.sh` (ADR-0023, ADR-0008).

## Acceptance criteria

- [ ] Given the tree, when the change lands, then no file outside `docs/adr/` cites ADR-0009 or
  ADR-0012, and each ADR-0002 that remains is one the exemption list names.
- [ ] Given a fixture that cites a superseded ADR outside the record, when the check runs, then it
  fails, naming the file and the line. Given the same citation at a place the list exempts, then
  it passes.
- [ ] Given a supersession that leaves one citation behind, when the gate runs, then it fails.

## Read first

- ADR-0023 and ADR-0018.
- The conventions *A rule changes everywhere it is stated* and *A self-test asserts what it
  builds*.
- The description of the take-up of v0.6.1, commit ba9dfa1.
- `tests/parity/normalize.tsv` and `tests/parity/variants.tsv`, for the template's numbers that
  stay.

## Blocked by

- None (can start immediately)

## Mode

with the human — the exemptions and where the check runs need the human's word, and ADR-0007
keeps every ticket with the human, one at a time.
