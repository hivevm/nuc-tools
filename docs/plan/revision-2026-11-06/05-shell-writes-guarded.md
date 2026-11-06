# Ticket: Hold the shell's writes as its reads are held

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The review of the merge driver (#54) found a blocker: the driver made its scratch directory under
the system's temporary directory with a predictable name, which another user could take over with
a symbolic link, and which broke Q-8. It is fixed, and the directory is now made fresh in the git
directory. Nothing would catch the next such write. The structural test holds every read and
listing of the shell to the functions `tests/functional_core.guarded` names, but no write: since
the hooks, `src/shell/hook.rs` writes the pending tree, the record of green trees and its
temporary file, the merged index, and the scratch directory, and removes files, in six functions,
`pre_push`, `record_pending`, `remove`, `merge_adr_index`, `scratch_dir`, and `merge_sides`, which
no list names.

Extend the structural test to the writes: a write, a rename, a creation or removal of a file or
directory in `src/shell/` stands only in a function a list beside the test names, with the paths
it writes said in a comment, as for the reads; and `std::env::temp_dir` stands nowhere in
`src/`, since every write stays inside the repository and its git directory (Q-8).

## Acceptance criteria

- [ ] Given a write added to a function of `src/shell/` that the list does not name, when the
  structural test runs, then it fails naming the function.
- [ ] Given `std::env::temp_dir` written anywhere under `src/`, when the structural test runs,
  then it fails.
- [ ] Given today's source, when it runs, then it passes with the writes of `shell::hook` listed.

## Read first

- Touches: `tests/functional_core.rs`, `tests/functional_core.guarded`,
  `tests/functional_core.reads`, a list of writes beside them
- ADR-0024 (proposed, restating ADR-0010), *Enforcement*; Q-8.
- `tests/functional_core.rs`, `the_shell_reads_only_through_the_guard` and the test of a read
  outside the guarded functions, the pattern to copy.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket, and the structural test is a protected
part.
