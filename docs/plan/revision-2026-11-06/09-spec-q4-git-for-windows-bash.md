# Ticket: Draft what Q-4 asks of a hook on Windows

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

This is a drafted specification change, for the human to decide and land
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7).

Q-4 asks that the full suite pass on four platforms "with no bash on the `PATH` of the process
under test". Git for Windows hands every hook the bash it bundles, whatever the `PATH` holds, so a
shim run through git there always finds bash, and the check a shim runs on 3 runs. #51 found it in
the first Windows check run and changed its test (24c719b): on Windows it asserts that the check
runs, and shows that a check which cannot run is refused by a missing script instead; on Unix it
asserts the refusal with bash missing. The criterion reads as if a hook on Windows ran without
bash, which git does not allow, and the test now decides something narrower there than the text.

Drafted text, replacing Q-4:

> - **Q-4** — Portability: the full test suite passes in CI on Linux x64, Linux arm64, macOS, and
>   Windows x64, with no bash on the `PATH` of the process under test; a hook git for Windows
>   runs gets the bash git bundles, and only the checks a shim runs on 3 may use it.

The alternatives for the human: keep Q-4 as it reads and say in the test why Windows differs,
which leaves the criterion claiming what no test there decides; or name the bundled bash as part
of the platform in the glossary rather than in Q-4.

Accepted ADRs whose basis it may move, for the human to judge: ADR-0021, whose context cites
Q-4's "no bash on the `PATH`" for the shims in POSIX sh; ADR-0008, which runs the suite on the four
platforms. Neither is contradicted by the draft as far as this revision read them. That is no
judgement that none is affected.

## Acceptance criteria

- [ ] Given the draft, when the human has decided it, then it lands as a specification change by
  the human, naming each accepted ADR it moves, or it is recorded as declined.
- [ ] Given the change landed, when the no-bash test of `tests/hooks.rs` is read, then each
  branch of it decides a part of Q-4 as it reads.

## Read first

- Touches: `docs/SPECIFICATION.md`, `tests/hooks.rs`
- `docs/SPECIFICATION.md`, *Quality Goals*; ADR-0021 (*Context*); ADR-0008.
- The *Doubts* in the description of #51.

## Blocked by

- None (can start immediately)

## Mode

with the human — the specification is the human's
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7).
