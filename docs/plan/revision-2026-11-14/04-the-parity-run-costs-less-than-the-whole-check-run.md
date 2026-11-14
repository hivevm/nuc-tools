# Ticket: The parity run costs less than the whole check run

## What to build

`scripts/check-parity.sh` took 774 of the 775 seconds of `scripts/check-all.sh` on the head of pull
request #84, and 751 of 754 and 723 of 727 in the two runs before it, and it passed in every run of
the work since the revision of 2026-11-08 that the logs under `.scratch/` keep. The pre-push hook
runs it whenever its inputs change, and `scripts/lib/` is among them, which every template take-up
changes. Find where the time goes (the fetch of the corpus, the bash runs per tree and variant, the
`nuc` runs, the hook scenarios), measure it, and cut the largest part without dropping a tree, a
variant, or a scenario: for example, by keeping the bash outputs of a pinned tree and variant under
`.scratch/parity/`, keyed by the commit and the variant, since a pinned tree's bash output does not
change.

## Acceptance criteria

- [ ] Given a run of `scripts/check-parity.sh` with a filled cache, when it is timed, then the
      hand-off names the seconds before and after, per part.
- [ ] Given a corpus or a variant changed, when the harness runs, then it computes what changed
      again and reports the same findings as without the cache.
- [ ] Given the parity self-test, when it runs, then it passes, with a case where a stale cache
      entry is not used.

## Read first

- Touches: `scripts/check-parity.sh`, `scripts/lib/hook-parity.sh`, `scripts/test-check-parity.sh`
- TEST-parity-harness and TEST-hook-scenario-parity; `.scratch/check-all-takeup-0.11.0.log` and
  `.scratch/push-tickets-03-04.log` for the times.

## Blocked by

- None (can start immediately)

## Mode

with the human — DEV-one-agent-one-ticket allows no unattended ticket, and the harness is the
oracle of every port.
