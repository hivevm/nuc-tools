# Ticket: Read `nuc.toml` into the rules it turns on in one call

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Since ticket 02 of the last revision, `config` answers which IDs `nuc.toml` may name, but turning
a configuration into the rules it turns on is still three steps every caller strings together:
`config::load`, `config::known`, then a selection. `nuc check` selects through `config::select`
over `rules::ALL`; `turned_on` in `shell::hook` writes the same filter again over `hook::ALL`,
since `select` takes only a `Rule`. A caller that skips `known` turns on less than the file
names and refuses nothing, which ADR-0023 (*Fail closed*) rules out.

Give `config` one function that loads the text, refuses as `load` and `known` refuse, and
returns the rules of a given list it turns on, for the rules and the hook rules alike; `nuc
check` and `turned_on` call it, and the second filter goes. No exit code, message, or order of
refusals changes.

The header of `tests/functional_core.outcomes` still says that `rules::find` gives the shell a
rule for `nuc explain`; the shell now asks `config::explanation`, which returns only an ID and
a reason. Correct the sentence in the same change.

## Acceptance criteria

- [ ] Given a `nuc.toml` with an unknown ID, a minimum above the running version, or neither,
  when `nuc check` and `nuc hook pre-commit` run, then each exits and words its refusal as
  before; `tests/hooks.rs` `nuc_check_and_nuc_hook_refuse_an_unknown_id_alike` and the tests of
  `tests/contract.rs` pass unchanged.
- [ ] Given the source, when `src/shell/` is read, then it filters no rule list by the IDs of a
  configuration itself; a unit test of `config` shows the one function over a rule list and a
  hook rule list.
- [ ] Given `tests/functional_core.outcomes`, when its header is read, then it names what the
  shell calls for `nuc explain`.

## Read first

- Touches: `src/config.rs`, `src/shell/mod.rs`, `src/shell/hook.rs`,
  `tests/functional_core.outcomes`
- ADR-0020 (the keys of `nuc.toml`), ADR-0023 (*Fail closed*, *Hooks*), ADR-0024 (*Shell*).
- The *Configuration* block of `docs/ARCHITECTURE.md`.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket.
