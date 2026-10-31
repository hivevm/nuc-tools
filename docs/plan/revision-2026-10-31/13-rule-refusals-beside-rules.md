# Ticket: Decide where a rule's own refusals live

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The convention has the shell read and the rules decide, and every refusal end the run before a
rule runs. The refusals that belong to one rule have followed it into the shell:
`undecided_widths` asks `docs_prose_width` which lines it cannot count and refuses there,
`unit_separator_free` refuses 0x1F for the rules of the record, and `read_landing` refuses a
draft name with a newline for `docs/no-draft-on-main`. Each port added one, and the shell now
holds what each rule cannot decide beside what it reads. Decide with the human between keeping
them in the shell, written down in the conventions, and letting a rule return a refusal of its
own that the shell reports before printing any finding. The second changes what a rule returns,
which ADR-0009 words as findings, so it may need a proposed ADR.

## Acceptance criteria

- [ ] Given the human's decision, when it is taken, then it is recorded as a convention or a
  proposed ADR, and the refusals follow it.
- [ ] Given a tree that one rule cannot decide, when `nuc check` runs, then it still exits 2
  with no finding printed, as the tests in `tests/check.rs` pin today.

## Read first

- ADR-0009, ADR-0019 (exit 2).
- The conventions on the shell reading for the rules and on porting the bash.

## Blocked by

- docs/plan/revision-2026-10-31/01-shell-tree-module.md

## Mode

with the human — it may change the shape ADR-0009 decides.
