# Ticket: `pre-merge-commit`, and every shim fails closed

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`pre-merge-commit` through its shim and `nuc hook pre-merge-commit`: a merge on `main` is refused,
a merge into any other branch runs the project's documentation check through the shim and is
refused while it is red, and a repository without that check merges. Every reference shim now
fails closed: with `nuc` absent from the `PATH` it stops with a message naming how to install
it, and with a `nuc.toml` whose minimum is above the running `nuc` the hook stops through `nuc`
with the update hint (G-5).

## Acceptance criteria

- [ ] Given the scenario harness, when it runs the hook self-test's merge cases, then they agree
  with the bash of each pinned release, `merge.ff` false included.
- [ ] Given a fixture with each reference shim installed and no `nuc` on the `PATH`, when git runs
  each hook, then each exits non-zero with the install message, and the merge driver's shim stops
  the merge.
- [ ] Given a `nuc.toml` whose minimum is above the running version, when git runs each hook,
  then each exits non-zero with the update hint, before any check runs.
- [ ] Given the shim tests, when the suite runs on Linux x64, Linux arm64, macOS, and Windows x64,
  then they pass with no bash on the `PATH` of the process under test (Q-4), and a shim told to
  run a check that cannot run, bash missing included, refuses.
- [ ] Given git failing, a panic in `nuc hook` (exit 101), and an exit code the contract does not
  name, when each shim meets them, then it refuses with a message naming the cause (Q-1).

## Read first

- Touches: the hook module under `src/`, the reference shims, the scenario harness, `tests/`.
- The template's `.githooks/pre-merge-commit` and the merge cases of `scripts/test-git-hooks.sh`.
- ADR-0023 (fail closed), ADR-0021 (the hook contract), Q-4 in `docs/SPECIFICATION.md`.
- Golden path: the `pre-commit` hook and shim of ticket 02.

## Blocked by

- docs/plan/hooks-port/02-tracer-pre-commit.md

## Mode

with the human — ADR-0007 allows no unattended work; the shims become the gate every derived project
runs, and G-5 decides what a project without `nuc` sees.
