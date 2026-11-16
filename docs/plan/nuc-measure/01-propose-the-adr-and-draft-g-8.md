# Ticket: Propose the ADR of `nuc measure` and draft G-8

## What to build

`nuc measure` is the first command of `nuc` that reads outside the repository, and the
specification has no goal for it. Before any code, this ticket proposes the ADR and drafts the
goal, for the human to decide.

- **The ADR**, through `propose-adr`: `nuc measure` reads a harness's transcripts outside the
  repository, read-only, through one adapter per harness. It fails closed on what it cannot read,
  and prints numbers and names only, never content.
  - Its interrogation starts from the decisions of the spec, which the human gave on 2026-11-16:
    the figure, the selection, the default directory, fail closed, and the output.
  - Its Enforcement names the tests of tickets 02 and 03.
- **The draft of G-8** in the hand-off, with the criteria it bears on (Q-1, Q-7, Q-8, Q-10, Q-11).
  It also names the accepted ADRs a change of the specification would move. The human lands it.
- **API-command-line-keyed-findings** names the commands of `nuc`. The ticket says whether the new
  command needs that ADR superseded or a revision of the new one, and proposes what it needs.

## Acceptance criteria

- [ ] Given the ADR, when `scripts/check-docs.sh` runs, then it passes with the ADR proposed and its
  index row in place.
- [ ] Given the hand-off, when the human reads it, then it carries the wording of G-8 and the ADRs a
  change of the specification moves.
- [ ] Given the human's decision, when it is recorded, then the ADR is accepted or the ticket hands
  back the reasons it was not.

## Read first

- Touches: `docs/adr/`, `docs/adr/README.md`
- docs/plan/nuc-measure/spec.md; the `propose-adr` skill
- API-command-line-keyed-findings, ARCH-rules-outcome-hooks-answer, OPS-crates-pass-deny-and-audit,
  NUC-template-learns-from-projects
- Q-1, Q-7, Q-8, Q-10, Q-11 of `docs/SPECIFICATION.md`

## Blocked by

- None (can start immediately)

## Mode

with the human — the ADRs and the specification are the human's to decide (DEV-one-agent-one-ticket,
NUC-documents-are-fixed §2).
