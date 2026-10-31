# Ticket: Fail the build on a read of the shell that skips the tree's guard

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The port broke "never read outside the repository" three times: `AGENTS.md` read through a
symbolic link, an absolute Project Layout path that replaced the root, and directories linked
outside the repository listed. Each was found by a review, never by a check. Extend the
structural test of `tests/functional_core.rs` so that inside `src/shell/` a call that reads a
file's bytes or lists a directory, such as `fs::read` or `fs::read_dir`, appears only in the
functions of the tree module that test the path first. A test of existence, as the bash's `-e`,
`-f`, and `-d`, stays free.

## Acceptance criteria

- [ ] Given a shell source that calls `fs::read` or `fs::read_dir` outside the guarded
  functions, when the structural test runs on it, then it fails and names the function.
- [ ] Given the shell as it stands after ticket 01, when the structural test runs, then it
  passes.
- [ ] Given the allowed functions, when the test lists them, then the list is kept beside the
  test, as `functional_core.forbidden` is.

## Read first

- ADR-0009 and its Enforcement; `tests/functional_core.rs` and its list.
- The conventions on the shell reading for the rules.

## Blocked by

- docs/plan/revision-2026-10-31/01-shell-tree-module.md

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
