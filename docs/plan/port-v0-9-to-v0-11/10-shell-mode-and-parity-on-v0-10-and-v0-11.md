# Ticket: shell/mode, and parity on v0.10.0, v0.11.0, and this repository

## What to build

`shell/mode`, a new rule, reports a script under `scripts/` or `.agents/skills/*/` whose mode in
git's index is not 100755, symbolic links, subdirectories, and conflicted files exempt, as the bash
of v0.10.0 reads `git ls-files -s`. It is the one rule of `check-shell.sh` that `nuc` ports.

Then the corpus pins the template at the commits of v0.10.0 and v0.11.0, and this repository at a
commit of its `main` on v0.11.0. The harness runs `check-shell.sh` of a tree that has the rule for
`shell/mode` only, the rest of its output left out as not ported, and runs each tree's scripts as
its release reads them, the root by `--root` from v0.10.0.

## Acceptance criteria

- [ ] Given the cases of the copied `check-shell.txt` for `shell/mode`, when the runner runs, then
      they agree, and the pending list is empty; the cases of `shell/bash-floor` and `shell/root`
      stay listed as not ported.
- [ ] Given a script under `scripts/` with mode 100644 in the index, when `nuc check` runs, then
      `shell/mode` reports it with the key the bash names.
- [ ] Given the corpus with the template at v0.10.0 and v0.11.0 and this repository, when the
      parity harness runs, then it agrees on each, their variants included.
- [ ] Given a `nuc.toml` that lists `shell/bash-floor`, when `nuc check` runs, then it is refused
      as an unknown rule.

## Decided

- The human accepted ARCH-rules-need-only-git on 2026-11-15. This ticket writes its Enforcement,
  the refusal in `tests/contract.rs` and the one spawn in `tests/functional_core.rs`, and flips it
  to accepted in that change.

## Read first

- Touches: a new rule module, `src/rules/mod.rs`, `src/shell/mod.rs`, `src/shell/git.rs`,
  `tests/contract.rule-ids`, `tests/parity/corpus.toml`, `tests/parity/releases.tsv`,
  `tests/parity/variants.tsv`, `tests/parity/normalize.tsv`, `scripts/check-parity.sh`,
  `tests/git_cli.neutralized`, `tests/functional_core.rs`, `tests/contract.rs`, `docs/adr/`
- The template's `scripts/check-shell.sh` at v0.10.0, its `shell/mode` part only.
- ARCH-rules-need-only-git.
- Seam: the table runner; the parity harness.

## Blocked by

- docs/plan/port-v0-9-to-v0-11/09-the-checks-of-v0-10.md

## Mode

with the human: the parity harness and the corpus, and a new git command in the shell.
