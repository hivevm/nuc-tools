# ADR-0010: The shell spawns the git CLI with a cleared environment, an allowlist passed through, the repository named by `-C`, and command-running configuration neutralized

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** every spawn of git in `src/shell/`, the environment and arguments it passes, and the list of neutralized configuration keys kept beside its test

## Context

The constraints make the git CLI the only runtime dependency, spawned rather than linked through
libgit2, so that `nuc` behaves exactly as git does (*Constraints*). git, when it runs a hook,
sets variables such as `GIT_DIR` and `GIT_CONFIG_PARAMETERS` that point at a repository and a
configuration; inherited into `nuc` and the git it spawns, they can make it act on another
repository, as the leak of `GIT_DIR` into the template's self-tests did (*Problem*, *Threats*).
The environments `nuc` runs in, CI's among them, carry tokens (*Threats*). The threats ask that
`nuc` act only on the repository it was pointed at (Q-9), disclose no token (Q-10), and execute no
repository content (Q-7). The shape keeps every spawn in `src/shell/`
([ADR-0009](0009-functional-core.md)).

Some git configuration keys make git run a command, such as `core.fsmonitor` and the hooks under
`core.hooksPath`. A container needs the user's global configuration for `safe.directory`.

## Decision

We will spawn git from `src/shell/` only with a cleared environment into which an allowlist is
passed, with the repository named by `git -C <path>` and the configuration keys that run commands
neutralized by `-c` on every call.

- **Allowlist:** `PATH` and `HOME`, and on Windows `SystemRoot`, passed from the parent; set by
  `nuc`: `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, and `GIT_OPTIONAL_LOCKS=0`. Nothing else, no
  variable of the parent starting with `GIT_` among them.
- **Repository:** determined from the directory `nuc` runs in, or from an explicit `--repo`, never
  from the environment, and passed to every call as `-C <path>`.
- **Neutralized keys:** `core.fsmonitor=false`, and `core.hooksPath` set to the null device
  (`/dev/null`, on Windows `NUL`), which can hold no hook and needs nothing written; a key that
  makes a git command `nuc` uses run another command joins the list, kept beside the
  test, in the change that starts using that command.
- **Configuration:** the system and global configuration stay in effect, so that git behaves as
  it does for the user.

**Out of scope:** which git commands `nuc` runs; the minimum git version; processes other than
git, which `nuc` does not spawn.

## Alternatives considered

- **No system and global configuration** (`GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL` emptied) —
  safer against a hostile configuration, but it drops `safe.directory`, which a container needs,
  and makes git behave unlike the user's, against the reason for the CLI and likely against
  parity (Q-2).
- **Only the `GIT_*` variables removed, the rest inherited** — tokens such as `GITHUB_TOKEN` would
  reach git and whatever it starts, against Q-10.
- **libgit2 or gitoxide** — excluded by the constraints: the behaviour would be a library's, not
  git's.

## Sources / Prior art

- [git(1), *Environment variables*](https://git-scm.com/docs/git#_environment_variables):
  `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_CONFIG_PARAMETERS`, `GIT_OPTIONAL_LOCKS`,
  `GIT_TERMINAL_PROMPT`, and `-C`.
- [githooks(5)](https://git-scm.com/docs/githooks): the variables git sets for a hook.
- [git-config(1)](https://git-scm.com/docs/git-config): `core.fsmonitor`, `core.hooksPath`,
  `safe.directory`.

## Consequences

- Positive: an inherited `GIT_DIR` or `GIT_CONFIG_PARAMETERS` cannot redirect `nuc` (Q-9); no
  token of the parent reaches git (Q-10); the hooks and the file-system monitor of the target
  repository do not run (Q-7).
- Negative / trade-offs: a key the list does not yet name can still run a command until it is
  added; the user's global configuration is trusted, and a hostile one is out of reach of this
  decision; on a platform that needs another variable to start a process, the allowlist grows.
- Follow-ups: the minimum git version; the commands `nuc` runs and the keys each adds to the list.

## Enforcement

The test of Q-9 runs every command with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and
`GIT_CONFIG_PARAMETERS` set to a decoy repository and checks that it reports on the repository it
was pointed at and leaves the decoy untouched; it cites this ADR with
`Verifies: ADR-0010`. A test of the shell spawns a git stand-in that
records its environment and arguments, and fails on a variable outside the allowlist, on a call
without `-C`, and on a call without each key of the list.
