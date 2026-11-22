# Ticket: `nuc upgrade` takes up every other file by the three-way rule and lists the rest

## What to build

`nuc upgrade B` decides every path outside the base by its blob in A, B, and `HEAD`, as the spec
says: it replaces, adds, and deletes what the project never changed, and lists every other case
as a judgment, a finding line under an `upgrade/` rule. A run with a judgment writes its plan and
exits 1. A symlink or a submodule on either side is a judgment. The path guards refuse, before the
first write, a path through a symlink of the working tree. `nuc explain` explains each new rule.

Ticket 02 built the base of this: the reasons `link`, `added`, and `mode` for the base paths, the
settling of the plan against the working tree (a symlink above a path refuses, a path the working
tree keeps of its own is `added`), the guards of `verify_path`, and the case collisions. This
ticket runs them over every other path. A directory the working tree keeps at a path to write is
`added` without regard to the plan's deletions so far; the swap of a file and a directory makes
that "after the plan's deletions", as the ADR says. Once every path is planned, the isolation test
of a template tree with a `.git` entry expects the refusal `path` rather than a run that leaves
`.git/config` alone.

## Acceptance criteria

- [ ] Given a path the template changed and the project did not, when `nuc upgrade B` runs, then
      the file holds B's blob with B's mode.
- [ ] Given a path B adds and the project lacks, and a path B removes and the project carries as A
      had it, when it runs, then the first is added and the second deleted.
- [ ] Given a path both changed, a path the project deleted and B changed, and a path B adds where
      the project has its own, when it runs, then each is a finding line with its key, the file
      keeps the project's bytes, and the exit is 1.
- [ ] Given a symlink or a submodule in A, B, or `HEAD` at a path the template changed, when it
      runs, then it is a judgment and nothing is written there.
- [ ] Given a template that replaces a file with a directory of the same name, and one that
      replaces a directory with a file, when it runs, then both are taken up.
- [ ] Given each new rule ID, when `nuc explain <id>` runs, then it prints the reason, and the rule
      IDs and keys stand in `tests/contract.rule-ids` and `tests/contract.rule-keys`.

## Read first

- Touches: `src/upgrade.rs`, `src/shell/upgrade.rs`, `src/finding.rs`, `tests/contract.rs`,
  `tests/contract.rule-ids`, `tests/contract.rule-keys`, `tests/isolation.rs`
- The ADR of ticket 01; the spec, Implementation decisions (Every other path, Writing).
- Golden path: the plan function and its shell of ticket 02.
- Seam: the plan function on values for the cases; the binary for the writes and the guards.

## Blocked by

- docs/plan/nuc-upgrade/02-tracer-base-and-release-line.md

## Mode

with the human: it widens what the shell writes, an effect DEV-one-agent-one-ticket keeps with the
maintainer.
