# ADR-0028: The build, test, and lint run on Linux arm64, macOS, and Windows as a job of the Checks workflow whose check runs are required on `main`

- **Status:** 🟡 proposed
- **Date:** 2026-11-05
- **Deciders:** Markus Brigl
- **Applies to:** the `portability` job of `.github/workflows/checks.yml`, the required checks of `CONTRIBUTING.md`, and the ruleset on `main`

## Context

Q-4 asks that the full test suite pass in CI on Linux x64, Linux arm64, macOS, and Windows x64.
The `build` job of the Checks workflow runs `scripts/check-build.sh` on Linux x64 and is a required
status check. The other three platforms ran the same script in a workflow of their own,
`portability.yml`, which no ruleset required. ADR-0008 left the platform matrix of Q-4 and which of
its jobs are required out of scope, and named it as a follow-up.

That gap let a failure through. #9 merged into `main` while its macOS job failed: a test wrote a
file name that is no UTF-8, which APFS refuses, and `main` stayed red on macOS until #10. In #29
the Windows job caught a path compared as text, `src\rules\` against `src/rules/` (bdaf85a), only
because the human waited for it. [`AGENTS.md` §5](../../AGENTS.md#5-quality-bar--definition-of-done)
forbids merging red locally and in CI; nothing enforced it for three of the four platforms.

The reasons `portability.yml` gave for staying optional no longer hold. A job with a matrix reports
one check run per entry, which the required checks of `CONTRIBUTING.md` already name that way
(check 9 of `scripts/check-docs.sh`, from template release v0.6.0). The runners are not the
bottleneck: the three jobs take under a minute each, while the required `parity` job takes ten to
fifteen. The repository is public, and GitHub's billing states that standard GitHub-hosted runners
are free there.

## Decision

We will run `scripts/check-build.sh` on Linux arm64, macOS, and Windows as the `portability` job of
the Checks workflow, one entry of its matrix per platform, and require each of its check runs on
`main`.

- **The job:** `portability` in `.github/workflows/checks.yml`, its matrix `os` the runner labels
  `ubuntu-24.04-arm`, `macos-latest`, and `windows-latest`, `fail-fast` off, so that one red
  platform does not hide another. It replaces `portability.yml`, which is deleted. Linux x64 stays
  the `build` job.
- **The required checks:** `CONTRIBUTING.md` names `portability (ubuntu-24.04-arm)`,
  `portability (macos-latest)`, and `portability (windows-latest)` between its `required-checks`
  markers, and the ruleset on `main` requires them, which `scripts/sensor-repo-settings.sh`
  compares.
- **The local gate:** `scripts/check-all.sh` already runs `scripts/check-build.sh` on the
  machine it runs on; the job adds no line there.

**Out of scope:** which platforms Q-4 names, a decision of the specification; the benchmark
workflow and its runner; a platform beyond the runner images GitHub hosts.

## Alternatives considered

- **Not required, recorded** — the jobs stay in `portability.yml`, and an ADR states that they do
  not gate the merge. A red platform keeps slipping through whenever nobody waits for it, which is
  how #9 merged; Q-4 would hold only by habit.
- **Not required, reported** — the review binding sensor or the pull request template lists each
  platform's result for the human who merges. It reminds rather than decides, and a reminder is
  read or skipped like the result it repeats.
- **Required, the workflow kept apart** — `portability.yml` stays, and its three check runs are
  required. Check 9 compares the required checks with the jobs of `checks.yml` alone, so the list
  would have to stand outside its markers, unchecked, or the template's check would have to change.

## Sources / Prior art

- GitHub, *Troubleshooting required status checks*: a required check counts only from an eligible
  trigger such as `pull_request` or `push`, and a workflow skipped by a path filter leaves its
  check pending, so a required job runs on every pull request.
  https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/troubleshooting-required-status-checks
- GitHub, *About billing for GitHub Actions*: "The use of standard GitHub-hosted runners is free
  … in public repositories". The page does not name each platform in that sentence.
  https://docs.github.com/en/billing/managing-billing-for-your-products/managing-billing-for-github-actions/about-billing-for-github-actions
- The check runs of this repository: a matrix job reports one check run per entry, named
  `<job> (<values>)`, as `build (macos-latest)` on the pull requests #35 to #41, where the three
  jobs took under a minute each and `parity` ten to fifteen.

## Consequences

- Positive: Q-4 gates every merge into `main`; a platform failure is caught before the merge,
  whoever waits or does not. The settings sensor compares the required checks with the ruleset.
- Negative / trade-offs: an outage of one runner image blocks every merge until it recovers, or
  until a maintainer bypasses the ruleset. A pull request waits for three more runners, though not
  longer than for `parity`.
- Negative: no check compares the names of the check runs with the values of the matrix. A label
  renamed in the matrix but not in `CONTRIBUTING.md` leaves the ruleset waiting for a check run
  that never reports, and an entry dropped from the list runs unrequired.
- Follow-ups: whether a runner image that moves under its label, as `macos-latest` does, should be
  pinned to a version; whether check 9 should read the values of a matrix, a change of the
  template's check.

## Enforcement

Check 9 of `scripts/check-docs.sh` fails when a job of `checks.yml` is missing from the required
checks of `CONTRIBUTING.md`, and when the list names the `portability` job, which has a matrix, by
its bare name instead of its check runs, `<job> (<values>)`. `scripts/sensor-repo-settings.sh`,
run on a maintainer's instruction, reports a ruleset on `main` that differs from the list.

**Not mechanically decidable:** that the values and the number of the check runs the list names
match the matrix of the job, which check 9 leaves to the workflow; and that the ruleset on GitHub
requires them, which the settings sensor reports only when a maintainer runs it, since no test of
this repository can reach the ruleset.
