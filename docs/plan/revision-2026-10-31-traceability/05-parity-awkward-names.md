# Ticket: Try the parity table on names the corpus lacks

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Two of the three reviews of the port found rows of `tests/parity/normalize.tsv` narrower than
the messages the bash prints: the citation rows did not read a citing path with a space or a
colon, nor an accepted draft, and the coverage rows did not read a draft slug with capitals or
a path with a space. The corpus holds no such name, so the harness passed on each. `compare` in
`scripts/check-parity.sh` also cuts a bash message at its first `: ` and reads a `nuc` line up to
its first colon, so a path holding a colon is placed wrong on both sides. Make the harness try
the names: variants of `tests/parity/variants.tsv` that write files whose names hold a space, a
colon followed by a space, and capitals in a draft slug, each where a ported rule names it, so
that a row too narrow, or a split in the wrong place, is a difference. Where `nuc` refuses such
a name instead, follow the decision of
docs/plan/revision-2026-10-31/04-byte-class-parity.md on stating a refusal.

Since the uncited ADR's finding names the ADR's file, the split at the first colon meets that
finding too: an ADR file named `docs/adr/0001-a:b.md` reads as "nuc output not understood" where
both sides named `docs/adr` before.

## Acceptance criteria

- [ ] Given the rows as they stood before the reviews' fixes, when the harness runs the new
  variants, then it reports a difference.
- [ ] Given the rows as they stand, when the harness runs on the corpus and every variant, then
  it reports no difference, or each difference left is filed with the place it comes from.
- [ ] Given a path with `: ` in it, when the harness compares a finding at that path, then bash
  and `nuc` are placed at the same file.

## Read first

- ADR-0022 (variants), ADR-0019 (the line form `nuc` writes).
- `scripts/check-parity.sh`, `scripts/test-check-parity.sh`, and the two tables under
  `tests/parity/`.
- docs/plan/revision-2026-10-31/04-byte-class-parity.md, the same gap for the bytes of a file.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, and a name the bash and `nuc` read
apart may need a decision of ticket 04 of the revision before it.
