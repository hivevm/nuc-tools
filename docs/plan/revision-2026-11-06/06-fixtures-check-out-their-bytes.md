# Ticket: Build every fixture repository to check out the bytes it commits

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Three fixes of the hooks' port were Windows behaviour no local run could see, each found by the
portability check runs after the push: git for Windows hands its hooks its own bash (24c719b), a
ref name of no UTF-8 that its file system refuses (8f2d052), and the line endings a Windows
checkout gives the merged index (1f7c0e4). The last is a fixture's, not the system's: git for
Windows sets `core.autocrlf` in its system configuration, which the cleared environment of
ADR-0012 does not reach, and the fixtures pass `HOME`, so a user's global configuration reaches
them too. A fixture that compares checked-out bytes then compares what the platform or the user
made of them. The fix set `core.autocrlf false` in the one fixture of the driver; the next test
that compares bytes on checkout meets it again.

Have the helpers of `tests/support/` set it in the configuration of every fixture repository they
build, where neither the system's nor the user's can override it, and drop the line from the
driver's fixture. The identity the fixtures commit with comes from the variables the helpers set,
and needs nothing of this.

## Acceptance criteria

- [ ] Given a fixture built by `tests/support/`, when its configuration is read, then
  `core.autocrlf` is `false` in the repository's own.
- [ ] Given the driver's test, without its own setting, when it runs on Windows in CI, then it
  passes.

## Read first

- Touches: `tests/support/mod.rs`, `tests/hooks.rs`
- ADR-0012; the test pattern in `docs/ARCHITECTURE.md`.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket; the proof is the Windows check run, which
only a push shows.
