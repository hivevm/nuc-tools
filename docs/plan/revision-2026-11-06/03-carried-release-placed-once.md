# Ticket: Place the release a tree carries in one reading

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Where the README's `Template release` line places a tree is read twice in the shell.
`shell::carried_release` places it for the rules, refusing a release before the earliest and a
README that names none. `shell::hook::contributing` places it for the draft notice of
`pre-push`, deciding a missing or unnamed release as the newest and one before v0.5.0 as v0.5.0,
as ADR-0023's *Hooks* clause has it, and compares it with `rules::CONTRIBUTING_GUIDE` itself,
where the rules ask `Inputs::carries_at_least` and `Inputs::guide`. The two readings differ by
design in what they refuse, and agree in everything else; a release at which the guide moves again
would have to be taught to both.

Read the line once, into one value the shell keeps, and let the hook's leniency be one step on that
value rather than a second reading. The choice of the guide's name, README or `CONTRIBUTING.md`,
comes from one function for the rules and the notice. No finding, notice, or exit code changes.

## Acceptance criteria

- [ ] Given trees on v0.6.1, v0.7.0, `unreleased`, before v0.5.0, and without the line, when
  `nuc check` and `nuc hook pre-push` run on each, then they answer as today, the parity harness
  and its hook scenarios included.
- [ ] Given the source, when the comparison with `CONTRIBUTING_GUIDE` is searched for, then it
  stands in one function.

## Read first

- Touches: `src/shell/mod.rs`, `src/shell/hook.rs`, `src/rules/mod.rs`,
  `src/rules/docs_template_release.rs`
- ADR-0023, *Not decided* and *Hooks*.

## Blocked by

- None (can start immediately)

## Mode

with the human — `src/shell/` and the reading of the README are protected parts of ADR-0007.
