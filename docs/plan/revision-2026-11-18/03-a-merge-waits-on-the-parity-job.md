# Ticket: A merge into main waits on the parity job, once per change on main

## What to build

The ruleset of `main` requires a branch to be up to date before it merges, as
`scripts/sensor-repo-settings.sh` asks of every project. On CI the job `parity` takes 20 to 26
minutes, of which about 20 compare the bash checks and `nuc` on the corpus. Locally the whole of
`scripts/check-all.sh` takes 420 to 445 s. Every other job finishes well before `parity`. Each
change that lands on `main` puts the open pull requests behind, and each then waits one more
`parity` run.

The evidence comes from the day of this revision, just past its scope. #112 accepts an ADR. It
touches the headers of two ADRs, the index, and one self-test. Its `parity` was started three
times. The first two runs were cancelled when the branch was brought up to date after #110 merged,
and again after #111 merged. Each restart cost up to 20 minutes of a run that had found nothing. In
all pull requests of this revision's scope, `parity` found nothing that the local checks had not
found first.

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
- [ ] Given the change the human chose, when a pull request of the kind it targets runs CI, then
  the `parity` job takes at most half the time of the median of its last ten runs on `main`
  before the change, both read from the run's job timings.
- [ ] Given a change to `src/`, when its pull request runs CI, then `parity` still compares the
  whole corpus.

## Read first

- Touches: `.github/workflows/checks.yml`, `scripts/check-parity.sh`
- TEST-parity-harness; SEC-actions-pinned-by-sha (for any cache action); NUC-main-only-via-pr

## Blocked by

- None (can start immediately)

## Mode

with the human: it changes a required gate of `main`, and which inputs may skip it is the
human's to decide.
