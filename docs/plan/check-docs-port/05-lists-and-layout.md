# Ticket: Port the check-list, Project Layout, and skill pointer checks

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`nuc check` reports what checks 9, 10, and 11 of `check-docs.sh` report: the jobs of `checks.yml`,
the required checks of the README, and the scripts of the workflows and `check-all.sh` agree; every
path of the Project Layout exists and every tracked top-level part is named; and every skill has
its pointer, a symlink that resolves to it.

## Acceptance criteria

- [ ] Given each case of `test-check-docs.sh` for checks 9, 10, and 11, when the rule runs, then
  it reports the same finding or none.
- [ ] Given a skill pointer that is a copy instead of a symlink, and one that does not resolve,
  when `nuc check` runs on a fixture repository, then it reports each as the bash does, on Linux,
  macOS, and Windows.
- [ ] Given the corpus and a variant per check (a job not named in the README, a layout path that
  does not exist, a skill without its pointer), when the harness runs, then it reports no
  difference.

## Read first

- ADR-0009 (the shape), ADR-0010 (git, where the rule reads it), ADR-0013, ADR-0019.
- The golden path and the test pattern in `docs/ARCHITECTURE.md`; the conventions on the shell
  reading for the rules and on porting the bash with its quirks.
- The checks this ticket ports in the template's `scripts/check-docs.sh` and their cases in
  `scripts/test-check-docs.sh`, at each release of the corpus.
- ADR-0003 (the skills and their pointers).

## Blocked by

- docs/plan/check-docs-port/01-corpus-breadth.md

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
