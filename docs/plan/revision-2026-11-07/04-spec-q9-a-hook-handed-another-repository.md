# Ticket: Say in Q-9 what a hook does when handed another repository

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

This is a drafted specification change, for the human to decide and land
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7).

Q-9 asks that, with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and `GIT_CONFIG_PARAMETERS` set
to a decoy repository, "every command reports on the repository it was pointed at and leaves the
decoy untouched". Three tests cite Q-9; since #59 one of them,
`nuc_hook_leaves_a_decoy_in_the_environment_untouched_for_any_shim` in `tests/hooks.rs`, runs
every shim's `nuc hook` that way. A hook that decides on the git state does not report on any
repository there: handed a git directory other than the one it finds from the directory git runs
it in, it ends the run not decided, exit 2, because `nuc` takes the repository never from the
environment (ADR-0011) and a hook that cannot tell never passes (Q-1). Two hooks need no git
state for what they do and act as before: `reference-transaction` passes a transaction that
touches no `main` before it checks the git directory, since its pass depends on no repository
(Q-5); and the merge driver checks no handed git directory and merges the files git names it in
the repository git runs it in, exit 0 in the test. The test decides the right thing; the
criterion reads as if every hook reported on its repository. The test calls `nuc hook` directly,
not through git, since git itself follows a decoy `GIT_DIR` (#59, *Doubts*), and Q-9 names no way
of calling.

Drafted text, replacing Q-9:

> - **Q-9** — One repository: with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and
>   `GIT_CONFIG_PARAMETERS` set to a decoy repository, every command leaves the decoy untouched
>   and reports on the repository it was pointed at; a hook that would decide on the git state
>   does not decide where it is handed a git directory other than the one it finds, and the
>   merge driver merges the files git names it in the repository git runs it in.

The alternatives for the human: keep Q-9 as it reads and let "reports" cover a refusal not
decided, which leaves a reader to infer the hooks' exit 2 from Q-1; or have the hooks follow the
handed git directory, which ADR-0011 rules out.

Accepted ADRs whose basis it may move, for the human to judge: ADR-0011, whose *Context* cites
Q-9 and whose *Enforcement* repeats its wording, "reports on the repository it was pointed at";
ADR-0021, whose hooks decide from the git state. Neither is contradicted by the draft as far as
this revision read them. That is no judgement that none is affected. The section *Threats &
Forbidden Actions* of `docs/SPECIFICATION.md` states the security goal as "never act on a
repository other than the one it was pointed at (Q-9)"; the draft keeps that sentence true, and
the human's change keeps the two consistent.

## Acceptance criteria

- [ ] Given the draft, when the human has decided it, then it lands as a specification change by
  the human, naming each accepted ADR it moves, or it is recorded as declined.
- [ ] Given the change landed, when `tests/hooks.rs`
  `nuc_hook_leaves_a_decoy_in_the_environment_untouched_for_any_shim` is read, then each branch of
  it decides a part of Q-9 as it reads, and a transaction that touches no `main` passes as the
  draft says.

## Read first

- Touches: `docs/SPECIFICATION.md`
- `docs/SPECIFICATION.md`, *Quality Goals*; ADR-0011, *Context* and *Enforcement*; ADR-0021.
- The *Doubts* in the description of #59.

## Blocked by

- None (can start immediately)

## Mode

with the human — the specification is the human's
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7).
