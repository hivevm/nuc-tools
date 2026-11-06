# Ticket: Dispatch every hook through one table in shell::hook

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The design revision of 2026-11-06 found the dispatch of `nuc hook <name>` in two shapes.
`shell::hook::run` tests `pre-push`, `reference-transaction`, and `merge-adr-index` by name in
an `if` chain, and finds `pre-commit` and `pre-merge-commit` in the table `HOOKS`. The review of
`pre-merge-commit` (#51) asked for that table so that a later hook could not fall through to
another's decision; the three hooks after it went back to the chain ahead of it. The steps every
hook takes are written once per hook as well: the current directory read with the same message
four times, `trim_end_matches(['\n', '\r'])` after each git answer, and the closure that turns
the list of `turned_on` into a predicate three times.

Make one table of every hook `nuc hook` knows, each name with its function, so that the error for
an unknown hook and the dispatch read the same list, and take the repeated steps into one helper
each. Interface-preserving: no exit code, message, or answer changes.

## Acceptance criteria

- [ ] Given `src/shell/hook.rs`, when `run` dispatches, then every hook it knows is found in one
  table, and no hook name is compared outside it.
- [ ] Given a hook name the table lacks, when `nuc hook <name>` runs, then it exits 2 with the
  message it gives today, which `tests/contract.rs` holds.
- [ ] Given the suite, `tests/hooks.rs` and `bash scripts/check-parity.sh --hooks` included,
  when it runs, then it passes unchanged.

## Read first

- Touches: `src/shell/hook.rs`
- ADR-0021, the *Decision*; the draft ADR-draft-rules-return-an-outcome-and-hooks-an-answer, its
  *Hooks* and *Shell* clauses, where it is accepted by then.
- The *Review* in the description of #51, which asked for the table.

## Blocked by

- None (can start immediately)

## Mode

with the human — `src/shell/` is a protected part of ADR-0007, and no ticket's *Mode* allows
unattended work there.
