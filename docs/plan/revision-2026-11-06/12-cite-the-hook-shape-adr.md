# Ticket: Implement the draft on the shape of the hooks

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The revision proposed ADR-draft-rules-return-an-outcome-and-hooks-an-answer, which restates
ADR-0010 with a *Hooks* clause: the hook rules listed in `src/hook.rs`, decided one function per
hook, which returns an answer. Once the human has reviewed the draft, implement it while it is
proposed, after its commit on the same branch
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 4): the code already has the shape, so
the work is the citations. Every test, list, and document that cites ADR-0010 for the shape cites
the draft, the structural test's `Verifies:` marker among them, and the overview names it in its
*Rules* and *Hooks* blocks.

## Acceptance criteria

- [ ] Given the branch, when `grep -rn ADR-0010` runs outside `docs/adr/`, then no citation of the
  shape is left, and `tests/adr_citations.rs` passes once the human accepts the draft.
- [ ] Given the structural test, when it runs, then it cites the draft and holds `src/hook.rs` as
  part of the library outside `src/shell/`.

## Read first

- Touches: `tests/functional_core.rs`, `tests/functional_core.outcomes`,
  `tests/functional_core.forbidden`, `tests/check.rs`, `src/lib.rs`, `src/shell/mod.rs`,
  `src/rules/mod.rs`, `Cargo.toml`, `docs/ARCHITECTURE.md`, `docs/CONVENTIONS.md`
- `docs/adr/draft-rules-return-an-outcome-and-hooks-an-answer.md`, ADR-0010.

## Blocked by

- None (no ticket blocks it; it starts once the human has reviewed the draft, rule 4)

## Mode

with the human — the draft is a decision of the human, and the structural test a protected part of
ADR-0007.
