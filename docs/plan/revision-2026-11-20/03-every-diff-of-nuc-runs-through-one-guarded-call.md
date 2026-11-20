# Ticket: Every diff of nuc runs through one guarded call, and a test drives every sensor in both forms

## What to build

The guards `--no-ext-diff --no-textconv`, and `-c core.quotePath=false` where the release's bash
sets it, are written by hand at about ten call sites of `src/shell/sense.rs`, and so are the `-c`
keys of the neutralized filter drivers. The reviews found a repository's command run where one site
lacked a guard:

- the textconv driver under `git log -G` (ticket 01, first review);
- the clean filter of the working-tree diff of plain-rules (ticket 06, first review). The sensor's
  test of its drivers ran only the form that reads a range of commits.

`tests/git_cli.rs` checks that every call carries the keys of `tests/git_cli.neutralized`. That
test drives `nuc check` alone; only the test of the lazy fetch beside it runs a sensor,
`nuc sense revision-due`.

SEC-repository-commands-neutralized-or-refused decides which guards and neutralizations `nuc` uses;
this ticket builds the function and the fixture once. Build one function in `shell::git` that starts
a diff, a `diff-tree`, a `log -p`, or a `log -G` with every guard. The call sites of `shell::sense`
call it. Extend the stand-in test of `tests/git_cli.rs`: it drives every sensor of `sense::ALL`, for
a sensor that reads a change both with a head and on the working tree, against a fixture whose
configuration names a clean and a process filter, a textconv driver, an external diff, and
`diff.submodule`. It asserts the guards on each diff call, and the canary uncreated. The `log -G` of
revision-due carries `--no-textconv` alone today; through the function it gets `--no-ext-diff` too,
as the ADR asks.

## Acceptance criteria

- [ ] Given `src/shell/sense.rs`, when it is searched for `--no-textconv` and `--no-ext-diff`,
  then neither stands there, only in `shell::git`.
- [ ] Given a sensor that reads a change, when the test runs it with a head and on the working
  tree against the fixture, then every diff call carries the guards and no command of the fixture
  runs.
- [ ] Given a mutant that drops one guard from the new function, when the suite runs, then it
  fails.

## Read first

- Touches: `src/shell/git.rs`, `src/shell/sense.rs`, `tests/git_cli.rs`, `tests/git_cli.neutralized`
- SEC-git-cleared-without-lazy-fetch; SEC-repository-commands-neutralized-or-refused;
  API-git-only-sensors-under-sense; the convention "A git command a port adds is tried against the
  repository's own commands" in `docs/CONVENTIONS.md`, which this check makes needless where it
  holds

## Blocked by

- None (can start immediately)

## Mode

with the human: it changes the code that spawns git, a protected part of DEV-one-agent-one-ticket.
