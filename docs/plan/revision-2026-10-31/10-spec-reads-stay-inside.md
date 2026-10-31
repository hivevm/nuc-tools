# Ticket: Draft a quality goal for reads that stay inside the repository

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

A drafted specification change, for the human to decide and land
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7). The specification forbids writes outside
the repository (Q-8) but says nothing of reads. The port refuses every read, listing, or link that
resolves outside the repository, after three reviews found it reading there; the tests in
`tests/check.rs` decide it, but no criterion they could cite says it. Tree content is untrusted
(*Threats*), and a symbolic link in a fork's pull request could point `nuc` at files of the machine
it runs on.

Drafted text, a new quality goal after Q-11 and a line among the forbidden actions:

> - **Q-12** — Reads stay inside: no command of `nuc` reads a file or lists a directory of the tree
>   that resolves outside the repository; a fixture whose link, symbolic link, or listed path
>   leads a read outside shows exit 2 and a message naming it. A test of existence, as the bash's
>   `-e`, follows the path and reads nothing. Git's own configuration, attribute, and
>   exclude files are git's to read.

> - The system shall never read through the tree outside the repository (Q-12).

Accepted ADRs whose basis it may move, for the human to judge: ADR-0009, whose Context lists the
threats the shell holds (Q-7 to Q-11); ADR-0010, which keeps git's system and global
configuration in effect, outside the repository; ADR-0011, whose fixtures would test it. None is
contradicted as far as this revision read them; that is not a judgement that none is affected.

## Acceptance criteria

- [ ] Given the draft, when the human has decided it, then it lands as a specification change by
  the human, naming each accepted ADR it moves, or is recorded as declined.
- [ ] Given the change landed, when the tests that refuse a path outside the repository are
  read, then they cite the new criterion.

## Read first

- `docs/SPECIFICATION.md`, *Quality Goals* and *Threats & Forbidden Actions*.
- ADR-0009, ADR-0010, ADR-0011.

## Blocked by

- None (can start immediately)

## Mode

with the human — the specification is the human's ([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules),
rule 7).
