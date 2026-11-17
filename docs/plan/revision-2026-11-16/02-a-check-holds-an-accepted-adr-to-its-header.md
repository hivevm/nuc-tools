# Ticket: A check fails a change to the text of an accepted ADR outside a normalization

## What to build

NUC-decisions-are-proposed-then-accepted lets the flip to accepted change the header line only
(clause 5). A commit that changes an ADR's text changes nothing outside `docs/adr/` (clause 4). In
the work on `nuc measure`, the commit that flipped two ADRs also rewrote two sentences of one of
them, beside code. No check caught it; the review of the ticket did, and the change was taken back
(126f57c, afff3ab).

The check reads a branch's commits against its base and fails where an ADR whose status was
accepted before the commit changes in any line but its header. An ADR that was proposed before
the commit is not covered. A commit of a normalization is exempt, by the marker the `normalize`
skill gives its commits. It runs in `scripts/check-all.sh`.

## Acceptance criteria

- [ ] Given a branch with a commit that changes a line of an accepted ADR's body, when the check
  runs, then it fails naming the commit, the ADR, and the line.
- [ ] Given a commit that only flips a proposed ADR's status line to accepted, or a normalization's
  commit, when it runs, then it passes.
- [ ] Given its self-test, when `scripts/check-all.sh` runs, then the cases above pass.

## Read first

- Touches: `scripts/` (a new check and its self-test), `scripts/check-all.sh`,
  `.github/workflows/checks.yml`
- NUC-decisions-are-proposed-then-accepted, clauses 4 and 5; the `normalize` skill; the existing
  checks of `scripts/` as the pattern

## Blocked by

- None (can start immediately)

## Mode

with the human: it adds a gate to `scripts/check-all.sh` and CI, which DEV-one-agent-one-ticket
keeps with the maintainer.
