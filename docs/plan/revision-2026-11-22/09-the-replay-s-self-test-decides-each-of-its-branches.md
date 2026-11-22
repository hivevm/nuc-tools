# Ticket: The replay's self-test decides each of its branches

## What to build

`scripts/lib/take-up-parity.sh` replays the take-ups of `tests/parity/take-ups.tsv` with
`nuc upgrade`, and `scripts/test-check-parity.sh` holds it. The second review round of ticket 05
of `nuc-upgrade` left lesser findings open, which are the evidence:

- no case reaches "its replay could not be set up";
- the self-test holds a differing executable bit in one direction only;
- the path of `take-up-differences.tsv` is computed twice, in `take_up_load` and
  `take_up_replays`;
- one case counts its expected paths by hand;
- only the last shard reads the two tables, so a malformed table passes shards 1 to 5.

The self-test gains a case for each branch, and the replay reads its table's path once. Whether
every shard reads the tables is weighed against the cost of a shard that runs no replay.

## Acceptance criteria

- [ ] Given a replay whose repository cannot be set up, when the self-test runs it, then the
  harness fails and names the take-up.
- [ ] Given a take-up whose file has the executable bit and a replay that writes it without, and the
  reverse, when the self-test runs each, then each fails and names the path.
- [ ] Given the change, when `scripts/lib/take-up-parity.sh` is read, then the path of the table of
  differences is computed in one place.
- [ ] Given a malformed table of take-ups, when a shard other than the last runs, then it fails
  with exit 2, or the hand-off gives the reason it does not.

## Read first

- Touches: `scripts/lib/take-up-parity.sh`, `scripts/check-parity.sh`,
  `scripts/test-check-parity.sh`
- TEST-parity-harness; API-upgrade-by-blob-toml-optional (*Enforcement*, the replay); the
  convention "A self-test asserts what it builds" in `docs/CONVENTIONS.md`

## Blocked by

- None (can start immediately)

## Mode

with the human: DEV-one-agent-one-ticket allows no unattended work.
