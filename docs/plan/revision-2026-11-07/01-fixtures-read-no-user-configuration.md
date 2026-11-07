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
configuration stay for it. `git_in` passes `core.fsmonitor=false`, `maintenance.auto=false`,
and `gc.auto=0` on its command line, and `Fixture::new` writes the same three keys into the
repository's own configuration: keep them in one place. A test like those of
`tests/fixtures.rs`, run as a child with a parent `HOME` whose configuration sets keys no fixture
sets locally, shows that none reaches it.

This amends a decision. ADR-0012's *Cleared environment* passes "the same allowlist the shell
passes, plus a fixed author, committer, and date"; a `HOME` fixed by the tests is not on it. And
ADR-0011's *Configuration* keeps the system and global configuration in effect for the git `nuc`
runs, `safe.directory` in a container among it, which a fixed `HOME` takes from that git in the
tests. So the first step is a draft ADR through `propose-adr`, a restatement of ADR-0012's clause
that names the fixed `HOME` and what it costs, handed to the human in a commit of ADRs only
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rules 3 and 4). No code is written before the
human has reviewed it; one the human sends back ends the ticket there.

## Acceptance criteria

- [ ] Given the draft ADR, when the human has reviewed it, then the implementation follows it on
  the same branch, or the ticket is recorded as declined with the reason.
- [ ] Given a parent `HOME` whose global configuration sets `merge.conflictStyle diff3`, when a
  child merges two sides that both changed a rule of the ADR index through the merge driver's
  shim and the built `nuc`, then the conflict markers carry no base section (`|||||||`), so the
  setting reached neither git nor `nuc`.
- [ ] Given a parent `HOME` whose global configuration names a `core.attributesFile` that sets
  `* eol=crlf`, when a child commits a file of LF lines in a fixture and checks it out again, then
  its bytes are those committed, with no CR.
- [ ] Given the helpers, when the source of `tests/support/` is read, then `core.fsmonitor`,
  `maintenance.auto`, and `gc.auto` are set for a fixture in one place.
- [ ] Given the suite, when it runs on the four platforms of Q-4 in CI, then it passes unchanged.

## Read first

- Touches: `docs/adr/`, `tests/support/mod.rs`, `tests/fixtures.rs`, `tests/hooks.rs`
- ADR-0012, *Decision* and *Enforcement*; ADR-0011, *Configuration* and *Consequences*, on the
  user's global configuration as trusted and `safe.directory`.
- The test pattern in `docs/ARCHITECTURE.md`.
- The *Doubts* and *Review* of #57, #58, and #64.

## Blocked by

- None (can start immediately)

## Mode

with the human — it starts with an ADR the human decides, ADR-0007 allows no unattended ticket,
and the proof on Windows and macOS is the portability check run, which only a push shows.
