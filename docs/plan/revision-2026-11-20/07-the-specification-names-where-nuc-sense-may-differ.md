# Ticket: The specification names where nuc sense may differ from the bash

## What to build

This is a drafted change of the specification. The human lands it or declines it
(NUC-documents-are-fixed §2).

Two criteria no longer say what the tests decide for `nuc sense`:

- **Q-2** asks for zero differences in findings and exit status for every ported script. For
  the sensors, `tests/parity/sensor-exits.tsv` lists exits that differ by
  API-git-only-sensors-under-sense, and the harness passes them. Today the list has two rows:
  revision-due and plan-frontier exit 1 where `nuc` exits 2.
- **G-9** says `nuc sense` prints the bash's report byte for byte "or stops with exit 2 where it
  cannot decide". Two known cases differ from that:
  - a filter driver configured only in the environment, which the cleared environment hides from
    `nuc`, so plain-rules can pass where the bash fails;
  - a file whose index holds bytes its filter would clean otherwise, which a sensor can read
    otherwise than the bash, as `docs/ARCHITECTURE.md` says.

  A hand-off that the shell cannot read, where review-binding exits 2 and the bash exits 0, is no
  such case: G-9 already allows exit 2. It is an exit that differs under Q-2, which no row of
  `tests/parity/sensor-exits.tsv` lists, since no scenario builds it. The hand-offs of tickets 06
  and 07 of `sensors-under-nuc-sense` name the two cases and the hand-off as known differences of
  the code.

Draft for the human:

- Q-2: "zero differences ..., except an exit of a sensor that an accepted ADR lists with its cause".
- G-9: either it names the configuration `nuc` does not see as outside its promise, or it stands,
  and ticket 02 of this revision closes the two cases by a refusal.

The accepted ADRs it would move: API-git-only-sensors-under-sense (*Parity*, whose second table
lists each bash exit that differs), and SEC-git-cleared-without-lazy-fetch, if G-9 is to close the
case of the environment.

## Acceptance criteria

- [ ] Given the human's choice, when the session lands it, then Q-2 and G-9 say what
  `tests/parity/sensor-exits.tsv` and the known differences decide, or the code refuses the two
  cases and G-9 stands.

## Read first

- Touches: `docs/SPECIFICATION.md`, `tests/parity/sensor-exits.tsv`
- API-git-only-sensors-under-sense; SEC-git-cleared-without-lazy-fetch; NUC-documents-are-fixed §2

## Blocked by

- docs/plan/revision-2026-11-20/02-propose-what-nuc-does-about-a-repository-s-commands.md

## Mode

with the human: a change of the specification is the human's.
