# Ticket: Propose the four ADRs the port needs

## What to build

Four `proposed` ADRs, written through the `propose-adr` skill with the human, each in commits of
ADRs only, so that the human decides them before any code of the port is written:

1. **The key of a finding in the output.** Each finding carries the reason the bash names, and the
   text, JSON, and SARIF output show the key `<rule>:<reason>`. Supersedes
   API-command-line-with-hook-signal and API-output-formats-short-description, whose clauses on
   everything but the shape of a finding it restates unchanged. It settles where in the text line
   the key stands (the bash of v0.9.1 appends ` [<key>]`), the JSON field, the SARIF property, and
   whether the JSON document's version moves.
2. **Testing against the template's fixture tables.** The rules are tested against copies of the
   fixture tables of the newest template release `nuc` ports, each pinned by its blob at that tag,
   with the cases not yet ported listed with their rule and reason. It says what a take-up of a
   later release does to the copies, and how this relates to the copied self-test cases of v0.8.1.
3. **No rule that runs the project's scripts.** `nuc` ports no bash rule whose decision needs a
   script of the project run, as `shell/bash-floor` and `shell/root` do (Q-7). A `nuc.toml` that
   lists such a rule is refused as unknown, so the template keeps those rules in bash.
4. **Which check the `pre-commit` shim runs.** From v0.9.0 the bash `pre-commit` hook runs
   `check-docs.sh` on an ordinary commit off `main` and `check-all.sh --changed` on a concluded
   merge. Supersedes API-hook-contract so that `nuc hook pre-commit` names which check the shim
   runs, while a shim of a release before v0.9.0 keeps working.

## Acceptance criteria

- [ ] Given the spec of `docs/plan/port-v0-9-to-v0-11/`, when this ticket lands, then four ADRs
      stand under `docs/adr/` with status `proposed`, each with its row in the index, each in a
      commit that touches ADR files only.
- [ ] Given each ADR, when `scripts/check-docs.sh` runs, then it passes.
- [ ] Given the four ADRs, when the human decides each, then the later tickets follow the accepted
      ones; a rejected one sends its ticket back to the spec.

## Read first

- Touches: `docs/adr/`, `docs/adr/README.md`
- The spec of this directory, its *Implementation decisions* and *ADR impact*.
- docs/adr/API-command-line-with-hook-signal.md, docs/adr/API-output-formats-short-description.md,
  docs/adr/API-hook-contract.md, docs/adr/TEST-parity-harness.md,
  docs/adr/TEST-copies-keep-old-identifiers.md.
- The template's v0.9.1 tag message and its `scripts/fixtures/check-docs-record.txt` header, read
  with `git show` from a clone of the template, never run.

## Blocked by

- None (can start immediately)

## Mode

with the human: ADRs are the human's to decide (DEV-one-agent-one-ticket), and `propose-adr`
interrogates the human in rounds.
