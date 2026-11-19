# SEC-git-cleared-without-lazy-fetch: The shell spawns the git CLI with a cleared environment, an allowlist passed through, no lazy fetch, the repository named by `-C`, and command-running configuration neutralized

- **Status:** 🟡 proposed
- **Date:** 2026-11-19
- **Deciders:** Markus Brigl
- **Applies to:** every spawn of git in `src/shell/`, the environment and arguments it passes, the list of neutralized configuration keys kept beside its test, the git versions that block the lazy fetch, and every read of an object a partial clone may lack
- **Supersedes:** [SEC-git-cleared-environment](SEC-git-cleared-environment.md)

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

A partial clone lacks objects, and git fetches one from the promisor remote the moment a command
reads it: `cat-file` in `nuc check`, `show` and `diff` in `nuc sense`. That fetch starts a
transport, and a transport runs what the repository's own configuration names, such as
`core.sshCommand`, so a repository can make `nuc` execute its content (Q-7) and reach the network
(Q-11). A neutralized `protocol.allow=never` does not hold: `protocol.<name>.allow` in the
repository's configuration overrides it, transport by transport. git 2.45.0 brings `git
--no-lazy-fetch`, which no configuration turns back on. With the fetch blocked, an object the clone
lacks fails to read, and a read that takes the failure for an absent file decides on wrong input
(Q-1).

## Decision

We will spawn git from `src/shell/` only with a cleared environment into which an allowlist is
passed, with no lazy fetch, the repository named by `git -C <path>`, and the configuration keys that
run commands neutralized by `-c` on every call.

- **Allowlist:** `PATH` and `HOME`, and on Windows `SystemRoot`, passed from the parent; set by
  `nuc`: `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, and `GIT_OPTIONAL_LOCKS=0`. Nothing else, no
  variable of the parent starting with `GIT_` among them.
- **Repository:** determined from the directory `nuc` runs in, or from an explicit `--repo`, never
  from the environment, and passed to every call as `-C <path>`.
- **No lazy fetch:** `nuc` must ask `git --version` once a run, under the cleared environment;
  this probe of the version is the one call without `-C` and the neutralized keys, since it reads
  no repository.
  - From git 2.45.0 on, `--no-lazy-fetch` opens every call, before `-C`.
  - A git before it gets no such option. It must run only on a repository whose configuration names
    no promisor remote, neither `extensions.partialClone` nor a `remote.<name>.promisor` whose value
    is other than the literal `false`, which errs toward refusing. On a repository that names one,
    the run ends not decided, exit 2, with a message that names 2.45.0.
- **Missing objects:** a read of an object tells a path the commit does not have from an object the
  clone lacks. The first is read as the bash reads an absent file; the second ends the run not
  decided, exit 2 (Q-1).
- **Neutralized keys:** `core.fsmonitor=false`, `log.showSignature=false`, `protocol.allow=never`,
  and `core.hooksPath` set to the null device (`/dev/null`, on Windows `NUL`), which can hold no
  hook and needs nothing written. A key that
  makes a git command `nuc` uses run another command joins the list, kept beside the test, in the
  change that starts using that command.
- **Configuration:** the system and global configuration stay in effect, so that git behaves as
  it does for the user.

**Out of scope:** which git commands `nuc` runs; processes other than git, which `nuc` does not
spawn; a project that wants its partial clone fetched, which fetches before it runs `nuc`; a git
whose `--version` names no version, which ends the run not decided.

## Alternatives considered

- **No system and global configuration** (`GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL` emptied) —
  safer against a hostile configuration. But it drops `safe.directory`, which a container needs.
  It also makes git behave unlike the user's, against the reason for the CLI and likely against
  parity (Q-2).
- **Only the `GIT_*` variables removed, the rest inherited** — tokens such as `GITHUB_TOKEN` would
  reach git and whatever it starts, against Q-10.
- **No clause for the lazy fetch** — what the record decided before it, accepted on 2026-10-31
  with every other clause of this one. It left the fetch of a partial clone to run what the
  repository's configuration names, and this ADR replaced that clause.
- **git 2.45.0 or later on every run** — one rule and no probe, but long-term distributions ship
  an older git, and on such a host every command of `nuc` would refuse, the hooks a project runs on
  each commit and push among them, although only a partial clone fetches lazily.
- **`protocol.allow=never` alone** — one more neutralized key, no version, but the repository's
  `protocol.<name>.allow` turns each transport back on. It stays on the list, a second guard.
- **`GIT_NO_LAZY_FETCH=1` in the allowlist** — the same effect from 2.45.0 on, but a git before it
  ignores the variable without a word, and the gap would stay open where nobody sees it.
