# Ticket: A round of review costs a whole parity run

## What to build

Tickets 05, 06, and 07 of `sensors-under-nuc-sense` took eleven rounds of review together, and
each round ended in a run of `scripts/check-all.sh`. The twelve logged runs took between 728 and
1348 s, 3.2 hours in all. The parity harness took all but 2 to 8 s of each run, since the other
checks run beside it. Every red result of the parity harness in that work came from the rows of
the sensor that the ticket ported:

- the variants `template-release-deleted` and `template-release-latest` for ticket 05;
- `adr-refs-invalid-utf8` for ticket 07.

The revisions of 2026-11-14 and 2026-11-18 cut the cost of the parity job on CI, but not of the
local run that each hand-off needs.

Measure what the comparisons of the sensors cost beside those of the checks and the hook
scenarios, on this machine and at the default number of jobs. Then propose to the human the change
that saves most without weakening the gate. Options to weigh:

- a `--sense <name>` of `scripts/check-parity.sh` for a session's own runs between rounds, while
  the run before the hand-off stays whole;
- fewer sensor rows per tree, where two rows decide the same case;
- a cost of each sensor scenario in the deal of the shards.

## Acceptance criteria

- [ ] Given the timings of a whole parity run, when the session reports them, then the hand-off
  names the share of the sensor comparisons, measured, not estimated.
- [ ] Given the change the human chose, when the session runs it, then the hand-off states the
  time it saves, and `scripts/check-all.sh` still runs every comparison.

## Read first

- Touches: `scripts/check-parity.sh`, `scripts/lib/sensor-parity.sh`, `tests/parity/sensors.tsv`,
  `tests/parity/sensor-scenarios.sh`
- TEST-parity-harness; API-sensor-cores-ask-the-shell

## Blocked by

- None (can start immediately)

## Mode

with the human: which comparisons a run may leave out is the human's to decide, and
DEV-one-agent-one-ticket allows no unattended work.
