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
  cannot decide". SEC-repository-commands-neutralized-or-refused decides that it stands for the
  commands a repository names: the three cases the reviews found where the code still reads
  otherwise than the bash are refused by tickets 09, 12, and 13 of this revision:
  - a command configured only in the caller's environment, a filter, textconv driver, or external
    diff (ticket 09);
  - a stat-dirty file under a filter whose raw bytes equal the index (ticket 12);
  - a path whose hunks a textconv driver, a diff command, `diff.external`, or `diff.submodule` of
    the repository would change (ticket 13).

  Configuration in the caller's environment that names no command, such as `core.autocrlf` or an
  `eol` attribute, can still make `nuc` read otherwise than the bash; the ADR leaves it out of
  scope. Either G-9 names it, or a later decision refuses it too.

  A hand-off that the shell cannot read, where review-binding exits 2 and the bash exits 0, is no
  such case: G-9 already allows exit 2. It is an exit that differs under Q-2, which no row of
  `tests/parity/sensor-exits.tsv` lists, since no scenario builds it.

Draft for the human:

- Q-2: "zero differences ..., except an exit of a sensor that an accepted ADR lists with its cause".
- G-9: unchanged, or "… where it cannot decide, configuration of the caller's environment that names
  no command aside", if the human takes that configuration into G-9 rather than a later refusal.

The accepted ADR it would move: API-git-only-sensors-under-sense (*Parity*, whose second table
lists each bash exit that differs).

## Acceptance criteria

- [ ] Given the human's choice, when the session lands it, then Q-2 says what
  `tests/parity/sensor-exits.tsv` decides, and G-9 reads as the human chose: unchanged, or with the
  caller's configuration that names no command set aside.

## Read first

- Touches: `docs/SPECIFICATION.md`, `tests/parity/sensor-exits.tsv`
- API-git-only-sensors-under-sense; SEC-git-cleared-without-lazy-fetch;
  SEC-repository-commands-neutralized-or-refused; NUC-documents-are-fixed §2

## Blocked by

- None (can start immediately)

## Mode

with the human: a change of the specification is the human's.
