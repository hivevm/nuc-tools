# Ticket: Say once which rule IDs this nuc knows

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Which IDs `nuc.toml` may name is answered in four places since the hooks were ported:
`config::select` refuses an ID `rules::ALL` lacks; `shell::check` first drops every ID
`hook::find` knows from the list so that `select` does not refuse it; `shell::hook::turned_on`
refuses an ID neither `rules::find` nor `hook::find` knows, with a refusal it builds itself; and
`shell::explain` looks up both lists in turn. A rule ID of a third kind would have to be added in
each, and a place that misses it refuses a `nuc.toml` the others accept.

Answer it in one place, a function of `config` or of the library that knows both lists, which the
two commands and `nuc explain` call. No exit code or message changes.

## Acceptance criteria

- [ ] Given a `nuc.toml` that lists a `docs/` rule, a `hook/` rule, and an unknown ID, when
  `nuc check` and `nuc hook pre-commit` run, then both refuse with exit 2 and the same message,
  naming the unknown ID, as today.
- [ ] Given the source, when the unknown-rule test is searched for, then one function decides it
  and the shell holds no list of its own.
- [ ] Given `nuc explain` with a `docs/` and a `hook/` ID, when it runs, then it explains both,
  as `tests/contract.rs` holds.

## Read first

- Touches: `src/config.rs`, `src/shell/mod.rs`, `src/shell/hook.rs`, `src/hook.rs`,
  `src/rules/mod.rs`
- ADR-0020 (the keys of `nuc.toml`), ADR-0023 (*Fail closed* and *Hooks*).

## Blocked by

- None (can start immediately)

## Mode

with the human — `src/shell/` and the parser of `nuc.toml` are protected parts of ADR-0007.
