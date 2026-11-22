# Ticket: Decide where the decisions of nuc upgrade stand beside the rules and the hooks

## What to build

ARCH-rules-outcome-hooks-answer decides two kinds of decision: rules, each a module of
`src/rules/` with one rule ID, listed in `rules::ALL` and returning an outcome; and hooks, decided
in `src/hook.rs`, listed in `hook::ALL` and returning an answer. Its *Consequences* say there are
two, and its *Follow-ups* ask "whether the two kinds of decision fold into one at a later
normalization, if a later command returns a third". `nuc upgrade` returns a third:

- `src/upgrade.rs` holds three rule IDs, `upgrade/file`, `upgrade/citation`, and `upgrade/notes`,
  in one module outside `src/rules/`, with a `Rule` type and a list `upgrade::ALL` of its own;
- its decision returns a `Plan` and refuses with an `upgrade::Refusal` keyed `upgrade/run`, beside
  a rule's `Outcome`, a hook's answer, and the refusals of `config` and of `nuc sense`;
- `nuc explain` asks `config::explanation` and then `upgrade::find`, and the tests of IDs and keys
  in `tests/contract.rs` chain `nuc::upgrade` beside the other two lists.

API-upgrade-by-blob-toml-optional decides what the command does, its rules and keys among them. No
ADR decides this shape of the core, and the structural test's rule of one module per rule ID does
not reach it.

The trigger the ADR named has come. The hand-off of ticket 02 of `nuc-upgrade` left it under
*Unsure*. `upgrade/file` stands in `tests/contract.rule-ids` under a header that speaks of the IDs
`nuc.toml` names, with a comment that no `nuc.toml` names it. `nuc explain` reaches it outside
`config`.

The maintainer chooses one of these:

- **Record the third kind:** an ADR superseding ARCH-rules-outcome-hooks-answer names the
  decisions of a command beside the rules and the hooks, what each returns, and where its rule IDs
  are listed and explained, with the structural test extended to them.
- **Fold the kinds into one:** one list of every rule ID with its reason and kind, which `config`,
  `nuc explain`, and the contract tests read alike, each kind keeping its own return type.
- **Leave it:** the record stays as it reads, and this ticket is dropped with the reason in its
  commit.

## Acceptance criteria

- [ ] Given the maintainer's choice, when the session lands it, then a `proposed` ADR supersedes
  ARCH-rules-outcome-hooks-answer and names the decisions of `nuc upgrade`, or the ticket is
  deleted with the reason.
- [ ] Given a proposed ADR, when `docs/ARCHITECTURE.md` describes the Rules, Hooks, and Upgrade,
  then it says what the ADR decides and nothing the record does not.

## Read first

- Touches: `docs/adr/`, `src/upgrade.rs`, `src/config.rs`, `src/shell/mod.rs`,
  `tests/functional_core.rs`, `tests/contract.rs`, `tests/contract.rule-ids`
- ARCH-rules-outcome-hooks-answer, in full; API-upgrade-by-blob-toml-optional (*Output*); the
  skill `propose-adr`

## Blocked by

- None (can start immediately)

## Mode

with the human: the choice moves an accepted ADR, which the human decides.
