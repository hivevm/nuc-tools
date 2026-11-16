# Ticket: `nuc measure` prints a draft of a template finding

## What to build

An option prints a draft of a template finding in the five fields of the template's issue form for
findings: Template release, Concerns, Finding, Measures, and Proposal.

- Template release is the release the README's line names.
- Measures carries the figure and the four parts of the selection, with the tool names and sizes of
  the largest outputs.
- Concerns, Finding, and Proposal stand as placeholders for the human.
- The draft carries no subagent description, branch name, session identifier, or path, so that
  nothing of the project leaves with it.
- It is printed on the standard output, not written. The design revision's step on template
  findings can redirect it into its draft file.

## Acceptance criteria

- [ ] Given fixture transcripts and a README naming v0.11.1, when `nuc measure` runs with the draft
  option, then it prints the five headings in order, with `v0.11.1` under Template release and the
  figures under Measures.
- [ ] Given the canary fixtures, and a branch name, subagent description, and path each holding a
  canary, when it prints the draft, then no canary is in it.
- [ ] Given a README with no release line, when it prints the draft, then Template release says so,
  and the command exits 0.

## Read first

- Touches: the output of the counting module, the shell's option and its reading of the README, the
  test file, `tests/contract.rs`, `README.md`
- NUC-template-learns-from-projects, clauses 2 to 4; the design-revision skill, step 6

## Blocked by

- docs/plan/nuc-measure/03-selection-and-breakdown.md

## Mode

with the human — what leaves the project through a finding is the human's to judge
(NUC-template-learns-from-projects §4).
