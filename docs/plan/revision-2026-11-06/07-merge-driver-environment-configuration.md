# Ticket: Pin that the merge driver reads no configuration from the environment

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The hand-off of the merge driver (#54) left a doubt for the human: `git -c
merge.conflictStyle=diff3 merge` hands the setting to the driver in `GIT_CONFIG_PARAMETERS`; the
bash driver writes diff3 markers there, while `nuc`, whose git runs under the cleared environment
of ADR-0011, writes plain ones. `GIT_CONFIG_GLOBAL` and `XDG_CONFIG_HOME` are dropped alike.

The revision weighed it: no criterion or ADR moves. ADR-0011 decides the cleared environment, Q-9
asks that a decoy in `GIT_CONFIG_PARAMETERS` change nothing, and Q-2 and ADR-0022 compare the
merged index only in the scenarios, none of which sets configuration through the environment. G-3
asks for the index both sides imply or a conflict, which both write. The overview now says it. What
is missing is a test that decides it, so that a change to the allowlist of ADR-0011 cannot
silently start reading the setting, or a later port silently stop.

## Acceptance criteria

- [ ] Given two branches whose index rows conflict in prose, when they merge through the shim with
  `git -c merge.conflictStyle=diff3`, then the conflict markers have no base section, and the test
  says that ADR-0011 decides it.
- [ ] Given the same merge with `merge.conflictStyle` set in the repository's configuration, when
  it runs, then the markers follow it.

## Read first

- Touches: `tests/hooks.rs`
- ADR-0011, ADR-0021 (*Merge driver*); Q-9; *How it runs* in `docs/ARCHITECTURE.md`, the driver.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket.
