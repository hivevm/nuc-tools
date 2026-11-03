# Ticket: Keep the identifiers of the template and the corpus out of a normalization's map

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`.agents/skills/normalize/remap.sh` rewrites `ADR-NNNN` and an ADR's file name in every tracked
text file outside `docs/adr/`. Its header assumes that "the self-tests assemble the ones in their
fixtures at run time". In this repository many identifiers are not references to its own record:

- **The template's numbers in messages**, which the bash of every corpus tree writes, and which
  `nuc` and `tests/parity/normalize.tsv` must match byte for byte, such as "(ADR-0002)" in
  `docs/no-draft-on-main` and "(ADR-0005)" in `docs/template-release`.
- **The files of a corpus tree** that `tests/parity/variants.tsv` edits, such as
  `docs/adr/0002-decisions-verified-by-tests.md`.
- **About 89 literal fixture identifiers** in the unit tests, as `# ADR-0001: A`, which #26 left
  as they are. They pass today only because those numbers exist in this record.

The take-up of v0.6.1 found that the script would rewrite the variants, and so did not use it.
The next normalization deletes ADR-0002, ADR-0005, ADR-0009, ADR-0012, and ADR-0020 and
renumbers the rest. With the script as it stands, it would break the parity harness and turn
the fixtures into references to numbers that no longer exist. The held normalization items (the
ADR-0013/0021 fold, the ADR-0022 Decision, the ADR-0006 ordinal) do not cover this.

Before the next normalization, make the identifiers that are not this record's survive it.
Assemble the fixture identifiers through `src/testing.rs` and `tests/support/`, and write the
template's numbers so that the session can tell them from this record's own, by the one list of
the places that carry the template's numbers, which ticket 03 of this revision introduces with
its check. Then decide with the human how a normalization
of this repository leaves `tests/parity/` and those constants alone. Because `remap.sh` is the
template's, a change to it goes to hivevm/nuc.

## Acceptance criteria

- [ ] Given a map of a trial normalization that deletes ADR-0002 and renumbers the rest, when
  `remap.sh` runs on a copy of the tree, then the parity harness and the suite still pass, or
  the steps the normalization takes by hand are written down where the normalize skill's
  session will read them.
- [ ] Given the unit tests, when the change lands, then no test writes an identifier of this
  record as a literal it does not mean.

## Read first

- [`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 5, and the `normalize` skill with its
  `remap.sh`.
- ADR-0018 and ADR-0024.
- The Doubts of #26 and of the take-up of v0.6.1, commit ba9dfa1.
- `src/testing.rs`, `tests/support/mod.rs`, `tests/parity/normalize.tsv`,
  `tests/parity/variants.tsv`.

## Blocked by

- docs/plan/revision-2026-11-03/03-superseded-adr-citations.md

## Mode

with the human — a normalization runs only on the human's instruction, and a change to the
template's script is the template's.
