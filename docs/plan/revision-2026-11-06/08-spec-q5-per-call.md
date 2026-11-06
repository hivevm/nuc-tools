# Ticket: Draft what Q-5 measures per call

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

This is a drafted specification change, for the human to decide and land
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7).

Q-5 reads: "`nuc hook reference-transaction` adds at most 2 ms median per call over a no-op hook
on Linux x64, measured over 1,000 calls." The hook runs on every update of a ref, and what it costs
depends on the update. The benchmark of #53, `tests/bench/hook-latency.sh`, gates the update
almost every git command makes, of a branch other than `main`, which asks git nothing: +645 µs in
the Dev Container. It reports, and does not gate, a write of `main` where it is (+1,670 µs) and a
move of `main` to `origin/main`, as `git merge --ff-only origin/main` makes it (+3,039 µs), which
needs two git processes. The implementer read the median as one over the calls a repository
makes, and left the reading to the human; the review of #53 called it a decision that narrows
Q-5. The criterion does not say which calls its 1,000 are, so the gate and the text can part.

Drafted text, replacing Q-5:

> - **Q-5** — Hook latency: `nuc hook reference-transaction` adds at most 2 ms median per call
>   over a no-op hook on Linux x64, measured over 1,000 updates of a branch other than `main`; an
>   update of `main` is measured and reported beside it.

The alternatives for the human:
- keep Q-5 as it reads, and have every kind of update meet 2 ms: the move of `main` then needs
  another design, one git process where it now spawns two, or the hook's answer without
  `merge-base`;
- a bound per kind, 2 ms for a branch and a looser one for `main`, each gated;
- the draft above, which says what the benchmark gates today.

Accepted ADRs whose basis it may move, for the human to judge: ADR-0008, its *Benchmark* clause,
which places the measure; ADR-0021, which has the hook ask git for `origin/main` as the
transaction leaves it. Neither is contradicted by the draft as far as this revision read them. That
is no judgement that none is affected.

## Acceptance criteria

- [ ] Given the draft, when the human has decided it, then it lands as a specification change by
  the human, naming each accepted ADR it moves, or it is recorded as declined.
- [ ] Given the change landed, when `tests/bench/hook-latency.sh` is read, then what it gates is
  what Q-5 says, and what it only reports is what Q-5 says it reports.

## Read first

- Touches: `docs/SPECIFICATION.md`, `tests/bench/hook-latency.sh`
- `docs/SPECIFICATION.md`, *Quality Goals*; ADR-0008 (*Benchmark*); ADR-0021.
- The *Doubts* and *Review* in the description of #53.

## Blocked by

- None (can start immediately)

## Mode

with the human — the specification is the human's
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7).
