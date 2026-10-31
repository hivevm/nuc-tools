# Ticket: Give the shell's reading of the tree a module of its own

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`src/shell/mod.rs` has grown to about 770 lines: the command line, the run of `nuc check`, and
`Tree` with a reader for each input. The test that a path resolves inside the repository is
written four times, in `read_bytes`, `read_adr_targets`, `list`, and `glob_matches`, and it is the
test three reviews of the port found missing. Move `Tree` and its readers into a module of
`src/shell/` of their own, with that test in one function every reader calls. ADR-0009 names
this split as its follow-up; it changes no behaviour.

## Acceptance criteria

- [ ] Given the suite as it stands, when the readers have moved, then every existing test passes
  unchanged and `nuc check` reports the same findings and refusals on the corpus.
- [ ] Given the module, when a reader resolves a path, then it does so through the one function
  that refuses a path outside the repository, and no other function of the shell canonicalizes a
  path to test it.
- [ ] Given `docs/ARCHITECTURE.md`, when the module lands, then the Shell block names it.

## Read first

- ADR-0009 (the shape and its follow-up), ADR-0010.
- The conventions on the shell reading for the rules.
- `tests/check.rs`, the cases whose names say "outside the repository".

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time, and the shell is
the code the threats aim at.
