# Ticket: Check a tree on template release v0.7.0 as its check-docs.sh does

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Template release v0.7.0 moves the required checks and the Project Layout from the README to
`CONTRIBUTING.md`. Its `check-docs.sh` reads them there in checks 9 and 10, and names
`CONTRIBUTING.md` in their findings. Check 14 reports a missing README itself, where earlier
releases left that to check 10. `nuc` knows no release later than v0.6.0 for `docs/check-lists`
and none at all for `docs/project-layout`, so it checks a tree on v0.7.0 as v0.6.x does. This
repository carries v0.7.0: `nuc` on its own tree reports "no job names between the
'required-checks' markers" and "no paths found under '## Project Layout'", where the bash
passes.

Let `docs/check-lists` and `docs/project-layout` read `CONTRIBUTING.md` from v0.7.0 on, with the
v0.7.0 messages, as ADR-0021 has a rule check as the release a tree carries. Pin the template at
its release v0.7.0 in `tests/parity/corpus.toml`, and give the variants that edit the README's
lists a twin for `CONTRIBUTING.md` that names its trees (ADR-0022). Then move the README's claim
that every check is ported as the release a tree carries checks it back to "v0.5.0 or later".

## Acceptance criteria

- [ ] Given a tree on v0.7.0 whose required checks and Project Layout stand in
  `CONTRIBUTING.md`, when `nuc check` runs `docs/check-lists` and `docs/project-layout`, then it
  reports what check 9 and check 10 of v0.7.0 report, naming `CONTRIBUTING.md`.
- [ ] Given a tree on v0.6.2 or earlier, when the same rules run, then they read the README as
  before.
- [ ] Given the corpus with the template at v0.7.0 and the new variants, when
  `scripts/check-parity.sh` runs, then it passes.

## Read first

- Touches: `src/rules/docs_check_lists.rs`, `src/rules/docs_project_layout.rs`,
  `src/shell/tree.rs`, `src/rules/mod.rs`, `tests/parity/corpus.toml`,
  `tests/parity/variants.tsv`, `tests/parity/normalize.tsv`, `README.md`.
- ADR-0021 and ADR-0022.
- The convention *A port copies the bash, quirks included*.
- `git -C <template clone> diff v0.6.2 v0.7.0 -- scripts/check-docs.sh scripts/test-check-docs.sh`.
- The release-gated case of `docs/check-lists`, `NOT_A_GATE_AND_MATRIX`, as the golden path.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
