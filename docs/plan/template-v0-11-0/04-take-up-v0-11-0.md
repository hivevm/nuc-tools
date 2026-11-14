# Ticket: Take up v0.11.0

## What to build

The repository works under v0.11.0. The `NUC-` files, `AGENTS.md`, and `docs/adr/NUC.blobs` are
replaced by the release's. A `feat/` branch integrates linearly. The hooks `pre-commit`,
`pre-merge-commit`, and `pre-push` refuse a merge on a `feat/` branch. `scripts/ready-for-main.sh`
takes up `origin/main` by a rebase. `scripts/sensor-review-binding.sh` reads the hand-off with
`--hand-off <file>`. The skills, the pull request template, the review-binding workflow, and the
named sections of `CONTRIBUTING.md` follow. "Measuring a session" is taken up as the template has
it. The `Template release` line moves to v0.11.0.

## Acceptance criteria

- [ ] Given the take-up, when `scripts/check-all.sh` runs, then it is green.
- [ ] Given a `feat/` branch, when a merge is attempted on it, then the hooks refuse it.
- [ ] Given a branch carrying a merge commit `origin/main` lacks, when `scripts/ready-for-main.sh`
  runs, then it refuses and names each.
- [ ] Given a caller of the review-binding sensor in this repository's workflows or scripts, when
  it relied on merge messages, then it passes `--hand-off <file>`.
- [ ] Given the `feat/` branches in flight at the take-up, when the hand-off is read, then it
  names each one carrying merges, to be rebuilt linearly before the hooks meet it.

## Read first

- Touches: `docs/adr/NUC-*`, `docs/adr/NUC.blobs`, `AGENTS.md`, `README.md` (Template),
  `.githooks/`, `scripts/ready-for-main.sh`, `scripts/sensor-review-binding.sh`,
  `scripts/sensor-release-notes.sh`, `.agents/skills/` (review, work-tickets, design-revision),
  `.github/PULL_REQUEST_TEMPLATE.md`, `.github/workflows/review-binding.yml`, `CONTRIBUTING.md`
- The release notes of v0.11.0, "Taking it up".
- `.devcontainer/` of this repository, only to keep its own observability stack apart from the
  template's section (spec, "Out of scope").

## Blocked by

- docs/plan/template-v0-11-0/03-take-up-v0-10-0.md

## Mode

with the human: the hooks and the integration method are kept with the human by the harness ADR,
and the repository settings change.
