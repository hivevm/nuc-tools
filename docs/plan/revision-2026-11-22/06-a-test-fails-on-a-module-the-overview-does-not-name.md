# Ticket: A test fails on a module the overview does not name

## What to build

A change to the structure updates `docs/ARCHITECTURE.md`, as
[`AGENTS.md` §5](../../../AGENTS.md#5-quality-bar--definition-of-done) asks. No check sees it.
The first review round of ticket 02 of `nuc-upgrade` found `src/upgrade.rs` and
`src/shell/upgrade.rs` added and the overview not updated, a must-fix. A mistake a check would
have caught becomes a check.

A test of this repository, beside `tests/functional_core.rs`, lists the modules of the crate: each
`src/*.rs` but `lib.rs` and `main.rs`, each `src/shell/*.rs` but `mod.rs`, and each directory
module under `src/`. It fails on one that `docs/ARCHITECTURE.md` does not name, by its path or by
its path in Rust, such as `shell::upgrade`. A module the overview names under another name, as a
sensor by its name, stands in a list beside the test with its reason.

## Acceptance criteria

- [ ] Given a module of the crate that `docs/ARCHITECTURE.md` does not name, when the test runs,
  then it fails and names the module.
- [ ] Given the tree as it stands, when the test runs, then it passes, each exception in its list
  with a reason.
- [ ] Given the test, when it is read, then it cites NUC-documents-are-fixed with a `Verifies:`
  marker, and takes the root from `tests/support/sources.rs`.

## Read first

- Touches: `tests/`, `tests/support/sources.rs`, `docs/ARCHITECTURE.md`
- NUC-documents-are-fixed §4; `tests/functional_core.rs`, as the pattern of a test that reads the
  sources and keeps a list beside it

## Blocked by

- None (can start immediately)

## Mode

with the human: DEV-one-agent-one-ticket allows no unattended work.
