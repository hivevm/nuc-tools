# Ticket: A check holds that only the tests' helper names the build-time root

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

During the work on the output formats a test binary built from a temporary worktree into the
same target read the worktree's tree, which was gone, and failed without naming why. Since
ticket 08 of the revision of 2026-11-07, `tests/support/sources.rs` `root` compares the
directory fixed when the binary was built with the one cargo runs it in and fails naming both
and `cargo clean -p nuc-tools`. That guard holds only for the tests that ask `root`: the ticket's
first criterion, that `env!("CARGO_MANIFEST_DIR")` stands in `tests/support/sources.rs` alone, is
true today and held by nothing. A new test that takes the root from the macro again reads a stale
checkout silently, the mistake the guard was built for.

Add a test that reads the Rust sources under `tests/` and `src/` and fails, naming the file and
the line, on a use of the macro with that variable outside `tests/support/sources.rs`. The test
builds the pattern it looks for from pieces, so that its own source does not match it.

## Acceptance criteria

- [ ] Given the tree as it is, when the test runs, then it passes.
- [ ] Given a test source outside `tests/support/sources.rs` that takes its root from
  `env!("CARGO_MANIFEST_DIR")` (a source fixture of the test, or the line added by hand and
  named in the hand-off), when the test runs, then it fails naming that file and line.

## Read first

- Touches: `tests/fixtures.rs` (beside the unit test of the comparison) or
  `tests/functional_core.rs`, `tests/support/sources.rs` (`rust_files`)
- ADR-0026 (the environment the tests run under), ADR-0002 (a check that holds a decision is
  cited by a test).
- The *test pattern* of `docs/ARCHITECTURE.md`.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket.
