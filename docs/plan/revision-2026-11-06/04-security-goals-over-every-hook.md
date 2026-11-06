# Ticket: Run the tests of Q-7 to Q-11 over every hook

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The spec of the hooks' port asked that the isolation tests of Q-7 to Q-11 extend to `nuc hook`.
Each ticket wrote its own, and two reviews found one missing (#50: the decoy test of ADR-0011 did
not reach `nuc hook`; #52: the test of Q-8 and Q-9 sits in `tests/hooks.rs`, not beside the
others). What stands now covers the hooks unevenly:
- Q-7, a canary script unrun when `nuc hook <name>` is called directly, as ADR-0021's
  *Enforcement* names it: `pre-commit` only, though `pre-merge-commit` and `pre-push` answer 3
  as well;
- Q-9, a decoy repository through git's variables: `pre-commit` (`tests/git_cli.rs`) and
  `pre-push`;
- Q-10, the canary token: `pre-commit` and `pre-push`;
- Q-11, the network blocked: the cases of `pre-commit` only, though Q-11 says every hook.

`tests/hooks.rs` already drives the fail-closed cases over `every_shim()` with `trigger()`, which
fails on a shim it has no git command for. Drive the four goals the same way, so that a hook added
later cannot skip them: each test runs over every shim under `shims/`, the merge driver included.

## Acceptance criteria

- [ ] Given each shim under `shims/`, when its hook is run through git with a canary script as
  the check, a decoy repository in `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and
  `GIT_CONFIG_PARAMETERS`, and the canary token in the environment and the configuration, then
  the canary stays unrun where `nuc hook` is called directly, the decoy is untouched, and the
  token is in no output and no written file.
- [ ] Given each shim, on Linux, when its hook runs with the network blocked, then its exit status
  and output equal those of the run with the network.
- [ ] Given a new file under `shims/` that `trigger()` does not know, when the suite runs, then
  these tests fail as the fail-closed test does.

## Read first

- Touches: `tests/hooks.rs`, `tests/git_cli.rs`, `tests/isolation.rs`
- `docs/SPECIFICATION.md`, Q-7 to Q-11; ADR-0021 and ADR-0011, *Enforcement*; ADR-0012.
- The test pattern in `docs/ARCHITECTURE.md`; `every_shim()` and `trigger()` in
  `tests/hooks.rs`.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket, and these tests decide the security goals.
