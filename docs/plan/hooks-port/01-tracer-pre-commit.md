# Ticket: <title>

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The first path through the hooks, end to end: git runs the reference shim for `pre-commit`, the
shim hands over to `nuc hook pre-commit`, and `nuc` refuses a commit on `main` and passes one on
any other branch, as the bash hook does. On a commit that concludes a conflicted merge, `nuc`
tells the shim to run the project's full check with `--changed`, and the shim runs it and refuses
on red with the bash's message. A scenario harness runs the hook self-test's `pre-commit` cases
once with the bash hooks of each template release the corpus pins and once with the shim and the
built `nuc`, and fails on any difference.

Before any code, two ADRs are proposed with the human (`propose-adr`), as the spec's *ADR impact*
names them: hook-scenario parity, and the hook contract between shim and `nuc`, including how
`pre-push` will later be told to run or skip the checks and record a green tree, so that the
contract is decided once for every hook.

## Acceptance criteria

- [ ] Given the two drafts, when this ticket starts implementing, then both are proposed in
  `docs/adr/` with their index rows and the human has reviewed them.
- [ ] Given a fixture repository on `main` with the reference shims as its hooks, when
  `git commit` runs, then it exits non-zero with the refusal of a `hook/` rule for a commit on
  main, and on another branch it passes.
- [ ] Given a conflicted merge resolved and staged on a branch, when `git commit` concludes it,
  then the shim runs the fixture's full check with `--changed`, passes on green and refuses on red,
  and `nuc` itself runs no script of the repository (a canary script stays unrun when `nuc hook
  pre-commit` is called directly).
- [ ] Given the scenario harness and the corpus's template trees, when it runs, then every
  `pre-commit` case agrees on exit status and mapped refusal for each release, and a scenario
  whose outcomes differ, an unknown bash message, and a release tree the corpus lacks each fail it.
- [ ] Given `check-all.sh` and the parity job, when either runs, then the scenario harness runs
  with them, and check 9 still holds the required checks together.

## Read first

- Touches: `src/main.rs`, `src/lib.rs`, a new hook module under `src/`, `src/shell/git.rs`, a new
  directory of reference shims, `tests/parity/`, `scripts/check-parity.sh`,
  `scripts/check-all.sh`, `docs/adr/`.
- The template's `.githooks/pre-commit` and `scripts/test-git-hooks.sh` at each pinned release,
  from the parity cache.
- ADR-0013 (exit codes, output), ADR-0014 (fail closed), ADR-0015 (the tree harness this sits
  beside), ADR-0016 (the gates never run the build), ADR-0011, ADR-0012.
- Glossary: shim, gate. The golden path: the tree harness and its normalization table.

## Blocked by

- None (can start immediately)

## Mode

with the human — two ADRs are proposed and reviewed before the implementation, and the hook
contract fixes the interface every later ticket builds on.
