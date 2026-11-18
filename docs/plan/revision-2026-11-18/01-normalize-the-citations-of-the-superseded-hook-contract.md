# Ticket: Move the citations of the superseded hook contract in the next normalization

## What to build

API-hook-contract-keyed-shims superseded API-hook-contract-names-check in #102. Accepted ADRs are
restated only by a normalization, so four of them still cite the superseded ADR in their bodies, as
the hand-off of #102 noted under Doubts:

- API-command-line-with-measure (twice);
- API-version-coupling-hook-defaults (three times);
- ARCH-rules-outcome-hooks-answer (three times);
- TEST-hook-scenario-parity (once).

`tests/adr_citations.rs` fails on such a citation only outside `docs/adr/`, so nothing holds them.
A reader who follows one lands on a superseded decision. The normalization moves each citation to
API-hook-contract-keyed-shims where the cited clause still holds there, and keeps the citation where
it means the history.

## Acceptance criteria

- [ ] Given a normalization the human has instructed, when it restates the four ADRs, then no
  accepted ADR cites API-hook-contract-names-check other than as the decision its successor
  supersedes, and no decision moves.

## Read first

- Touches: `docs/adr/` (in the normalization only)
- The `normalize` skill; the mechanics in `docs/adr/README.md`

## Blocked by

- None (can start immediately)

## Mode

with the human: a normalization runs only when the human instructs it.
