# ADR-0026: Tests build their repositories through the real git CLI in fresh temporary directories, under a cleared environment with a `HOME` of the test run's own

- **Status:** 🟢 accepted
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** every test that needs a git repository, the test helpers that build one, the environment the tests run under, and the `HOME` they pass to git and to `nuc`
- **Supersedes:** [ADR-0012](0012-fixture-repositories.md)

## Context

`nuc` reaches git only through its CLI, so that it behaves as git does
([ADR-0011](0011-git-cli-cleared-environment.md)); a test that mocks git would test the mock. The
template's self-tests showed what an inherited environment does to fixtures: a `GIT_DIR` leaked
from a hook into them, and they acted on the wrong repository. The tests of this project run from
the pre-push hook as well ([ADR-0002](0002-decisions-verified-by-tests.md)), so they meet the same
environment. Q-8 and Q-9 are decided by tests that observe what `nuc` writes and which repository
it acts on, which needs repositories a test owns entirely, and Q-4 asks the same suite to pass on
four platforms.

A cleared environment that passes the parent's `HOME` still hands every git a test runs, and every
git `nuc` runs under it, the global configuration of whoever runs the suite. A developer's global
`merge.conflictStyle` decides whether a merge writes a base section, a global `core.fsmonitor`
starts a monitor in a fixture, and a global `core.attributesFile` with an `eol` attribute changes
the bytes a fixture checks out. A key set in the repository's own configuration closes that key
only, and the next key a test depends on meets the same gap. ADR-0011 keeps the system and the
global configuration in effect for the git `nuc` runs, so that git behaves as it does for the user;
that governs `nuc`, not the environment a test gives it.

## Decision

We will have every test that needs a git repository build it through the real git CLI in a fresh
temporary directory of its own, with the test's processes under a cleared environment whose
`HOME` is an empty directory the test run owns.

- **No committed repositories:** no `.git` directory and no archive of one is checked in; a
  fixture is the sequence of git commands that builds it.
- **No mock of git:** a test that needs git runs git. A stand-in that records its environment and
  arguments is allowed only to test what the shell passes to git.
- **Cleared environment:** the helpers that run git or `nuc` in a test pass the allowlist the shell
  passes, `HOME` excepted, plus a fixed author, committer, and date, so that an inherited `GIT_*`
  variable or a user's identity changes nothing.
- **A `HOME` of the test run's own:** in place of the parent's, the helpers pass one empty
  directory per test process, under the system's temporary directory and outside any repository,
  to git and to `nuc` alike. A test that hands `nuc` the parent's environment on purpose, to show
  what the shell clears, sets this `HOME` over the parent's as well. So no global configuration
  reaches a fixture, through git or through the git `nuc` runs. No test writes into it; a test
  that needs a global configuration gives its child a `HOME` of its own.
- **The system configuration stays:** the configuration of the system git is installed with is
  read as before, and a fixture sets in its own configuration what the system's may change for
  it, `core.autocrlf` among them, as git for Windows sets it there.

**Out of scope:** the parity harness and its corpus, a decision of its own; the test framework
and crates; the configuration the git `nuc` runs reads outside the tests, which ADR-0011 governs;
a test that reads this project's own repository rather than a fixture, whose git runs as the
developer's does, `safe.directory` included.

## Alternatives considered

- **Committed fixture repositories or archives** — opaque in review, and a committed `.git` is
  repository content a fork could shape.
- **A mocked git** — fast, but it tests the mock, not git's behaviour, which the constraints chose
  the CLI to keep.
- **The tests' environment inherited** — the leak of `GIT_DIR` from a hook into the template's
  self-tests is the case this decision closes.
- **The parent's `HOME`, each key set in the fixture's own configuration** — closes a key only
  after a run has met it, on the machine of whoever met it.
- **`GIT_CONFIG_GLOBAL` set to the null device for the tests' git** — closes the global
  configuration for the git a test runs, but the shell clears every `GIT_*` variable, so the git
  `nuc` runs reads the parent's global configuration still.
- **A `HOME` per fixture** — keeps one fixture from seeing what another left in its `HOME`, which
  no test writes; it ties every call of `nuc` or git to a fixture, and a call outside one would
  need a way of its own.
- **The system configuration turned off as well** — `GIT_CONFIG_NOSYSTEM` would reach only the
  tests' git, since the shell clears it for the git `nuc` runs; and a developer changes the system
  configuration only as an administrator, while CI's images are fixed.

## Sources / Prior art

- [git(1), *Environment variables*](https://git-scm.com/docs/git#_environment_variables): the
  variables a hook inherits, `GIT_AUTHOR_DATE`, `GIT_COMMITTER_DATE`, `GIT_CONFIG_GLOBAL`, and
  `GIT_CONFIG_NOSYSTEM`.
- [git-config(1), *Files*](https://git-scm.com/docs/git-config#FILES): the global configuration
  read from `$HOME/.gitconfig` and `$XDG_CONFIG_HOME/git/config`, the latter defaulting under
  `HOME`, and the system configuration read from the installation's own file.
- The template's self-tests under `scripts/`, which build their fixture repositories the same way.

## Consequences

- Positive: the tests run alike from a shell, from a hook, and in CI, and on every developer's
  machine whatever their global configuration; a fixture reads as the git commands that built it.
- Negative / trade-offs: every repository test spawns git, which is slower than a mock, most on
  Windows. In the tests, the git `nuc` runs reads no global configuration, so a behaviour of
  `nuc` that depends on a user's global configuration, `safe.directory` in a container among it,
  is decided by no test; ADR-0011 keeps that configuration in effect outside them.
- Follow-ups: none.

## Enforcement

A test cited with `Verifies: ADR-0026` runs the
fixture helpers with `GIT_DIR`, `GIT_WORK_TREE`, and `GIT_CONFIG_PARAMETERS` set to a decoy and
checks that the decoy is untouched and the fixture is built where the helper said. Tests cited the
same way run the helpers in a child whose parent `HOME` holds a global configuration that sets
`merge.conflictStyle diff3` and a `core.attributesFile` with `* eol=crlf`, and check that a merge
through the merge driver's shim and the built `nuc` writes no base section and that a file of LF
lines checks out as committed; each fixture sets nothing of its own for the key under test, no
conflict style and no attributes, so each fails exactly when the global configuration reaches git or
`nuc`. Check 10 of [`scripts/check-docs.sh`](../../scripts/check-docs.sh) names every top-level
entry, so a committed fixture repository would have to be listed; that a test does not mock git is
review.
