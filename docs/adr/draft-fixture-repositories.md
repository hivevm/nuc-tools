# ADR-draft-fixture-repositories: Tests build their repositories through the real git CLI in fresh temporary directories, under a cleared environment

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** every test that needs a git repository, the test helpers that build one, and the environment the tests run under

## Context

`nuc` reaches git only through its CLI, so that it behaves as git does
([ADR-draft-git-cli-cleared-environment](draft-git-cli-cleared-environment.md)); a test that
mocks git would test the mock. The template's self-tests showed what an inherited environment
does to fixtures: a `GIT_DIR` leaked from a hook into them, and they acted on the wrong
repository (*Problem*). The tests of this project run from the pre-push hook as well
([ADR-0002](0002-decisions-verified-by-tests.md)), so they meet the same environment. Q-8 and
Q-9 are decided by tests that observe what `nuc` writes and which repository it acts on, which
needs repositories a test owns entirely.

## Decision

We will have every test that needs a git repository build it through the real git CLI in a fresh
temporary directory of its own, with the test's processes under a cleared environment.

- **No committed repositories:** no `.git` directory and no archive of one is checked in; a
  fixture is the sequence of git commands that builds it.
- **No mock of git:** a test that needs git runs git. A stand-in that records its environment and
  arguments is allowed only to test what the shell passes to git.
- **Cleared environment:** the helpers that run git or `nuc` in a test pass the same allowlist the
  shell passes, plus a fixed author, committer, and date, so that an inherited `GIT_*` variable
  or a user's identity changes nothing.

**Out of scope:** the parity harness and its corpus, a decision of its own; the test framework
and crates.

## Alternatives considered

- **Committed fixture repositories or archives** — opaque in review, and a committed `.git` is
  repository content a fork could shape.
- **A mocked git** — fast, but it tests the mock, not git's behaviour, which the constraints chose
  the CLI to keep.
- **The tests' environment inherited** — the leak of `GIT_DIR` from a hook into the template's
  self-tests is the case this decision closes.

## Sources / Prior art

- [git(1), *Environment variables*](https://git-scm.com/docs/git#_environment_variables): the
  variables a hook inherits, and `GIT_AUTHOR_DATE`, `GIT_COMMITTER_DATE`.
- The template's self-tests under `scripts/`, which build their fixture repositories the same way.

## Consequences

- Positive: the tests run alike from a shell, from a hook, and in CI; a fixture reads as the git
  commands that built it.
- Negative / trade-offs: every repository test spawns git, which is slower than a mock, most on
  Windows.
- Follow-ups: shared helpers for the fixtures that many rules need.

## Enforcement

A test cited with `Verifies: ADR-draft-fixture-repositories` runs the fixture helpers with
`GIT_DIR`, `GIT_WORK_TREE`, and `GIT_CONFIG_PARAMETERS` set to a decoy and checks that the decoy
is untouched and the fixture is built where the helper said. Check 10 of
[`scripts/check-docs.sh`](../../scripts/check-docs.sh) names every top-level entry, so a committed
fixture repository would have to be listed; that a test does not mock git is review.
