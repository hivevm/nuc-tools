# Ticket: Read the crate's sources for the structural tests in one place

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Two test files read the crate's own sources, and each has its own copy of the readings:

- `tests/functional_core.rs` has `sources(dir, skip, found)`, `rule_id`, which takes the string
  of the first line holding `const RULE_ID`, and `rule_modules`.
- `tests/bash_cases.rs` has `sources(dir, out)` and a `rule_id` that wants
  `pub const RULE_ID: &str = "` at the start of a line. It told the rule modules apart by their
  displayed path.

The second copy carried a mistake the first did not. It compared the displayed path with
`src/rules/`, which on Windows reads `src\rules\`, so it found no rule there. The Portability
job on Windows caught it before the merge of #29 (bdaf85a). Give the readings one definition in
`tests/support/`: every `.rs` file under a directory, the rule modules by path, and a module's
rule ID. Both tests then use it.

## Acceptance criteria

- [ ] Given the suite as it stands, when the readings have moved, then every test passes
  unchanged, on the four platforms of Portability.
- [ ] Given `tests/`, when the change lands, then each reading is defined once, and the two
  readings of a rule ID agree, or the one left says why it reads as it does.

## Read first

- ADR-0011, on where the test helpers live, and ADR-0025, for the structural test.
- `tests/functional_core.rs`, `tests/bash_cases.rs`, and `tests/support/mod.rs`.
- Commit bdaf85a.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
