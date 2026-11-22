# Ticket: The fixtures of nuc upgrade hold what their release holds

## What to build

`tests/support/upgrade.rs` builds the template releases of the tests of `nuc upgrade`.
`two_releases` names its first release v0.9.0 and has it ship a `nuc.toml`. The template's tag
v0.9.0 ships none, and only v0.12.0 and later do. API-upgrade-takes-up-by-blob rested on that
belief, and so did the fixtures that held it. It was superseded on the day it was accepted, by
API-upgrade-by-blob-toml-optional, once the replay of this repository's take-ups refused. That is
the evidence, a rework (commit cfa01b6, "feat(upgrade): nuc upgrade judges a nuc.toml only where
there is one").

The convention "A fixture named for a template release holds what its tag holds" in
`docs/CONVENTIONS.md` now asks for the opposite of what the fixtures do. The fixtures either name
releases that ship what they carry, such as v0.12.0 and later where a release has a `nuc.toml`,
or drop the files their release lacks. The test of a project and a template without a
`nuc.toml` then needs no fixture of its own.

## Acceptance criteria

- [ ] Given each release a fixture of `tests/support/upgrade.rs` names, when its files are set
  against the template's tag of that name, then the fixture holds no kind of file the tag lacks:
  no `nuc.toml` before v0.12.0.
- [ ] Given the change, when the contract and isolation tests of `nuc upgrade` run, then each
  still decides the case it names, and the hand-off gives the reason for each test it changes.

## Read first

- Touches: `tests/support/upgrade.rs`, `tests/contract.rs`, `tests/isolation.rs`
- The convention "A fixture named for a template release holds what its tag holds" in
  `docs/CONVENTIONS.md`; API-upgrade-by-blob-toml-optional (*Context*, *Enforcement*)

## Blocked by

- None (can start immediately)

## Mode

with the human: DEV-one-agent-one-ticket allows no unattended work, and the change rewrites
existing tests, each with its reason.
