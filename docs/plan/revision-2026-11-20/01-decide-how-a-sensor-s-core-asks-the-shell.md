# Ticket: Decide how a sensor's core asks the shell

## What to build

API-git-only-sensors-under-sense decides in its *Core* clause that each sensor is a pure function
of the library outside `src/shell/`. Under it, a sensor takes the values the shell gathered and
returns its report or a refusal. Two sensors do not take only gathered values:

- **review-binding:** its core in `src/sense/review_binding.rs` asks its questions through the
  trait `review_binding::Repo`, thirteen queries of git and the hand-off.
  `shell::sense::ReviewRepo` answers each with one git call.
- **binding-adrs:** `binding_adrs::report` and `bind` take the callback `diffed`. The core calls it
  for each path where the bash reads one, and the shell runs a git diff for each call.

Both keep the letter of the clause's last sentence: the core reads no file and runs no process,
and `tests/functional_core.rs` holds that. Both break "takes the values the shell gathered", since
the core picks which git call runs next. No test decides that part of the clause, and no ADR
decides the shape. `docs/ARCHITECTURE.md` describes both.

The evidence is that the hand-off of ticket 07 of `sensors-under-nuc-sense` left the question to the
maintainer under *Unsure*, and both of its reviews left it open. The first review weighed the other
side as well. The unit tests of `src/sense/review_binding/tests.rs` record the questions the core
asks a fake `Repo`, and decide their order. The reviewer found no simpler shape that reads the
hand-off and the merge messages only where the bash reads them. binding-adrs set the precedent.

The maintainer chooses one of these:

- **Gather, then decide:** the shell drives the bash's order of questions and hands the core the
  answers. The cores stay pure. The parity harness still holds the result, but the unit tests of
  the order of the questions are lost.
- **Keep the shape:** an ADR superseding API-git-only-sensors-under-sense lets a sensor's core ask
  the shell through a trait or a callback that only the shell implements. The ADR names when a
  sensor may do so, and what holds the shell's answers to git alone.

## Acceptance criteria

- [ ] Given the maintainer's choice, when the session lands it, then either the cores of
  review-binding and binding-adrs take only values and their parity rows of
  `tests/parity/sensors.tsv` pass unchanged, or a `proposed` ADR supersedes
  API-git-only-sensors-under-sense and names both the trait and the callback.
- [ ] Given the change, when `docs/ARCHITECTURE.md` describes the Sensors, then it says what the
  choice made, and nothing that the record does not decide.

## Read first

- Touches: `src/sense/review_binding.rs`, `src/sense/review_binding/tests.rs`,
  `src/sense/binding_adrs.rs`, `src/sense/binding_adrs/tests.rs`, `src/shell/sense.rs`,
  `docs/adr/`, `docs/ARCHITECTURE.md`
- API-git-only-sensors-under-sense (*Core*); ARCH-rules-outcome-hooks-answer; the convention
  "The shell reads, the rules decide" in `docs/CONVENTIONS.md`

## Blocked by

- None (can start immediately)

## Mode

with the human: the choice moves an accepted ADR or the shape of the core, and
DEV-one-agent-one-ticket allows no unattended work.
