# SEC-git-cleared-environment: The shell spawns the git CLI with a cleared environment, an allowlist passed through, the repository named by `-C`, and command-running configuration neutralized

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** every spawn of git in `src/shell/`, the environment and arguments it passes, and the list of neutralized configuration keys kept beside its test

## Context

The constraints make the git CLI the only runtime dependency. `nuc` spawns it rather than link
libgit2, so that it behaves exactly as git does (*Constraints*). When git runs a hook, it sets
variables such as `GIT_DIR` and `GIT_CONFIG_PARAMETERS`. They point at a repository and a
configuration. Inherited into `nuc` and the git it spawns, they can make it act on another
repository. The leak of `GIT_DIR` from a git hook into the template's self-tests did that
(*Problem*, *Threats*). The environments `nuc` runs in, CI's among them, carry tokens (*Threats*).
The threats ask that `nuc` act only on the repository it was pointed at (Q-9), disclose no token
(Q-10), and execute no repository content (Q-7). The shape keeps every effect, every spawn of git
among them, in `src/shell/` ([ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md)).

Q-9 asks more of the hooks than of the checks. A hook that would decide on the git state does not
decide where git hands it a git directory other than the one it finds. A reference transaction that
touches no `main` passes as it would without the decoy. The merge driver merges the files git names
it in the repository git runs it in. `nuc` takes the repository from the directory it runs in, so a
hook handed `GIT_DIR` by `git --git-dir` or a linked worktree has to compare the two directories
before it decides (Q-1).

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
  (`/dev/null`, on Windows `NUL`), which can hold no hook and needs nothing written. A key that
  makes a git command `nuc` uses run another command joins the list, kept beside the test, in the
  change that starts using that command.
- **Configuration:** the system and global configuration stay in effect, so that git behaves as
  it does for the user.

**Out of scope:** which git commands `nuc` runs; the minimum git version; processes other than
git, which `nuc` does not spawn.

## Alternatives considered

- **No system and global configuration** (`GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL` emptied) —
  safer against a hostile configuration. But it drops `safe.directory`, which a container needs.
  It also makes git behave unlike the user's, against the reason for the CLI and likely against
  parity (Q-2).
- **Only the `GIT_*` variables removed, the rest inherited** — tokens such as `GITHUB_TOKEN` would
  reach git and whatever it starts, against Q-10.
- **libgit2 or gitoxide** — excluded by the constraints: the behaviour would be a library's, not
  git's. The `bans` of `deny.toml` refuse such crates.

## Sources / Prior art

- [git(1), *Environment variables*](https://git-scm.com/docs/git#_environment_variables):
  `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_CONFIG_PARAMETERS`, `GIT_OPTIONAL_LOCKS`,
  `GIT_TERMINAL_PROMPT`, and `-C`.
- [githooks(5)](https://git-scm.com/docs/githooks): the variables git sets for a hook.
- [git-config(1)](https://git-scm.com/docs/git-config): `core.fsmonitor`, `core.hooksPath`,
  `safe.directory`.

## Consequences

- Positive: an inherited `GIT_DIR` or `GIT_CONFIG_PARAMETERS` cannot redirect `nuc` (Q-9). No
  token of the parent reaches git (Q-10). The hooks and the file-system monitor of the target
  repository do not run (Q-7). A setting handed on in `GIT_CONFIG_PARAMETERS`, as `git -c` does,
  does not reach the git `nuc` runs; the merge driver takes its conflict style from the
  configuration, never from the environment.
- Negative / trade-offs: a key the list does not yet name can still run a command until it is
  added. The user's global configuration is trusted, and a hostile one is out of reach of this
  decision. On a platform that needs another variable to start a process, the allowlist grows. A
  hook run under `git --git-dir` or in a linked worktree whose git directory differs does not
  decide, and exits 2.
- Follow-ups: the minimum git version; the commands `nuc` runs and the keys each adds to the list.

## Enforcement

`tests/git_cli.rs` cites this ADR with `Verifies: SEC-git-cleared-environment` and checks Q-9. It
runs `nuc check` with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and `GIT_CONFIG_PARAMETERS` set
to a decoy repository, and checks that `nuc` reports on the repository it runs in and leaves the
decoy untouched. It runs `nuc hook pre-commit` with the decoy's index and configuration, and checks
that the hook still decides on its own repository. With the decoy's git directory handed in, it
checks that the hook does not decide and leaves the decoy untouched. On Unix it spawns a git
stand-in first on the `PATH`, which records its environment and arguments. The test fails on a
variable outside the allowlist, on a call without `-C`, and on a call without each key of
`tests/git_cli.neutralized`, the list kept beside it.

`tests/hooks.rs` checks the rest of Q-9. Its test of every shim, which cites Q-9, sets the decoy
for each hook. It checks that a reference transaction that touches no `main` passes, and that the
merge driver merges the files git names it in the repository git runs it in. It checks that every
other hook does not decide, and that no hook touches the decoy. Its test of the merge driver's
conflict style cites this ADR with `Verifies: SEC-git-cleared-environment`. It checks that the
driver takes the style from the repository's configuration and never from a setting `git -c`
handed on in the environment.
