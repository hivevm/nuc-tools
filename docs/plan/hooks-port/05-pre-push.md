# Ticket: `pre-push` with the record of green trees

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`pre-push` through its shim and `nuc hook pre-push`: a push to the remote's `main`, its deletion
included, is refused; a push that only deletes passes without the checks; a push that sends a
draft ADR gets the notice in the wording of the carried release, README up to v0.6.1 and
`CONTRIBUTING.md` from v0.7.0; a clean tree the record of green trees holds skips the checks;
otherwise the shim runs the full check with `--changed`, with git's local variables cleared, and
on green has `nuc` record the tree, keeping the last 50 in the shared git directory. The signal
and the record step are those of the hook contract ADR.

## Acceptance criteria

- [ ] Given the scenario harness, when it runs the hook self-test's push cases, then they agree
  with the bash of each pinned release: refusal of main, deletions, the draft notice per release,
  the green-tree record with an amend, an untracked file, and a new tree, and a push from a linked
  worktree that hands the checks no git environment; the draft notice and the skip notice agree
  as the harness compares notices.
- [ ] Given a decoy repository through `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and
  `GIT_CONFIG_PARAMETERS`, when `nuc hook pre-push` runs, then it decides on the repository it was
  pointed at and writes the record nowhere else (Q-8, Q-9).
- [ ] Given the record file written by hand with a tree, when the tree is pushed, then the checks
  are skipped, as the bash allows.
- [ ] Given checks that change the working tree, or a linked worktree whose git variables the
  shim clears, when they pass, then the tree recorded is the one decided before they ran.
- [ ] Given malformed standard input, when `nuc hook pre-push` reads it, then it refuses (Q-1).
- [ ] Given a canary token in the environment and the git configuration, when the push scenarios
  run, then it appears in no output and not in the record (Q-10).

## Read first

- Touches: the hook module under `src/`, the reference shims, the scenario harness,
  `tests/isolation.rs`.
- The template's `.githooks/pre-push` at v0.6.1 and v0.7.0 and its push cases in
  `scripts/test-git-hooks.sh`.
- ADR-0021 (the hook contract), ADR-0023 (behaviour by the carried release), Q-8 to Q-10.
- Golden path: the hooks of tickets 02 and 03.

## Blocked by

- docs/plan/hooks-port/03-pre-merge-commit-and-fail-closed.md

## Mode

with the human — the green-tree record skips the project's checks, a gate a wrong result opens
silently; the human reviews the skip and record path before it lands.
