# Ticket: The sensors refuse a path whose diff a driver of the repository would change

## What to build

The bash sensors read hunks from a porcelain `git diff -U0`, and revision-due reads the commit that
set its date from `git log -G`. git runs a textconv driver for both, even into a pipe, and a diff
command and `diff.external` for the porcelain diff. A `diff.submodule` of `diff` in the repository's
configuration makes the porcelain diff print the submodule's own hunks, where tests-kept can find a
marker that `nuc`'s `short` hides. `nuc` passes `--no-textconv` and `--no-ext-diff` (Q-7), so it
compares the raw bytes, and its report can differ. The review of
SEC-repository-commands-neutralized-or-refused built a proposed ADR under a textconv driver where
the bash's `plain-rules --require` fails and `nuc`'s passes.

That ADR decides: a path whose hunks or compared content decide what `nuc` prints is not decided,
and named, where the bash's git reads it through a textconv driver or a diff command the
configuration makes active, through `diff.external`, or with a `diff.submodule` of `diff` or `log`
where the change moves a submodule. A driver is active as a filter is. `log -G` reads through a
textconv driver but runs no external diff.

The bash calls that read a patch, at v0.12.0:

- tests-kept: `git diff -U0` over the change;
- plain-rules: `git diff -U0 -M` of each file of rules (`scripts/lib/rules.sh`);
- binding-adrs: `git diff -U0 --no-renames` of each path it binds;
- revision-due: `git log -G` on `docs/ARCHITECTURE.md`.

Check each release `nuc sense` holds, and review-binding, whose `diff --stat` and plumbing
`diff-tree` print no patch through a driver. Read the active drivers once a run, as
`neutralized_filters` reads the filters, with `check-attr diff` for the paths whose hunks decide.
`diff.external` set refuses every such path. Share the reading with `shell::sense`, and with the
guarded call of ticket 03 where it lands first.

The five tests of `tests/contract.rs` that SEC-repository-commands-neutralized-or-refused cites
for the textconv driver, the external diffs, and `diff.submodule` expect a report today:

- `the_anchor_runs_no_textconv_driver_of_the_repository`;
- `tests_kept_runs_no_driver_of_the_repository`;
- `plain_rules_runs_no_driver_of_the_repository`;
- `binding_adrs_runs_no_driver_of_the_repository`;
- `review_binding_runs_no_driver_of_the_repository`.

Each puts its driver on a path that decides nothing, and keeps its canary. Their halves that move a
submodule under `diff.submodule=diff` (`moved_submodule`) expect exit 2 instead, with the canary
kept: in tests-kept every path decides, and binding-adrs diffs the submodule where checks name it. A
canary on such a path no longer fails on a flag taken away, since `nuc` diffs no path that decides
nothing: the stand-in test of ticket 03 checks the flags instead, so this ticket waits for it;
`diff.external` moves to a test of its own, which expects the refusal. The reason is the ADR's
clause.

## Acceptance criteria

- [ ] Given a proposed ADR under an active textconv driver whose change inserts a line above an old
  "should" line, when `nuc sense plain-rules --require` runs, then it exits 2 and names the path,
  and the driver's canary is not created.
- [ ] Given `diff.external` set in the repository's configuration, when tests-kept, plain-rules,
  and binding-adrs run on a change whose hunks decide their report, then each exits 2.
- [ ] Given a textconv driver on `docs/ARCHITECTURE.md`, when `nuc sense revision-due` runs, then
  it exits 2; given only `diff.external`, it reports as without it.
- [ ] Given `diff.submodule=diff` in the repository's configuration and a change that moves a
  submodule, when `nuc sense tests-kept` runs, then it exits 2.
- [ ] Given `diff.submodule=diff` and a change that moves a submodule that checks name, when
  `nuc sense binding-adrs` runs, then it exits 2.
- [ ] Given the fixture of the stand-in test of `tests/git_cli.rs` that ticket 03 builds, when this
  change lands, then that test still sees every diff call it checks: its drivers sit on paths that
  decide nothing, or it checks the refusal.
- [ ] Given a textconv driver on a path whose hunks decide nothing, when each sensor runs, then it
  reports as without it, and no driver runs.
- [ ] Given the parity harness, when it runs, then every row passes unchanged.

## Read first

- Touches: `src/shell/sense.rs`, `src/shell/git.rs`, `tests/contract.rs`, `tests/git_cli.rs`
- SEC-repository-commands-neutralized-or-refused; API-git-only-sensors-under-sense

## Blocked by

- docs/plan/revision-2026-11-20/03-every-diff-of-nuc-runs-through-one-guarded-call.md

## Mode

with the human: it changes the code that spawns git, a protected part of DEV-one-agent-one-ticket.
