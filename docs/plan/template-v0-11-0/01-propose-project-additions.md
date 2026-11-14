# Ticket: Propose the project's additions to the inherited ADRs as ADRs of its own

## What to build

The diff of step 1 of section 8 of the normalize skill of template release v0.9.1: the inherited
ADR-0001 to ADR-0006 and `AGENTS.md` of this repository against the template at v0.8.1. Each clause
the project added becomes a `proposed` ADR that extends or narrows a `NUC-` clause of v0.9.1. Among
them is the clause of ADR-0005 that keeps the template's numbers for `nuc`'s messages and the parity
corpus, restated so that the copies of old releases keep the numbers they copy. The ticket also
drafts the map of ADR-0007 to ADR-0028 to category identifiers, with the categories the project
adds, for the human to review. The ADRs travel in commits of ADRs only, on a branch of their own.
Ticket 02 stacks on it.

## Acceptance criteria

- [ ] Given the template cloned at v0.8.1, when the inherited ADRs and `AGENTS.md` are diffed
  against it, then the hand-off lists every added clause, with the `NUC-` clause it extends or
  narrows, or "none" with the evidence.
- [ ] Given each added clause, when it is proposed, then one `proposed` ADR per clause carries
  `Extends:` or `Narrows:` naming the `NUC-` clause, in commits of ADRs only.
- [ ] Given ADR-0007 to ADR-0028, when the map is drafted, then each has a category identifier, and
  the hand-off names every category beyond NUC, SEC, OPS, and API.
- [ ] Given the human's review, when the ADRs and the map are sent back or confirmed, then the
  ticket stops there. Nothing is migrated in this ticket.

## Read first

- Touches: `docs/adr/`, `AGENTS.md`
- The template at v0.9.1: section 8 of `.agents/skills/normalize/SKILL.md`, `docs/adr/README.md`
  (categories, `Extends:`, `Narrows:`), `docs/adr/NUC-base-is-fixed.md`, and the `NUC-` ADRs the
  shipped map `map-v0.9.0.txt` names for each of ADR-0001 to ADR-0006.
- The release notes of v0.9.0 (the tag annotation), "Taking it up".
- `docs/adr/0005-template-releases.md`, the bullet "A normalization keeps the template's numbers".
- Skill `propose-adr`.

## Blocked by

- None (can start immediately)

## Mode

with the human: every ADR here is the human's decision
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rules 3 and 4), and so is the category map.
