# Ticket: Have `work-tickets` name what a wave of ticket pull requests costs, upstream

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Seven tickets of the last revision were opened as pull requests into `main` one by one, #57 to
#63. The ruleset on `main` asks a branch to be up to date before it merges (*Repository settings*
in `CONTRIBUTING.md`), so every merge put the others behind: each had to take up `main` and run
all the required checks again, the four platforms included, before the next could merge, and two
were red meanwhile on a flaky test that had nothing to do with them. After three had merged, the
other four were closed and carried by one `feat/` branch instead (#65).
[`AGENTS.md` §6](../../../AGENTS.md#6-project-rules) already lets branches integrate on a
`feat/` branch on the human's instruction, and `work-tickets` step 3.2 names both ways, but
nothing says which costs what, so the choice was made for each ticket without it.

`work-tickets` is the template's, so the wording changes upstream, and this repository takes it
up with a template release. Draft for the template one sentence in step 3.2 of
`work-tickets`, a how and no rule (ADR-0003): under a ruleset that asks a branch to be up to date,
a wave of tickets merged one pull request at a time runs the required checks once per merge for
each pull request still open, and a `feat/` branch that carries the wave runs them once for it.
The choice stays the human's.

## Acceptance criteria

- [ ] Given the draft, when the human has decided it, then it is raised with the template, or
  recorded as declined with the reason.
- [ ] Given a template release that carries it, when this repository takes it up, then step 3.2
  of `work-tickets` reads as the draft says.

## Read first

- Touches: `.agents/skills/work-tickets/SKILL.md`
- [`AGENTS.md` §6](../../../AGENTS.md#6-project-rules), *Branches may integrate first on a
  `feat/` branch*; `CONTRIBUTING.md`, *Integration branches* and *Repository settings*.
- The descriptions of #65 and of the pull requests it superseded, #60 to #63.

## Blocked by

- None (can start immediately)

## Mode

with the human — `work-tickets` is inherited from the template, and only its maintainer changes
it there.
