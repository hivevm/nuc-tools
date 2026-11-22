# Ticket: nuc upgrade is held to Q-9, Q-10, and Q-12

## What to build

The spec of `nuc upgrade` named Q-7, Q-8, Q-9, Q-11, and Q-12 as served: "the first command that
writes many files must keep each of them". The tests landed for Q-7 and Q-8 alone, in
`tests/isolation.rs`. No test runs `nuc upgrade` against Q-9, Q-10, or Q-12, though each speaks of
every command, and `nuc upgrade` is the first command that writes many files of a tree:

- **Q-9:** with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and `GIT_CONFIG_PARAMETERS` set to a
  decoy, a take-up leaves the decoy untouched and writes the repository it runs in.
- **Q-10:** a canary token in the environment and in the git configuration appears in no output of
  a take-up and in no file it writes.
- **Q-12:** a `README.md` or a `nuc.toml` that is a symbolic link out of the repository ends the
  run with exit 2 and a message naming it, and nothing outside is read. A directory above a path
  to write that links out is held under Q-8 already.

Q-11 binds checks and hooks only, and `nuc upgrade` reaches no network by its ADR. Whether Q-11
names it is the drafted change of ticket 08.

This is a check to write: a mistake there would reach a project unseen.

## Acceptance criteria

- [ ] Given a decoy repository named by the four variables, when `nuc upgrade` takes up a release,
  then the decoy's files, index, and refs are as before, and the repository it runs in is taken up.
- [ ] Given a canary token in the environment and in the git configuration, when `nuc upgrade`
  takes up a release that lists a judgment, then neither output nor any written file holds it.
- [ ] Given a `README.md` or a `nuc.toml` that is a symbolic link out of the repository, when `nuc
  upgrade` runs, then it exits 2 with a message naming the path, and the target is not read.
- [ ] Given the tests, when they run, then each cites `API-upgrade-by-blob-toml-optional` and its
  criterion with a `Verifies:` marker.

## Read first

- Touches: `tests/isolation.rs`, `tests/contract.rs`, `tests/support/upgrade.rs`
- The tests of Q-9 and Q-10 in `tests/hooks.rs` and of Q-12 in `tests/check.rs`, as the pattern
- Q-9, Q-10, and Q-12 in `docs/SPECIFICATION.md`; API-upgrade-by-blob-toml-optional
  (*Enforcement*); TEST-fixtures-with-own-home

## Blocked by

- None (can start immediately)

## Mode

with the human: DEV-one-agent-one-ticket allows no unattended work. A test that fails shows a
defect of the shell, which the maintainer judges.
