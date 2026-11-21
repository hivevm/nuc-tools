# Ticket: the take-ups of the corpus are replayed by `nuc upgrade`, and the command is documented

## What to build

A test that proves `nuc upgrade` does what a human did. For each take-up commit it lists, at least
this repository's take-up of v0.12.1 from v0.12.0 and one take-up of a derived repository of the
corpus, it checks out the parent, fetches the two template tags as the corpus fetches its trees,
runs `nuc upgrade`, and compares the working tree with the take-up commit's: every path not listed
as a judgment must match. The README of this repository documents `nuc upgrade` beside the other
commands, and CONTRIBUTING.md says how a maintainer of `nuc` adds a take-up to the list.

## Acceptance criteria

- [ ] Given the list of take-ups, when the replay runs, then each path `nuc upgrade` wrote or
      deleted matches the take-up commit, and each path that differs is one it listed.
- [ ] Given a take-up where `nuc upgrade` writes a path the human left alone, when the replay
      runs, then it fails and names the path.
- [ ] Given the replay, when CI runs, then it runs in the job that holds the network for the
      corpus, not in a test that must run offline (Q-11).
- [ ] Given the README, when a maintainer of a derived project reads it, then it says how to fetch
      the two tags, run the command, and settle the list.

## Read first

- Touches: `tests/parity/`, `scripts/check-parity.sh`, `README.md`, `CONTRIBUTING.md`
- `docs/adr/TEST-parity-harness.md`: how the corpus is pinned and fetched.
- Golden path: `tests/parity/corpus.toml` and how `scripts/check-parity.sh` clones a pinned tree.
- Seam: the script of the corpus, as the parity harness runs.

## Blocked by

- docs/plan/nuc-upgrade/04-citations-and-skipped-releases.md

## Mode

with the human: the take-ups to replay and what counts as a match on a judgment are the
maintainer's to pick.
