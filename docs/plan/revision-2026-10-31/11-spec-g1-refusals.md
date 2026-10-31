# Ticket: Draft G-1 to say where nuc refuses instead of matching the bash

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

A drafted specification change, for the human to decide and land
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7). G-1 says `nuc check` reports the same
findings as the bash checks on the same tree. The tests decide more: on a tree the bash reads by its
locale or its buffers rather than by the bytes alone, `nuc` exits 2 where the bash passes or
reports, as for a NUL byte past grep's first buffer, a wide line holding a byte of no valid UTF-8 or
0x1F, an ADR holding 0x1F, and a draft name holding a newline. The port of
`check-traceability.sh` added four: a specification that exists but cannot be read, which the
bash passes over; a file holding a marker and a NUL byte past grep's first buffer; a path of no
UTF-8 whose file holds a marker; and a symbolic link in git's listing that leads outside the
repository, refused before its content is known, where the bash reads it. Q-1 outranks G-1, so
the behaviour is right, but G-1's text no longer says what the tests decide, and a reader
comparing the two would take the refusals for a defect.

Drafted text for G-1:

> 1. **G-1** — When a derived project runs `nuc check`, it reports the same findings as the bash
>    checks of the template release the project carries report on the same tree, or refuses a
>    tree the bash would read otherwise than by its bytes (Q-1).

Accepted ADRs it may move, for the human to judge: ADR-0019, whose exit 2 the refusals use, and
ADR-0020, whose parity compares exit status on the corpus. Neither appeared contradicted to this
revision; that is not a judgement that none is affected.

## Acceptance criteria

- [ ] Given the draft, when the human has decided it, then it lands as a specification change by
  the human, naming each accepted ADR it moves, or is recorded as declined.

## Read first

- `docs/SPECIFICATION.md`, G-1, Q-1, Q-2; ADR-0019, ADR-0020.
- The refusals of `src/shell/` whose messages say the bash reads a byte otherwise.

## Blocked by

- None (can start immediately)

## Mode

with the human — the specification is the human's ([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules),
rule 7).
