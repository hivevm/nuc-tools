# Ticket: The hooks of v0.9.0 and v0.11.0

## What to build

`nuc hook` decides as the bash hooks of v0.9.0 to v0.11.0:

- from v0.9.0, `pre-commit` on a commit off `main` that concludes no merge answers that the
  documentation check runs, and the shim runs `scripts/check-docs.sh` and refuses on red; on a
  concluded merge the shim runs `scripts/check-all.sh --changed` as today, as the ADR accepted from
  ticket 01 decides how the shim learns which;
- from v0.9.0, `pre-push` gives no notice on drafts;
- from v0.11.0, `pre-merge-commit` refuses a merge on a `feat/` branch, and `pre-commit` refuses
  the conclusion of one.

Each new decision is a hook rule of its own, with its ID. A tree whose README names no release is
decided as v0.11.0, the newest release `nuc` then knows.

## Acceptance criteria

- [ ] Given the template's trees of v0.9.0 and v0.11.0 in the corpus, when the hook scenarios run,
      then the bash hook and the shim with `nuc` agree on every scenario, new ones for these
      decisions included.
- [ ] Given a tree on v0.8.1, when each hook runs, then it decides as today, the draft notice
      included.
- [ ] Given a shim of a release before v0.9.0, when `nuc hook pre-commit` answers, then that shim
      still runs what it ran before.
- [ ] Given the new hook rule IDs, when the contract test runs, then they are listed, on without
      a `nuc.toml` and governed by its list where there is one.

## Read first

- Touches: `src/hook.rs`, `src/shell/hook.rs`, `shims/pre-commit`, `shims/pre-merge-commit`,
  `src/config.rs`, `tests/hooks.rs`, `tests/contract.rule-ids`, `tests/parity/hook-scenarios.sh`,
  `scripts/lib/hook-parity.sh`
- API-hook-contract or the ADR that supersedes it from ticket 01; TEST-hook-scenario-parity;
  API-version-coupling-hook-defaults *Hooks*.
- The template's `.githooks/pre-commit`, `.githooks/pre-merge-commit`, `.githooks/pre-push`,
  diffed v0.8.1 to v0.11.0.
- Seam: the hook scenarios.

## Blocked by

- docs/plan/port-v0-9-to-v0-11/01-propose-the-adrs-of-the-port.md
- docs/plan/port-v0-9-to-v0-11/10-shell-mode-and-parity-on-v0-10-and-v0-11.md

## Mode

with the human: the hooks and their fail-closed path, protected under DEV-one-agent-one-ticket.
