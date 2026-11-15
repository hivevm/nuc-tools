# Ticket: A macOS job runs no script behind the bash floor

<!-- A ticket of the design revision of 2026-11-14, in the shape of the plan-feature skill's
     ticket template, without a spec. Claim it by creating the branch
     ticket/revision-2026-11-14/<NN>-<slug> BEFORE any work, by the steps of the work-tickets
     skill. The branch that lands it deletes this file. Its sections are the parts a ticket
     carries, as docs/adr/NUC-large-work-is-planned.md decides (clause 3). -->

## What to build

A workflow job that runs on macOS runs no script that stops below bash 4.4. The take-up of
v0.9.1 put the bash floor into `scripts/lib/selftest.sh`; the self-test of the release tooling,
which `release.yml` and `release-test.yml` ran on `macos-latest`, then stopped on the runner's
bash 3.2. CI found it only after the pull request opened, and c8dccce skipped the step on macOS.
The same holds for `check-build.sh` in the portability job, which a9ab6c7 kept off the floor by
hand. A check of the project fails on a step of a job whose `runs-on` or matrix names a macOS
runner and that runs `bash scripts/<name>.sh` where that script sources `lib/bash.sh` or
`lib/selftest.sh`, unless the step is skipped on macOS by its `if:`.

## Acceptance criteria

- [ ] Given the workflows as they stand, when the check runs, then it passes.
- [ ] Given a step `run: bash scripts/test-release.sh` in a job on `macos-latest` without an `if:`
      that skips macOS, when the check runs, then it fails and names the workflow, the job, and
      the script.
- [ ] Given `scripts/check-build.sh` sourcing `lib/bash.sh`, when the check runs, then it fails
      on the portability job.

## Read first

- Touches: `scripts/check-workflows.sh` and its self-test, or a new check with its self-test,
  `scripts/check-all.sh`, `.github/workflows/checks.yml`
- c8dccce and a9ab6c7 for the two cases; the rule `shell/bash-floor` of `scripts/check-shell.sh`
  for how a script is known to source the floor.
- The convention "A check's cases are a table" of `docs/CONVENTIONS.md`, which the golden path
  predates: the new check writes its cases into a table under `scripts/fixtures/`.

## Blocked by

- None (can start immediately)

## Mode

with the human — DEV-one-agent-one-ticket allows no unattended ticket.
