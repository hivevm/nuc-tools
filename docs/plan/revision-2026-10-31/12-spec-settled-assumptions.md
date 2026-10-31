# Ticket: Draft the assumptions the record has settled out of the specification

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

A drafted specification change, for the human to decide and land
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7). Two *Assumptions* still state as open
what accepted ADRs and this spec's work settled:

- "`nuc` learns which template release a project carries either from the `Template release`
  line of its README or from its `nuc.toml`; the founding decision on version coupling settles
  which." ADR-0013 settled it: the README.
- "which derived repositories form it is decided when the harness is planned." ADR-0020 and its
  proposed successor leave the members to pull requests that move `tests/parity/corpus.toml`,
  each on template release v0.5.0 or later.

Drafted text:

> - The repositories of the parity corpus stay readable to the harness.
> - `nuc` reads the template release a project carries from the `Template release` line of its
>   README, and that line keeps the form the template's check 14 holds.

Accepted ADRs it may move, for the human to judge: ADR-0013, whose Context cites the open
assumption, and ADR-0020. Neither decision changes; their Context would cite a settled assumption.

## Acceptance criteria

- [ ] Given the draft, when the human has decided it, then it lands as a specification change by
  the human, naming each accepted ADR it moves, or is recorded as declined.

## Read first

- `docs/SPECIFICATION.md`, *Assumptions*; ADR-0013, ADR-0020.

## Blocked by

- None (can start immediately)

## Mode

with the human — the specification is the human's ([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules),
rule 7).
