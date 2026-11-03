# Ticket: Decide whether the platforms of Q-4 gate the merge into main

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

#9 merged into `main` while its `build (macos-latest)` job of the Portability workflow failed.
A test created a file whose name is no UTF-8, which APFS refuses, so `main` was red on macOS
until #10.
[`AGENTS.md` §5](../../../AGENTS.md#5-quality-bar--definition-of-done) forbids handing off or
merging red "locally and in CI". Nothing stopped the merge, because `portability.yml` is no
required check. Its header gives the reason: one check name per platform, and slower runners
that would gate every push. That reason is recorded only in the workflow's comment. In #29 the
Windows job caught a path separator (bdaf85a) before the merge, because the human waited for it.

A check could have caught the macOS failure: a required status check. Whether the three
Portability jobs become required is a decision on what gates the merge. It belongs to the human
and is recorded as an ADR: ADR-0008 names the required jobs of `checks.yml` and is silent on
Portability. Q-4 asks that the suite pass on all four platforms.

Options to weigh with the human:

- **Required:** the three `build (<os>)` check runs join the ruleset, and the README's required
  checks name them. Check 9 of `scripts/check-docs.sh` compares that list with the jobs of
  `checks.yml` only, so the jobs may have to move into `checks.yml`, or the README's wording
  change.
- **Not required, recorded:** a proposed ADR states that Portability does not gate the merge, and
  why.
- **Not required, reported:** the review binding sensor or the pull request template reports
  each platform's result to the human who merges.

## Acceptance criteria

- [ ] Given the human's choice, when this ticket closes, then a proposed ADR records it, and the
  workflow, the README's Repository settings, and `scripts/sensor-repo-settings.sh` agree with it.
- [ ] Given the required form was chosen, when a pull request's macOS job fails, then the ruleset
  refuses the merge, shown on a throwaway pull request or by the settings sensor.

## Read first

- ADR-0008 and ADR-0023.
- Q-4 in docs/SPECIFICATION.md.
- `.github/workflows/portability.yml`, `.github/workflows/checks.yml`, and the README's
  Repository settings.
- The description of #10.

## Blocked by

- None (can start immediately)

## Mode

with the human — the ruleset and what gates the merge are the human's, through a proposed ADR
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules)).
