# Ticket: `reference-transaction`, and Q-5 gates its latency in CI

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`reference-transaction` through its shim and `nuc hook reference-transaction`: in the `prepared`
state, an update of the local `main` passes only to a commit that `origin/main` has as the same
transaction leaves it; a write of `main` where it already is, a deletion, and a repository without
`origin/main` pass; the refusal carries the bash's hint, the one for a `main` already ahead
included. A benchmark measures Q-5 and gates it in CI on Linux x64.

## Acceptance criteria

- [ ] Given the scenario harness, when it runs the hook self-test's `reference-transaction` cases,
  then they agree with the bash of each pinned release, fetch, reset, `git branch -f`, and pack-refs
  included.
- [ ] Given malformed standard input, when `nuc hook reference-transaction` reads it, then it
  refuses with a message naming it (Q-1).
- [ ] Given 1,000 calls of `nuc hook reference-transaction` and of a no-op hook on Linux x64, when
  the benchmark runs in CI, then it fails when the median difference exceeds 2 ms, and passes
  otherwise (Q-5).
- [ ] Given the pre-push hook, when it runs, then the benchmark is not part of it.

## Read first

- Touches: the hook module under `src/`, the reference shims, the scenario harness,
  `.github/workflows/checks.yml`, `CONTRIBUTING.md` (required checks, if the job is new).
- The template's `.githooks/reference-transaction` and its cases in `scripts/test-git-hooks.sh`.
- Q-5 in `docs/SPECIFICATION.md`; ADR-0008 (CI jobs), ADR-0018 (action references), ADR-0015.
- Golden path: the hooks of tickets 02 and 03.

## Blocked by

- docs/plan/hooks-port/03-pre-merge-commit-and-fail-closed.md

## Mode

with the human — the benchmark adds a gate to CI, and whether it is a new required check or a
step of an existing job is the human's call on the ruleset.
