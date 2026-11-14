# Ticket: Take up v0.10.0

## What to build

The repository works under v0.10.0. The `NUC-` files, `AGENTS.md`, and `docs/adr/NUC.blobs` are
replaced by the release's. The files the notes list are taken up, merged where the project changed
them. Every script of the project under `scripts/`, other than its self-tests and the check runner,
takes its root only as `--root <dir>` through `scripts/lib/root.sh` and refuses an argument it does
not read. Every script is tracked 100755. Callers in workflows, hooks, and scripts pass `--root`.
The project's ADRs that name a check of `check-docs.sh` by number are each either superseded by one
naming the rule key or left to bind the whole file, as the human decides. The `Template release`
line moves to v0.10.0.

## Acceptance criteria

- [ ] Given the take-up, when `scripts/check-all.sh` runs, then it is green, the rules
  `shell/root`, `shell/bash-floor`, `shell/mode`, and `docs/check-lists` included.
- [ ] Given a project script called with a positional root or an unknown argument, when it runs,
  then it exits 2.
- [ ] Given OPS-rust-stable-behind-scripts, API-version-coupling, TEST-parity-harness, and
      API-version-coupling-hook-defaults under their new identifiers, when each is read for a check
      named by number, then the hand-off records the human's decision on each, and a superseding ADR
      stands in commits of ADRs only before the code that depends on it.
- [ ] Given the design revision's due line, when it is read, then it keeps the project's own
  number.
- [ ] Given the pull request template, when it is read, then it carries the `Measures:` line and
  the four fields of the Doubts.

## Read first

- Touches: `docs/adr/NUC-*`, `docs/adr/NUC.blobs`, `docs/adr/README.md`, `AGENTS.md`,
  `README.md` (Template), `scripts/`, `.agents/skills/`, `.github/`, `.githooks/merge-adr-index`,
  `.editorconfig`, `.gitattributes`, `CONTRIBUTING.md`
- The release notes of v0.10.0, "Taking it up", which maps the check numbers to rule keys.
- The template at v0.10.0: `scripts/lib/root.sh` and one script that uses it, as the golden path
  for the project's own scripts.

## Blocked by

- docs/plan/template-v0-11-0/02-take-up-v0-9-1.md

## Mode

with the human: the supersessions of the project's ADRs are the human's decision, and the hooks and
workflows it changes are kept with the human by the harness ADR.
