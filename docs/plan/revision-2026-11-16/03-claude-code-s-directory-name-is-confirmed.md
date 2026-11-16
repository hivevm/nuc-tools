# Ticket: Confirm how Claude Code names a working tree's directory, and test it from the harness's side

## What to build

`nuc measure` finds Claude Code's transcripts under `projects/<key>`. The key is the working tree's
path with each character but an ASCII letter or digit replaced by `-`, as read on Linux in this
repository. The rule is unconfirmed for:

- Windows paths with a drive letter;
- macOS, where the temporary directory is `/var`, a link to `/private/var`, and git's toplevel
  names the real path;
- paths with non-ASCII characters;
- long paths;
- sessions run in a git worktree.

The test of the default directory builds the expected key with the same algorithm as the code, so
it cannot catch a difference. A mismatch fails closed with exit 2, and `--transcripts` helps, but
the default is what a maintainer runs.

Read Claude Code's documentation or its behaviour for each case. Write the expected keys as fixed
strings in the tests, per platform where they differ, and change the code where it differs.

## Acceptance criteria

- [ ] Given each case above, when a test builds the working tree's key, then it compares with a key
  written as Claude Code writes it, not computed by the code under test.
- [ ] Given CI on Linux, macOS, and Windows, when the tests run, then the default directory is found
  on each.

## Read first

- Touches: `src/shell/measure.rs` (`default_dir`), `tests/measure.rs`
- API-measure-reads-harness-transcripts; the CI matrix of Q-4

## Blocked by

- None (can start immediately)

## Mode

with the human: it decides what `nuc measure` reads outside the repository by default.
