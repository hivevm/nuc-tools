# Ticket: Compare bash and nuc on the bytes the bash reads otherwise

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Every review of tickets 02 to 08 found a byte-level divergence from the bash that no test had
tried: a byte of no valid UTF-8, the byte 0x1F the record's awk splits on, a last line `read`
drops, octal and overflow in bash arithmetic. The corpus holds none of these bytes, so the parity
harness cannot see them. Decide with the human how parity tries them, then build it: variants of
`tests/parity/variants.tsv` that write such bytes where `nuc` emulates the bash (an unterminated
last line, a line anchor with a leading zero, 0x1F outside the record), and, where `nuc` refuses
instead (invalid UTF-8 in a wide line, 0x1F in an ADR), a way to state that refusal as the
expected result. The harness today fails on any exit 2 of `nuc`, so the second part may need a
revision of ADR-draft-parity-variants-name-their-trees.

## Acceptance criteria

- [ ] Given a variant per byte class each rule emulates, when the harness runs, then bash and
  `nuc` agree on it.
- [ ] Given a byte class `nuc` refuses, when the harness runs on it, then it checks that `nuc`
  exits 2 and fails when `nuc` exits 0 or 1, as the human decides.
- [ ] Given the harness's self-test, when it runs, then it covers each new kind of variant.

## Read first

- ADR-0020 and ADR-draft-parity-variants-name-their-trees.
- The conventions on porting the bash with its quirks.
- The refusals of `src/shell/` whose messages say the bash reads a byte otherwise, and the
  tests in `tests/check.rs` that pin them.

## Blocked by

- None (can start immediately)

## Mode

with the human — the refusal case is a decision about the harness, and ADR-0007 keeps every
ticket with the human.
