# Ticket: Test that nuc check runs no repository content and reaches no network

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The port's rules read the workflows, the scripts, the skills, and every text file, which is the
repository content Q-7 asks `nuc` never to run, and Q-11 asks every check to report the same with
the network blocked. Neither has a test. Write the fixtures of Q-7 for `nuc check`: a tree whose
scripts, workflows, hooks, `core.fsmonitor`, and git attributes would create a canary file if run,
checked with every rule turned on. Write the test of Q-11 where CI can block the network, or say
in the hand-off why it cannot and what stands in for it.

## Acceptance criteria

- [ ] Given a fixture carrying content that would create a canary file if run, when `nuc check`
  runs on it with every rule turned on, then the canary file does not exist, and the test cites
  Q-7.
- [ ] Given the network blocked for the process, when `nuc check` runs on a fixture, then it
  reports what it reports with the network, and the test cites Q-11; or the hand-off names why
  the platform forbids it.

## Read first

- ADR-0010 (the neutralized keys), ADR-0011, ADR-0002.
- `docs/SPECIFICATION.md`, Q-7 and Q-11; the test pattern in `docs/ARCHITECTURE.md`.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 keeps every ticket with the human, and the test touches the shell's
spawn of git.
