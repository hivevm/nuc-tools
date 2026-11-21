# Ticket: an ADR decides how `nuc upgrade` takes up a template release

## What to build

A `proposed` ADR, by the `propose-adr` skill, that decides what the spec of this directory leaves
to one: how `nuc upgrade <tag>` decides a take-up and how it writes it. The maintainer accepts it
before ticket 02 starts.

It decides at least:

- the inputs: the release line as the old release, the argument as the new, both read from
  `refs/template/<tag>`, no network;
- the base replaced outright and every other path by the three-way rule by blob, symlinks and
  submodules always a judgment;
- the release line edited in place; citations of a `NUC-` identifier the new release lacks, and
  the releases between the two tags, listed as judgments;
- the writing: the blob's bytes with no filter run, regular files only, the path guards, every
  refusal passed before the first write;
- the output and exit codes, the `upgrade/` rule IDs and their keys, and whether that supersedes
  API-command-line-with-sense or extends it;
- the refusals the spec lists.

The ADR names its alternatives at least: a manifest in the template, migrations coded in `nuc`,
`nuc` fetching the tag itself, a dry run by default, `nuc` committing the take-up.

## Acceptance criteria

- [ ] Given the spec of `docs/plan/nuc-upgrade/`, when the ADR is submitted, then it stands under
      `docs/adr/` as `proposed`, listed in the index, and `scripts/check-all.sh` passes.
- [ ] Given the ADR, when the review of the `propose-adr` skill has run, then its findings are
      settled and the maintainer has said "<ID> ist angenommen" before its status flips.
- [ ] Given an ADR that changes the command contract, when it is accepted, then the ADR it
      supersedes is marked so in the same change.

## Read first

- Touches: `docs/adr/`, `docs/SPECIFICATION.md` (G-4, if its wording sharpens)
- The spec of this directory, all of it.
- `docs/adr/API-command-line-with-sense.md`, `docs/adr/ARCH-rules-outcome-hooks-answer.md`,
  `docs/adr/API-version-coupling-hook-defaults.md` (Consequences: its follow-up on `nuc upgrade`),
  `docs/adr/SEC-repository-commands-neutralized-or-refused.md`.
- NUC-releases-are-semver-tags §5, §6, and §7 and section 7 of the normalize skill: the
  take-up by hand this command does.
- Golden path: `docs/adr/API-git-only-sensors-under-sense.md`, the last ADR that added a command.

## Blocked by

- None (can start immediately)

## Mode

with the human: an ADR is decided by the maintainer, and its acceptance is the maintainer's word
alone.
