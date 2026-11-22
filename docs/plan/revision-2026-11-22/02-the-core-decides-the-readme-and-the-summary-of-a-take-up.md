# Ticket: The core decides the README and the summary of a take-up

## What to build

`shell::upgrade::run` decides two things itself that ARCH-rules-outcome-hooks-answer gives the
core:

- which README the release line is moved in: the project's, the new release's blob where the plan
  writes `README.md`, or a refusal where the plan deletes it;
- what the summary counts: the paths written, the README counted once where only its line moved,
  the paths deleted, and the paths listed, several findings at one path counted once.

The shell is to gather values, call the core, and carry out what it returns. Code in the shell is
tested only through the binary, and these branches have no such test.

The evidence is the review of `nuc-upgrade`. Ticket 03's second round left two branches untested:
the plan deleting `README.md`, and the line moved into the new release's bytes. Ticket 04's first
round found the summary counting findings, not paths, and its second round left the new count
untested where the two differ.

A pure function of `src/upgrade.rs` takes the plan, the README's bytes, and the blobs, and returns
the README to write or a refusal, and the counts of the summary. The shell prints and writes what it
returns. Each branch gets a unit test on values.

## Acceptance criteria

- [ ] Given a plan that deletes `README.md`, when the core decides the README, then it refuses with
  `release`, and a unit test holds it.
- [ ] Given a plan that writes `README.md` from the new release, when the core decides the README,
  then the line is moved in the new release's bytes, and a unit test holds it.
- [ ] Given two findings at one path, or a README whose line alone moved, when the core counts the
  summary, then the path counts once, and a unit test holds each.
- [ ] Given the change, when `src/shell/upgrade.rs` is read, then `run` holds no branch on what the
  plan does to `README.md` and no count, and the contract tests of `nuc upgrade` pass unchanged.

## Read first

- Touches: `src/upgrade.rs`, `src/shell/upgrade.rs`
- ARCH-rules-outcome-hooks-answer (*Shell*); API-upgrade-by-blob-toml-optional (*The release line*,
  *Output*); the convention "The shell reads, the rules decide" in `docs/CONVENTIONS.md`

## Blocked by

- docs/plan/revision-2026-11-22/01-the-release-line-is-found-by-one-reading.md

## Mode

with the human: DEV-one-agent-one-ticket keeps the code with effects with the maintainer.
