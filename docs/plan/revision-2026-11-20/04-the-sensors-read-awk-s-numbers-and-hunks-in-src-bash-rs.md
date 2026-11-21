# Ticket: The sensors read awk's numbers and git's hunk headers in src/bash.rs

## What to build

`docs/ARCHITECTURE.md` says that the readings of the bash's tools stand in `src/bash.rs`, and
`tests/functional_core.bash` lists the names that the structural test holds there. The sensors
kept two readings for themselves:

- `awk_number` twice, in `src/sense/binding_adrs.rs` and `src/sense/plain_rules.rs`. On an
  overflow one saturates and the other reads 0. `tests_kept::hunk_start` reads awk's number a third
  time. `tests/functional_core.bash` lists none of these names, so the structural test does not see
  them.
- three readers of a hunk header, each copying another awk pass of its bash:
  - `tests_kept::hunk_start` takes the first `+<digits>` anywhere on the line;
  - `binding_adrs::held_in_hunks` reads `^@@ [^+]*\+([0-9,]+) ` and an empty count as 1;
  - `plain_rules::added_lines` reads after `@@ `.

The evidence is that review found this class twice in the same work. In ticket 06 of
`sensors-under-nuc-sense`, `plain_rules::unquoted` repeated `adr::awk_unescape` (fifth review,
finding 2). In ticket 05, a regex interner was written by hand (second review, finding 3).

Move the reading of awk's number and the readings of a hunk header to `src/bash.rs`, and list
their names in `tests/functional_core.bash`. Each reading keeps the quirks of its bash, as "A port
copies the bash, quirks included" asks. So there is one function per awk pass, or one function
with a parameter for the pass, never one reading that drops a quirk.

awk reads a number past 64 bits as a double, which none of the three reads today. git writes no
such hunk header, so no corpus row has one. A sensor that meets one must refuse it, exit 2, as not
decided.

## Acceptance criteria

- [ ] Given `tests/functional_core.bash`, when it lists the reading of awk's number and of a hunk
  header, then the structural test passes, and each stands only in `src/bash.rs`.
- [ ] Given a hunk header whose number runs past 64 bits, when each of the three sensors reads it,
  then the sensor ends with exit 2 as not decided, as a unit test of `src/bash.rs` and one test of
  `tests/contract.rs` decide.
- [ ] Given each awk pass of the three bash sensors, when the unit tests of `src/bash.rs` run, then
  each pass's quirk is decided: the first `+<digits>` anywhere, an empty count as 1, and the text
  after `@@ `.
- [ ] Given the parity rows of `tests/parity/sensors.tsv`, when the harness runs, then they pass
  unchanged.

## Read first

- Touches: `src/bash.rs`, `src/sense/binding_adrs.rs`, `src/sense/plain_rules.rs`,
  `src/sense/tests_kept.rs`, `tests/functional_core.bash`
- ARCH-rules-outcome-hooks-answer; API-sensor-cores-ask-the-shell; the convention "A port copies
  the bash, quirks included" in `docs/CONVENTIONS.md`

## Blocked by

- None (can start immediately)

## Mode

with the human: DEV-one-agent-one-ticket allows no unattended work.
