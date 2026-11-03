# Ticket: Draft a measure of Q-1 that its tests decide

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

This is a drafted specification change, for the human to decide and land
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 7).

Q-1 ends with "no input makes `nuc` exit 0 with a finding dropped". #15 cited Q-1 from the class
fixtures its measure names. The review raised a doubt, which the description recorded without
deciding it: that clause is universal, and no finite test decides it. The markers rest on the
fixtures and on the parity harness over the corpus and its variants. "If the human reads that
clause as part of the measure, Q-1 should carry no marker until a test decides it." The criterion
does not say what its tests decide, so the traceability check counts as cited a criterion whose
last clause nothing decides.

Drafted text, replacing the measure of Q-1:

> - **Q-1** — Integrity of the gates: a check or hook never passes when it could not decide. For
>   each class of failure — git missing or failing, an unreadable or malformed input, an unknown
>   rule, an internal error or panic — a fixture shows a non-zero exit and a message naming the
>   cause; on every tree of the parity corpus and every variant, `nuc` exits 0 only where the bash
>   reports no finding of a ported rule.

The drafted measure leans on Q-2's parity harness: it is decided only as far as the corpus and
its variants reach, and it moves with them. The alternatives for the human: keep the clause as
the property and say that the measure is the fixtures and the harness; or keep it as it reads and
take the markers off Q-1 until a test decides it.

Accepted ADRs whose basis it may move, for the human to judge: ADR-0019, on the exit codes;
ADR-0022, the harness, which compares the exit status; ADR-0025, on a rule's refusal. None is
contradicted as far as this revision read them. That is no judgement that none is affected.

## Acceptance criteria

- [ ] Given the draft, when the human has decided it, then it lands as a specification change by
  the human, naming each accepted ADR it moves, or it is recorded as declined.
- [ ] Given the change landed, when the tests that cite Q-1 are read, then each decides a part of
  the measure as it reads.

## Read first

- `docs/SPECIFICATION.md`, *Quality Goals*.
- The Doubts of #15.
- ADR-0019, ADR-0022, ADR-0025.

## Blocked by

- None (can start immediately)

## Mode

with the human — the specification is the human's ([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules),
rule 7).
