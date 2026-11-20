# Ticket: nuc refuses a command the caller's environment configures

## What to build

SEC-repository-commands-neutralized-or-refused decides that `nuc` reads its own environment for git
configuration. Where it names a filter, a textconv driver, or an external diff that `nuc`'s git,
under the cleared environment, would not see, every command of `nuc` takes the side that never
passes, or is not decided. The bash's git reads a path through such a command and `nuc`'s reads it
raw, so the answer can differ (G-9, Q-1). The code does not yet hold this: the review of ticket 06
of `sensors-under-nuc-sense` built a case where `plain-rules --require` passes where the bash fails.

Read in `src/shell/`, once a run, before any git call, and pass none of them on:

- `GIT_CONFIG_PARAMETERS`, as git parses its quoted `'key'='value'` pairs, and `GIT_CONFIG_COUNT`
  with its `GIT_CONFIG_KEY_<n>` and `GIT_CONFIG_VALUE_<n>`. A key under `filter.`,
  `diff.<driver>.textconv`, `diff.<driver>.command`, `diff.external`, or `diff.submodule` names one.
  So does a `core.attributesFile`, `include.path`, or `includeIf.<condition>.path` whose file names
  such a driver, or a `filter` or `diff` attribute for one;
- `GIT_EXTERNAL_DIFF`, set to anything: the bash of binding-adrs runs a porcelain `git diff`
  without `--no-ext-diff`, whose hunks the command replaces;
- `XDG_CONFIG_HOME`, `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_SYSTEM`, `GIT_CONFIG_NOSYSTEM`, and
  `GIT_ATTR_SOURCE`. Each names one only where it makes git read a configuration or attributes file
  other than `nuc`'s git reads, and that file names such a driver, or a `filter` or `diff` attribute
  for one, itself or through an `include.path` or `core.attributesFile` it names. A variable set to
  the file `nuc`'s git reads anyway names none.

Then: `nuc sense` is not decided, `nuc check` is not decided, and `nuc hook pre-push` takes the tree
as not clean. The commands that read no diff, filtered file, or hash of the working tree, such as
`nuc measure` and `nuc explain`, are untouched. The tests that run `nuc` with the runner's own
environment, such as those of `tests/git_cli.rs` that run `nuc check`, clear these variables, so
that a developer's `XDG_CONFIG_HOME` does not decide them. A value that cannot be parsed counts as
naming one, the side that never passes.

## Acceptance criteria

- [ ] Given a repository with a file of rules the change adds, when `nuc sense plain-rules` runs
  with `GIT_CONFIG_PARAMETERS` naming `filter.<d>.clean`, then it exits 2 and names the key.
- [ ] Given `GIT_CONFIG_COUNT=1` with `GIT_CONFIG_KEY_0=diff.external`, or `GIT_EXTERNAL_DIFF`
  set, when any sensor runs, then it exits 2.
- [ ] Given `GIT_CONFIG_PARAMETERS` naming a `core.attributesFile` whose file puts a file of rules
  under the repository's active driver, when `nuc sense plain-rules` runs, then it exits 2.
- [ ] Given `XDG_CONFIG_HOME`, `GIT_CONFIG_GLOBAL`, or `GIT_ATTR_SOURCE` pointing at a file that
  names a filter driver or a `filter` attribute, and no such file where `nuc`'s git reads, when
  `nuc sense plain-rules` runs, then it exits 2.
- [ ] Given `GIT_CONFIG_GLOBAL` pointing at a file whose `include.path` names a file that defines
  a filter driver, when `nuc sense plain-rules` runs, then it exits 2.
- [ ] Given such a variable, when `nuc check` runs, then it exits 2, and when `nuc hook pre-push`
  runs, then it runs the checks.
- [ ] Given those variables naming only other keys, such as `user.name`, or `XDG_CONFIG_HOME` set
  to `$HOME/.config`, when the sensor runs, then it reports as without them.

## Read first

- Touches: `src/shell/sense.rs`, `src/shell/git.rs`, `src/shell/hook.rs`, `src/shell/tree.rs`,
  `tests/contract.rs`, `tests/check.rs`, `tests/hooks.rs`, `tests/git_cli.rs`
- SEC-repository-commands-neutralized-or-refused; SEC-git-cleared-without-lazy-fetch;
  [git(1), *Environment variables*](https://git-scm.com/docs/git#_environment_variables);
  [git-config(1), *Files*](https://git-scm.com/docs/git-config#FILES)

## Blocked by

- None (can start immediately)

## Mode

with the human: it changes the code that spawns git, a protected part of DEV-one-agent-one-ticket.
