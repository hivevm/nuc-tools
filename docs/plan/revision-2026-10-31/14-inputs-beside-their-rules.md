# Ticket: Keep each rule's input type beside its rule

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`src/rules/mod.rs` defines the input types of single rules, `CheckLists` for
`docs/check-lists`, `Layout` for `docs/project-layout`, and `Skills`, `Pointer`, and `Entry` for
`docs/skill-pointers`, while `Landing` lives in `docs/no-draft-on-main`, the rule it serves. The
module that lists every rule grows by a type with each port. Move each type that one rule reads
into that rule's module, as `Landing` is, and keep in `src/rules/mod.rs` what several rules
share.

## Acceptance criteria

- [ ] Given the crate, when a type of the inputs is read by one rule only, then it is defined in
  that rule's module.
- [ ] Given the suite, when the types have moved, then every test passes unchanged.

## Read first

- ADR-0009; the golden path in `docs/ARCHITECTURE.md`, step 2.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
