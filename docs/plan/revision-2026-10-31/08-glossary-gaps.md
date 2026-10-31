# Ticket: Close the glossary's gaps the port opened

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The port uses terms the glossary does not define, against rule 6 of
[`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules): **variant**, an edit that makes a ported rule fire
on a copy of a corpus tree, used by ADR-0020, the draft that supersedes it, the harness, and its
files; the **carried release**, the template release a tree names, which
ADR-0021 turns into a behaviour; and **not decided**, the wording
of every refusal of the shell. The entry **parity harness** still says it runs on "the same tree"
only. Add the entries, correct the stale one, and move code and documents to the chosen words where
they use a synonym.

The port of `check-traceability.sh` uses three more: **marker**, the `Verifies:` line that
cites identifiers, which ADR-0023 names; **pending criterion**, a criterion no marker cites, which
the summary of `nuc check` names; and **area**, the first part of a rule ID, `docs` or
`traceability`, which the README and the specs use.

## Acceptance criteria

- [ ] Given `docs/GLOSSARY.md`, when the ticket lands, then it defines the six terms, each in
  one or two sentences, and the parity harness entry names the variants.
- [ ] Given a synonym the glossary lists under _Avoid:_, when the tree is searched for it, then
  no code, comment, or document uses it.

## Read first

- ADR-0001 (the glossary's role); `docs/GLOSSARY.md`.
- ADR-0020, ADR-0022,
  ADR-0021.

## Blocked by

- None (can start immediately)

## Mode

with the human — the words are the human's to settle, and ADR-0007 keeps every ticket with the
human.
