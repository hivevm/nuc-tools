# Ticket: nuc hook pre-push runs no filter to tell a clean tree

## What to build

`nuc hook pre-push` tells a clean working tree with `git status --porcelain`, to decide whether the
tree's green record holds. `git status` hashes each stat-dirty tracked file through the filter its
attributes name, running the driver's `clean` or `process` command, and asks each checked-out
submodule for its own status, which runs that submodule's filters (Q-7). The review of ticket 05 of
`sensors-under-nuc-sense` found it; it was recorded as a backlog entry, which this ticket replaces.

SEC-repository-commands-neutralized-or-refused decides the fix. Take a checked-out submodule as not
clean first, before any git looks into it. Then ask the status with every active filter driver
neutralized, as `neutralized_filters` of `src/shell/sense.rs` does, and without
`--ignore-submodules`: with no submodule checked out, git runs nothing inside one, and a staged
change of a gitlink shows as the bash's git shows it. Take the tree as not clean where a tracked
path under an active filter is stat-dirty, whatever its raw bytes, or its stat is no older than the
index, since git then compares the content. Ask `git diff-files --name-only` for the stat-dirty
paths with the same neutralizing keys, since git hashes a racily clean entry. `diff-files` does not
name a racily clean path whose raw bytes equal the index, so compare the modification time of each
tracked path under an active filter with the index file's in whole seconds, as git compares them, in
`src/shell/`. Share the reading of the drivers with `shell::sense` rather than writing it again.

## Acceptance criteria

- [ ] Given a repository whose filter driver writes a canary file, and a stat-dirty file under it,
  when `nuc hook pre-push` runs, then the canary is not created.
- [ ] Given a stat-dirty file under an active filter whose raw bytes equal the index, as a file
  committed before its filter was set, when the hook runs, then it runs the checks rather than
  taking the green record.
- [ ] Given a tree with a checked-out submodule, when the hook runs, then no git runs inside the
  submodule, and the hook runs the checks.
- [ ] Given a staged change of the gitlink of a submodule that is not checked out, when the hook
  runs, then it runs the checks, as the bash hook does.
- [ ] Given a file under an active filter whose stat matches the index and whose modification time
  falls in the same second as the index's, below it by a fraction, when the hook runs, then no
  filter runs and the hook runs the checks.
- [ ] Given the hook scenarios of `tests/parity/hook-scenarios.sh` without filters or submodules,
  when the harness runs, then they pass unchanged.

## Read first

- Touches: `src/shell/hook.rs`, `src/shell/sense.rs`, `src/shell/git.rs`, `tests/hooks.rs`,
  `tests/parity/hook-scenarios.sh`
- SEC-repository-commands-neutralized-or-refused; SEC-git-cleared-without-lazy-fetch

## Blocked by

- None (can start immediately)

## Mode

with the human: it changes the code that spawns git, a protected part of DEV-one-agent-one-ticket.
