# Ticket: ShellCheck is the same version in the Dev Container and in CI

## What to build

The shell job of CI and the Dev Container lint with one version of ShellCheck, so that a script
clean in the container is clean in CI. The Dev Container installs ShellCheck from the base
image's Debian archive (`postCreateCommand` of `devcontainer.json`); the shell job uses the one
the runner image ships. ShellCheck 0.11 on the runner reported SC2329 where the container's
version reported SC2317, and the shell job failed on `scripts/test-gh-host-login.sh` after it was
green locally (e81aead). Either both install one pinned version, or the shell job fails with the
version it found when it differs from the one the container pins, whichever the human chooses.

## Acceptance criteria

- [ ] Given the Dev Container and the shell job, when each prints `shellcheck --version`, then
      both name the same version.
- [ ] Given a change of that version, when it is made, then it is made in one place, or a check
      fails while the two differ.

## Read first

- Touches: `.devcontainer/devcontainer.json`, `.github/workflows/checks.yml` (the shell job)
- OPS-dev-container-adds-rust and SEC-actions-pinned-by-sha for how the container and the
  workflows pin what they install.

## Blocked by

- None (can start immediately)

## Mode

with the human — the choice between pinning and failing on a difference is the human's, and the
Dev Container's configuration is kept with the human by the harness ADR.
