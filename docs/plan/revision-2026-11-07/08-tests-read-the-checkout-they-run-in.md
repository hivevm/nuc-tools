# Ticket: Tests that read the tree read the checkout they run in, or say why not

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Five places in the tests take the root of the tree from `env!("CARGO_MANIFEST_DIR")`, fixed when
the test binary is built: `tests/support/sources.rs` `root`, `tests/adr_citations.rs` twice,
`tests/hooks.rs` for the shims, and `tests/devcontainer.rs`. During the work on the output
formats a temporary worktree was built into `/workspace/target`; cargo then ran those binaries
from `/workspace`, and `adr_citations` and `bash_cases` read the worktree's path, which was gone,
and failed without naming why, until `cargo clean -p nuc-tools`. While such a worktree still
exists the same tests read *its* tree and may pass on files that are not the ones under test.

Route the five through the one `root` of `tests/support/sources.rs`, and have it compare the
directory fixed at build time with the one cargo runs the test in (cargo sets
`CARGO_MANIFEST_DIR` in the environment of a test it runs, and runs it in that directory) and
fail, where they differ, with a message that names both and `cargo clean -p nuc-tools`.

## Acceptance criteria

- [ ] Given the tests, when their sources are read, then `env!("CARGO_MANIFEST_DIR")` appears
  only in `tests/support/sources.rs`.
- [ ] Given a test binary whose build-time root differs from the directory it runs in (a unit test
  of the comparison on two paths), when `root` is called, then it panics with a message naming
  both directories and `cargo clean -p nuc-tools`; on a normal run every test passes as before.

## Read first

- Touches: `tests/support/sources.rs`, `tests/adr_citations.rs`, `tests/hooks.rs`,
  `tests/devcontainer.rs`
- ADR-0026 (the environment the tests run under).
- The *test pattern* of `docs/ARCHITECTURE.md`.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket.
