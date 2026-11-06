# Ticket: Have the no-gate clause name a workflow that measures, upstream

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`benchmark.yml` runs the benchmark of Q-5 weekly, as ADR-0008's *Benchmark* clause decides, and
fails on a median above 2 ms. Check 9 of `scripts/check-docs.sh` would fail on its job, which runs
no script under `scripts/`, so `scripts/check-all.sh` names it on a `# Not a gate:` line. That
line, ADR-0002's clause behind it, and the header of `scripts/check-all.sh` all say it is for "a
workflow that publishes rather than checks, such as a release". The benchmark publishes nothing
and checks something no push can: the timing of a runner. The hand-off of #53 left it to the human
whether a clause should say so, and why the benchmark stands under `tests/bench/`: under
`scripts/`, check 9 would have the local gate run it, which ADR-0008 rules out for timing.

ADR-0002, `scripts/check-docs.sh`, and the header of `scripts/check-all.sh` are the template's, so
the wording changes upstream, and this repository takes it up with a template release. Draft the
change for the template: the no-gate clause covers a workflow whose jobs run what no push can gate,
a release that publishes or a benchmark that times a runner, its scripts still counted; and a
script that only such a workflow runs stands outside `scripts/`, since check 9 would put it into
the local gate. Until a release brings it, nothing changes here; the comment above the line in
`scripts/check-all.sh` already says why the benchmark is named.

## Acceptance criteria

- [ ] Given the draft, when the human has decided it, then it is raised with the template, or
  recorded as declined with the reason.
- [ ] Given a template release that carries it, when this repository takes it up, then the
  `# Not a gate:` line of `benchmark.yml` reads as the clause says.

## Read first

- Touches: `scripts/check-all.sh`
- ADR-0002, *The checks*; ADR-0008, *Benchmark*; check 9 in the header of
  `scripts/check-docs.sh`.
- The *Doubts* in the description of #53.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0002 is inherited from the template, and only its maintainer changes it
there.