- **Every `protocol.<name>.allow` neutralized** — the list of transports has no end: a remote
  helper names its own.
- **The fetch allowed, the object read** — the answer the bash gives on a partial clone, but the
  network and the repository's commands, against Q-7 and Q-11.
- **libgit2 or gitoxide** — excluded by the constraints: the behaviour would be a library's, not
  git's. The `bans` of `deny.toml` refuse such crates.

## Sources / Prior art

- [git(1), *Environment variables*](https://git-scm.com/docs/git#_environment_variables):
  `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_CONFIG_PARAMETERS`, `GIT_OPTIONAL_LOCKS`,
  `GIT_TERMINAL_PROMPT`, and `-C`.
- [githooks(5)](https://git-scm.com/docs/githooks): the variables git sets for a hook.
- [git-config(1)](https://git-scm.com/docs/git-config): `core.fsmonitor`, `core.hooksPath`,
  `safe.directory`, `protocol.allow`, `protocol.<name>.allow`, `core.sshCommand`.
- [git(1)](https://git-scm.com/docs/git): `--no-lazy-fetch`, which the release notes of git
  2.45.0 announce
  (<https://raw.githubusercontent.com/git/git/master/Documentation/RelNotes/2.45.0.adoc>); the
  notes of 2.44.0 do not name it.
- The review of `nuc sense` on 2026-11-19, which built a partial clone whose `core.sshCommand` ran
  when `nuc` read a missing object, and turned it back on with `protocol.ssh.allow=always`.

## Consequences

- Positive: an inherited `GIT_DIR` or `GIT_CONFIG_PARAMETERS` cannot redirect `nuc` (Q-9). No token
  of the parent reaches git (Q-10). The hooks, the file-system monitor, and the transports of the
  target repository do not run (Q-7, Q-11). A setting handed on in `GIT_CONFIG_PARAMETERS`, as `git
  -c` does, does not reach the git `nuc` runs; the merge driver takes its conflict style from the
  configuration, never from the environment.
- Negative / trade-offs: a key the list does not yet name can still run a command until it is
  added. The user's global configuration is trusted, and a hostile one is out of reach of this
  decision. On a platform that needs another variable to start a process, the allowlist grows. A
  hook run under `git --git-dir` or in a linked worktree whose git directory differs does not
  decide, and exits 2. Each run asks `git --version` once more, and with a git before 2.45.0, as
  long-term distributions ship, it asks each repository's configuration once as well. On such a
  git a partial clone is not decided at all, even where `nuc` would read none of its missing
  objects. A partial clone that lacks an object `nuc` reads is not decided, where the bash fetches
  the object and reports; CI that clones without blobs fetches them first.
- Follow-ups: the commands `nuc` runs and the keys each adds to the list.

## Enforcement

`tests/git_cli.rs` cites this ADR with `Verifies: SEC-git-cleared-without-lazy-fetch` and checks
Q-9. It runs `nuc check` with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and
`GIT_CONFIG_PARAMETERS` set to a decoy repository, and checks that `nuc` reports on the repository
it runs in and leaves the decoy untouched. It runs `nuc hook pre-commit` with the decoy's index and
configuration, and checks that the hook still decides on its own repository. With the decoy's git
directory handed in, it checks that the hook does not decide and leaves the decoy untouched. On Unix
it spawns a git stand-in first on the `PATH`, which answers `--version` as git 2.45.0 and records
its environment and arguments. The test fails on a variable outside the allowlist, on a call that
does not open with `--no-lazy-fetch` and `-C`, and on a call without each key of
`tests/git_cli.neutralized`, the list kept beside it. With a stand-in that answers as git 2.43.0 and
refuses `--no-lazy-fetch`, it checks that `nuc check` passes on a repository without a promisor
remote, and that `nuc check` and `nuc sense` exit 2 and name 2.45.0 on one with a promisor remote.

`tests/contract.rs` cites this ADR the same way. On a partial clone whose configuration names a
command for the ssh transport and turns that transport back on with `protocol.ssh.allow=always`,
it checks that `nuc sense backlog` runs no command and is not decided where an entry's object is
missing. On a partial clone that lacks the README at the merge base with main, it checks that
`nuc check` is not decided where `docs/template-release` reads it.

`tests/hooks.rs` checks the rest of Q-9. Its test of every shim, which cites Q-9, sets the decoy for
each hook. It checks that a reference transaction that touches no `main` passes, and that the merge
driver merges the files git names it in the repository git runs it in. It checks that every other
hook does not decide, and that no hook touches the decoy. Its test of the merge driver's conflict
style cites this ADR with `Verifies: SEC-git-cleared-without-lazy-fetch`. It checks that the driver
takes the style from the repository's configuration and never from a setting `git -c` handed on in
the environment.
