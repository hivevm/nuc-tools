# Ticket: The JSON writer joins its lists in one place

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`src/format.rs` writes four JSON arrays, the SARIF rules and results and the JSON findings and
notes, each with the same loop that writes a comma before every element but the first, and six
`write!` calls each with `.expect("a String takes every write")`. Logic written four times is
where the next array, or a fix to one, goes wrong in the copy that was not touched.

Give the writer one function that writes a JSON array of items, each by a closure, and use it for
the four, and keep the `expect` in as few places as the writes allow. No byte of any output
changes.

## Acceptance criteria

- [ ] Given the unit tests of `src/format.rs` and `tests/formats.rs`, when they run unchanged, then
  they pass.
- [ ] Given `src/format.rs`, when it is read, then the comma-before-element loop stands once.

## Read first

- Touches: `src/format.rs`
- ADR-0027 (*Without a crate*, fixed field order and the same bytes for the same findings).

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket.
