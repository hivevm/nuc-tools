# Ticket: A merge into main waits on the parity job, once per change on main

## What to build

The ruleset of `main` requires a branch to be up to date before it merges, as
`scripts/sensor-repo-settings.sh` asks of every project. On CI the job `parity` takes 20 to 26
minutes, of which about 20 compare the bash checks and `nuc` on the corpus. Locally the whole of
`scripts/check-all.sh` takes 420 to 445 s. Every other job finishes well before `parity`. Each
change that lands on `main` puts the open pull requests behind, and each then waits one more
`parity` run.

The evidence is the merges of 2026-11-18. #112 changes one ADR's header and a self-test, and
`parity` ran for it three times: the first run was outdated by #110, the second by #111. That made
about 75 minutes between a green local run and the merge. In all pull requests of this revision's
scope, `parity` found nothing that the local checks had not found first.

Find out what the `parity` job spends its 20 minutes on: fetching the corpus, building `nuc`, the
bash checks, or the comparisons. Then propose to the human the change that saves most without
weakening the gate. Options to weigh:

- cache the corpus between runs;
- split the corpus across jobs;
- skip the corpus comparison for a change that touches none of its inputs, as a step that passes
  rather than a skipped required check.

The decision of which inputs count is the human's.

## Acceptance criteria

- [ ] Given the timings of the steps of a `parity` job, when the session reports them, then the
  hand-off names where its time goes, measured, not estimated.
- [ ] Given the change the human chose, when a pull request that touches no input of the parity
  harness runs CI, then `parity` finishes in a time the hand-off states, and on a change to
  `src/` it still compares the whole corpus.

## Read first

- Touches: `.github/workflows/checks.yml`, `scripts/check-parity.sh`
- TEST-parity-harness; SEC-actions-pinned-by-sha (for any cache action); NUC-main-only-via-pr

## Blocked by

- None (can start immediately)

## Mode

with the human: it changes a required gate of `main`, and which inputs may skip it is the
human's to decide.
