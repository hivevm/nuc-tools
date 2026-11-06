# Ticket: <title>

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The merge driver of the ADR index through `nuc` (G-3): git calls the driver's shim with the base,
ours, theirs, and the three labels, and `nuc` writes into ours the union of both sides' rows where
the prose merged on its own equals the prose of the union, and otherwise the usual conflict
markers with git's labels and a non-zero exit, as the bash driver does.

## Acceptance criteria

- [ ] Given two branches that each append a row to the same table, when one merges the other, then
  the index holds both rows and the merge passes, with the same bytes as the bash driver writes.
- [ ] Given two branches that both reword the same rule, when they merge, then the merge stops
  with markers labelled as git passes them, as the bash driver's.
- [ ] Given a rule changed next to a row the other side appended, when they merge, then it stops
  with a conflict rather than keeping both versions.
- [ ] Given the scenario harness, when it runs the driver's cases, then they agree with the bash
  of each pinned release, and the driver's shim fails closed without `nuc` (ticket 02).

## Read first

- Touches: a merge module under `src/`, the reference shims, the scenario harness, `tests/`.
- The template's `.githooks/merge-adr-index`, `.gitattributes`, and the driver's cases in
  `scripts/test-git-hooks.sh`.
- G-3 in `docs/SPECIFICATION.md`; ADR-0011 (git CLI, `git merge-file`), the hook contract ADR.
- Golden path: the shims of tickets 01 and 02.

## Blocked by

- docs/plan/hooks-port/02-pre-merge-commit-and-fail-closed.md

## Mode

with the human — ADR-0007 allows no unattended work; the driver decides what a merge of the record
keeps.
