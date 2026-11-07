# Ticket: Run every fixture's git and `nuc` without the user's git configuration

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Three findings of the counted changes were a developer's or a platform's git configuration
reaching a fixture: the merge driver's conflict-style test passed or failed by the developer's
global `merge.conflictStyle` (#58, should-fix); a global `core.fsmonitor` could start a monitor
in a fixture (#64, should-fix); and a user's `core.attributesFile` with an `eol` attribute still
reaches one (#57, left under *Doubts*). Each was closed, or left, one key at a time, by a setting
in the repository's own configuration, and the next key a test depends on meets it again. The
helpers pass `HOME` from the parent, as the shell passes it (ADR-0012, *Cleared environment*), so
every git a test runs, and every git `nuc` runs under it, reads the global configuration of
whoever runs the suite.

Have `cleared` in `tests/support/` set `HOME` to an empty directory the test run owns, outside
any repository, instead of passing the parent's, so that no global configuration reaches a
fixture through git or through `nuc`. The system's configuration stays, as git for Windows sets
`core.autocrlf` there; the settings `Fixture::new` writes into the repository's own
configuration stay for it. `git_in` passes `core.fsmonitor`, `maintenance.auto`, and `gc.auto`
on its command line, and `Fixture::new` writes the same three into the repository: keep them in
one place. A test like those of `tests/fixtures.rs`, run as a child with a parent `HOME` whose
configuration sets a key that changes a fixture, shows that none reaches it.

The change touches only tests: no ADR is needed
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 2). Where the session finds that a fixed
`HOME` departs from ADR-0012's "the same allowlist the shell passes" rather than implementing it,
it stops and hands off with a draft ADR instead.

## Acceptance criteria

- [ ] Given a parent `HOME` whose global configuration sets `merge.conflictStyle diff3` and
  `core.autocrlf true`, when a child builds a fixture and runs a git command and `nuc` in it,
  then neither setting reaches the fixture.
- [ ] Given the helpers, when the source of `tests/support/` is read, then the settings that keep
  background work and line endings out of a fixture stand in one place.
- [ ] Given the suite, when it runs on the four platforms of Q-4 in CI, then it passes unchanged.

## Read first

- Touches: `tests/support/mod.rs`, `tests/fixtures.rs`
- ADR-0012, *Decision* and *Enforcement*; ADR-0011, *Consequences*, on the user's global
  configuration as trusted.
- The test pattern in `docs/ARCHITECTURE.md`.
- The *Doubts* and *Review* of #57, #58, and #64.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket; the proof on Windows and macOS is the
portability check run, which only a push shows.
