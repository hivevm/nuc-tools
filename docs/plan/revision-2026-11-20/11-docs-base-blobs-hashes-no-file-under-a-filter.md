# Ticket: docs/base-blobs hashes no file under a filter

## What to build

The rule `docs/base-blobs` of `nuc check` reads the base through `Tree::read_base` in
`src/shell/tree.rs`. It runs `git hash-object` on the `NUC-` files of the record and on
`AGENTS.md`, with the filters `.gitattributes` names, so that the blobs match the bash's. A filter's
`clean` or `process` command then runs (Q-7). The review of
SEC-repository-commands-neutralized-or-refused reproduced it with git 2.56.1.

That ADR decides: where a check hashes a file of the working tree to compare its blob, and an active
filter covers the file, the check is not decided. Read the active drivers as `shell::sense` does
(`neutralized_filters`), sharing that reading. Ask `check-attr` for the exact paths `read_base`
hashes, untracked `NUC-` files and `AGENTS.md` included, rather than for the tracked paths
`neutralized_filters` lists, and refuse before any hash where one is under an active driver. Hash
the rest as today, so that line endings convert as for the bash. Rewrite the doc comment of
`read_base`, which cites the trust in the configuration, to cite the ADR's refusal instead.

## Acceptance criteria

- [ ] Given a repository whose `.gitattributes` names an active filter for `docs/adr/*.md`, and
  whose driver writes a canary file, when `nuc check` runs, then the canary is not created and the
  run exits 2, naming the file.
- [ ] Given an untracked `NUC-` file of the record under an active filter, when `nuc check` runs,
  then no filter runs and the run exits 2.
- [ ] Given a filter attribute whose driver no entry makes active, when `nuc check` runs, then
  `docs/base-blobs` decides as today.
- [ ] Given the rules' parity, when the harness runs, then it passes unchanged.

## Read first

- Touches: `src/shell/tree.rs`, `src/shell/sense.rs`, `tests/check.rs`
- SEC-repository-commands-neutralized-or-refused; SEC-git-cleared-without-lazy-fetch

## Blocked by

- None (can start immediately)

## Mode

with the human: it changes the code that spawns git, a protected part of DEV-one-agent-one-ticket.
