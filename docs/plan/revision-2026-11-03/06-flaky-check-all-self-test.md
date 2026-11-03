# Ticket: Make the check-all self-test's TERM case decide by events, not by sleeps

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

#13 reported that the case "and no check it started finishes after it" of
`scripts/test-check-all.sh` failed once in a full run and then passed three times alone. It was
written off as "timing, not this change".
[`AGENTS.md` §5](../../../AGENTS.md#5-quality-bar--definition-of-done) says a flaky test is a
defect, not noise.
The case waits a fixed `sleep 1.2` after the runner returns, against stubs that `sleep 1` and
then touch `finished`. The runner is killed once the first `started` exists, after at most ten
polls of 0.1 s. Under load, a check that starts after TERM, or a stub that was not stopped,
can touch `finished` inside that window.

Find the cause, and make the case decide by what happened rather than by the wall clock. The
script is the template's (`scripts/test-check-all.sh` and `scripts/check-all.sh` come with
template releases), so the fix goes to hivevm/nuc and returns here by a release. Do not patch it
in place unless the human chooses to.

## Acceptance criteria

- [ ] Given a loaded machine, as a parallel `cargo build` alongside, when the self-test runs 50
  times, then the case never fails, or the failure is shown to be a defect of `check-all.sh`,
  which a test then decides.
- [ ] Given the human's choice, when this ticket closes, then the fix is proposed to the template,
  or landed here with the reason it could not wait for a release.

## Read first

- ADR-0024, on what a derived project takes from a template release.
- `scripts/check-all.sh` and `scripts/test-check-all.sh`.
- The Doubts of #13.

## Blocked by

- None (can start immediately)

## Mode

with the human — the script is the template's, and where its fix lands is the human's choice.
