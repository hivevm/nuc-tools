# Spec: Take up template release v0.11.0

## Problem

This repository carries template release v0.8.1. The template has since moved through v0.9.0,
v0.9.1, v0.10.0, and v0.11.0. The repository's own checks, hooks, skills, and ADR record are still
those of v0.8.1, so a contributor here works to rules the template no longer ships. A finding about
the template written from here names files and checks the template has replaced. The open ticket
on ADR wording waits for the migration of the record to v0.9.0.

## Solution

A contributor works this repository under the rules of v0.11.0: the fixed base of `NUC-` ADRs,
ADRs named by category and slug, the checks keyed by rule, scripts that take `--root`, and the
linear integration of `feat/` branches. The `Template release` line names v0.11.0.

## Existing behaviour

- The record holds ADR-0001 to ADR-0006 inherited from the template and ADR-0007 to ADR-0028 of
  the project. Code comments, tests, and documents cite them by number.
- The repository checks its own tree with the bash checks and hooks of v0.8.1, run by the check
  runner and CI. `nuc` does not check this tree: there is no `nuc.toml`, and the hooks under
  `.githooks/` are the template's bash. So `nuc` need not know v0.9 or later for the line to move.
- What must stay: every check the project added (build, dependencies, gates, workflows, parity,
  release) stays green, and every test of `nuc` passes unchanged. Messages of `nuc` that copy a
  bash message of an old release keep the old ADR number it cites. So do the ADR files of the
  parity corpus and the fixtures. The release take-up changes the record and the tooling around
  `nuc`, never what `nuc` decides.
- The check runner and the self-tests of each taken-up script pin the behaviour. The project's own
  scripts gain `--root` handling under the new rule of the shell check, which is what pins them.
- Callers move with the scripts: the workflows, hooks, and the project's scripts that pass a
  positional root or read a check by number.

## Goals served

The take-up serves no criterion of the specification directly. It keeps the repository on the
rules of the template it ports.

- G-4 is advanced indirectly: the take-up is a worked example of what `nuc upgrade` is to
  automate, and its hand-offs record which steps were mechanical and which needed judgement.

## User stories

1. As a contributor, I want the repository's rules to be those of the current template release,
   so that a finding about the template written from here is about the template as it is.
2. As a reviewer, I want each release taken up in its own pull request, green on its own, so that
   the migration of the record is reviewed apart from the routine take-ups.
3. As the human who decides, I want each clause the project added to an inherited ADR proposed as
   an ADR of its own before anything is migrated, so that no addition is lost in the migration.
4. As a maintainer of `nuc`, I want the messages and fixtures that copy an old release to keep
   their old ADR numbers, so that parity with that release still holds.
5. As a contributor, I want the open ticket on ADR wording unblocked, so that the next
   normalization can carry its fixes.

## Implementation decisions

- One pull request per release, in order: v0.9.1 with the migration of the numbered record, then
  v0.10.0, then v0.11.0. Each moves the `Template release` line and is green on its own.
- The migration follows section 8 of the normalize skill of v0.9.1, taken from the template at that
  tag. Its first step diffs the inherited ADRs and `AGENTS.md` against v0.8.1. That diff is lost
  once the release's files are in, so it runs before any file is replaced.
- Each clause the project added to an inherited ADR becomes an ADR of the project that extends or
  narrows a `NUC-` clause, proposed and decided before the migration. Among them is the clause of
  ADR-0005 that keeps the template's numbers for `nuc`'s messages and the parity corpus. It is
  restated for named ADRs: the copies of old releases keep the numbers they copy.
- The project's ADR-0007 to ADR-0028 receive category identifiers. The human reviews the map
  before it is applied. The project lists its categories beyond the fixed ones in the index.
- Each take-up leaves out what the template's notes leave out: `docs/plan/` and `docs/backlog/` of
  the template, the setup-project skill, the template setup test, and the issue template for
  template findings.
- A maintainer walks the repository settings against what changed in the contributing guide's
  section before each pull request merges. The settings sensor runs on the maintainer's
  instruction.

## Testing decisions

- The check runner, with the self-tests of the template's scripts at each release, is the seam.
  No new test is written for a file taken up unchanged.
- The project's own scripts are held to the new shell rules (`--root`, bash floor, mode) by the
  shell check of v0.10.0.
- The parity harness and the Rust suite pass unchanged across all three take-ups. A diff under
  `tests/parity/` outside a comment line, or a changed expected message of `nuc`, means the
  migration renamed a copy of an old release.

## Out of scope

- Porting the rules of v0.9.0 and later into `nuc`, and adding those releases to the parity
  corpus. That is its own spec, recorded in the backlog.
- The four wording fixes of the open revision ticket. They follow the migration in their own
  normalization.
- A normalization of the project's own ADRs beyond the identifiers. The migration renames and
  changes no decision.
- Changes to this repository's telemetry stack. v0.11.0's "Measuring a session" is reconciled
  with it in the guide only.

## ADR impact

- Binding: ADR-0001 to ADR-0006 as inherited. ADR-0005 governs the release line and the take-up by
  normalization. Its clause keeping the numbers 0001 to 0006 is exactly what the migration ends.
  ADR-0014 and ADR-0023 govern how `nuc` reads the release line, and are untouched because `nuc`
  does not check this tree.
- New `proposed` ADRs before implementation: one for each clause the project added to an inherited
  ADR, each extending or narrowing a `NUC-` clause. Ticket 01 finds and proposes them.
- ADRs of the project that name a check of `check-docs.sh` by number (ADR-0008, ADR-0014,
  ADR-0015, ADR-0023) bind the whole file after v0.10.0. Each is superseded by one naming the rule
  key where the narrow binding matters. Ticket 03 decides which.
