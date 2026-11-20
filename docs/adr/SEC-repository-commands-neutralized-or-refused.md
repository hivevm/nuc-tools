# SEC-repository-commands-neutralized-or-refused: nuc neutralizes the commands a repository's configuration names for git, and where that changes its answer, takes the side that never passes or is not decided

- **Status:** 🟢 accepted
- **Date:** 2026-11-20
- **Deciders:** Markus Brigl
- **Applies to:** every git call of `src/shell/` that reads the working tree, a diff, or a submodule, or hashes a file, the drivers and flags it neutralizes, the refusals `:filter` and `:submodule`, the reading of the caller's environment for git configuration, and `nuc hook pre-push`'s test of a clean tree

## Context

Q-7 asks that `nuc` execute no repository content. G-9 asks that `nuc sense <name>` print the
report of the bash sensor byte for byte, or stop with exit 2 where it cannot decide. Q-1 asks that
a check never pass on input it could not read as the bash does.
[SEC-git-cleared-without-lazy-fetch](SEC-git-cleared-without-lazy-fetch.md) spawns git with a
cleared environment and neutralizes a fixed list of keys by `-c`: "a key that makes a git command
`nuc` uses run another command joins the list". That covers a key with a fixed name. It cannot
cover the commands git runs for a diff, a status, or a hash, since their keys carry a name the
repository chooses:

- a clean or process filter, `filter.<driver>.clean` and `filter.<driver>.process`, which git runs
  for a file whose attributes name the driver, whenever it hashes the file from the working tree;
- a textconv driver, `diff.<driver>.textconv`;
- an external diff, `diff.external` and `diff.<driver>.command`;
- `diff.submodule=diff`, which starts a diff inside a submodule;
- a checked-out submodule, whose status git asks with that submodule's own configuration and
  filters.

`nuc sense` met each of these, and no record decided them first. The review of the sensor
`plain-rules` worked a policy out over four rounds (rounds 2 to 5 of ticket 06 of the spec
`sensors-under-nuc-sense`, 2026-11-19 and 2026-11-20):

- a refusal of every repository with an active filter, which made every repository with LFS
  undecidable;
- rename detection under a neutralized filter, which hashes the raw bytes where the bash's git
  hashes the cleaned ones, and so made `--require` pass where the bash fails;
- a C-quoted path under a filter, which escaped the refusal;
- driver names that were exempted, and a driver named `set`, which `check-attr` cannot tell from a
  bare `filter`;
- a driver configured only in the caller's environment, which `nuc`'s git does not see.

The review of the sensor `binding-adrs` (ticket 05) met the class twice more:
`--ignore-submodules=dirty`, which overrode the repository's own `ignore` setting, and
`diff.submodule`, which started an external diff. The diff that finds the commit setting the date
of `revision-due` ran a textconv driver until `--no-textconv` was added.

Outside `nuc sense`, three calls meet the same class. `nuc check` refuses a hook script under any
filter, since hashing it would run the driver. The rule `docs/base-blobs` of `nuc check` hashes the
`NUC-` files of the record and `AGENTS.md` with `git hash-object`, which runs their filter, so that
the blobs match the bash's. `nuc hook pre-push` asks `git status --porcelain`, which runs every
active filter on a stat-dirty file and asks each submodule's status. The review of this record
found the second; the review of ticket 05 found the third.

Neutralizing a driver keeps it from running, and makes git read the raw bytes where the bash's git
reads what the driver gives. For most paths that changes nothing `nuc` reports: a sensor that never
reads the path, or reads it from a commit, gets the same bytes. For some it does:

- a file of rules under a filter;
- a rename git would pair through the cleaned bytes;
- a file hashed under a filter;
- a stat-dirty file under a filter, which the bash's git hashes through the filter and `nuc`'s
  cannot. Its raw bytes can equal the index while its cleaned bytes do not, as a file committed
  before its filter was set.

There `nuc` cannot know what the bash would have printed.

The same holds for the drivers of a diff. The bash sensors read hunks from a porcelain
`git diff -U0`, and the commit that set a date from `git log -G`. git runs a textconv driver for
both, even into a pipe, and an external diff for the porcelain diff. A `diff.submodule` of `diff`
in the repository's configuration makes the porcelain diff print the submodule's own hunks, which
`nuc`'s `short` leaves out. `nuc` passes `--no-textconv` and `--no-ext-diff`, so it
compares the raw bytes. The review of this record built a proposed ADR under a textconv driver
where the bash's `plain-rules --require` fails and `nuc`'s passes.

The bash's git sees what the caller's environment configures. `GIT_CONFIG_PARAMETERS` and
`GIT_CONFIG_COUNT` hand on what `git -c` sets, and `GIT_EXTERNAL_DIFF` names an external diff.
`XDG_CONFIG_HOME`, `GIT_CONFIG_GLOBAL`,
`GIT_CONFIG_SYSTEM`, `GIT_CONFIG_NOSYSTEM`, and `GIT_ATTR_SOURCE` move where git reads its
configuration and attributes. `nuc`'s git sees none of them, since the environment is cleared. A
driver configured there runs in neither place for `nuc`, so Q-7 holds. The bash reads the path
through it and `nuc` reads it raw. `nuc` sees its own environment and can tell such a driver.

## Decision

We will neutralize, on every git call of `nuc` that can reach them, the commands a repository's
configuration names for a diff, a filter, or a submodule. Where neutralizing changes or may change
the answer, `nuc` takes the side that never passes where the command has one. Otherwise it is not
decided, exit 2.

- **Filters:** a driver is active where some entry of the configuration gives it a `clean` or
  `process` command that is not empty. Each call that may hash a file of the working tree gets
  `-c filter.<driver>.process=` and `-c filter.<driver>.required=false` for every active driver,
  or refuses before it hashes. A driver no entry makes active runs nothing and is left as git reads
  it. A driver whose name `-c` cannot carry, one with `=` or not UTF-8, is not decided.
- **A file hashed for its blob:** where a check hashes a file of the working tree to compare its
  blob, and an active filter covers the file, the check is not decided.
- **textconv and external diffs:** every `diff`, `diff-tree`, `log -p`, `log -G`, and `show` that
  produces a patch or compares content carries `--no-textconv` and `--no-ext-diff`.
- **A driver that changes what a diff compares:** a path whose hunks or compared content decide
  what `nuc` prints is not decided, and named, where the bash's git reads it through a driver. That
  is a textconv driver or a diff command the configuration makes active, `diff.external`, or a
  `diff.submodule` of `diff` or `log` where the change moves a submodule. A driver is active as a
  filter is: an entry gives it a command that is not empty. A bash call reads none where it passes
  `--no-textconv` and `--no-ext-diff` itself, or prints no patch and searches no content. `log -G`
  searches content through a textconv driver, but runs no external diff.
- **diff.submodule:** stays on the list of
  [SEC-git-cleared-without-lazy-fetch](SEC-git-cleared-without-lazy-fetch.md) as `short`.
- **Submodules:** git is never asked into a checked-out submodule's working tree. A call that reads
  the working tree leaves submodules out with `--ignore-submodules=all` only where the bash's answer
  does not depend on them. Where it does, a checked-out submodule takes the side that never passes,
  or is not decided, `:submodule`. A submodule that is not checked out is read as git reads it.
- **A stat-dirty path under a filter:** a tracked path under an active filter counts as changed,
  whatever its raw bytes, where its stat differs from the index. It counts so too where its stat
  matches but is no older than the index, since git then compares the content. git cannot name
  that path without hashing it, so `nuc` compares the file's modification time with the index's,
  in whole seconds as git compares them. Where git compares finer, that errs toward counting.
- **The side that never passes:** it applies to a path under an active filter whose content or
  pairing decides what `nuc` prints. It applies to a working tree whose cleanness depends on a
  filter or a submodule. `nuc hook pre-push` takes such a tree as not clean and runs the checks. A
  command with no such side is not decided, `:filter` or `:submodule`, and names the path.
- **A wider refusal:** a refusal that reaches further than this decision asks stands, since it never
  passes. `nuc check` refusing a hook under any `filter` value is one.
- **The caller's environment:** `nuc` reads its own environment for git configuration and never
  passes it on. Where it names a command of the list above that `nuc`'s git would not see, each
  command of `nuc` whose answer reads a diff, a filtered file, or a hash of the working tree takes
  the side that never passes, or is not decided. It names one where:
  - `GIT_CONFIG_PARAMETERS` or the `GIT_CONFIG_KEY_<n>` entries give a key under `filter.`,
    `diff.<driver>.textconv`, `diff.<driver>.command`, `diff.external`, or `diff.submodule`;
  - those entries give a `core.attributesFile`, `include.path`, or `includeIf.<condition>.path`
    whose file names such a driver, or a `filter` or `diff` attribute for one;
  - `GIT_EXTERNAL_DIFF` is set;
  - `XDG_CONFIG_HOME`, `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_SYSTEM`, `GIT_CONFIG_NOSYSTEM`, or
    `GIT_ATTR_SOURCE` makes git read a configuration or attributes file other than `nuc`'s git
    reads, and that file names such a driver, or a `filter` or `diff` attribute for one, itself or
    through an `include.path` or `core.attributesFile` it names.

  A variable set to what `nuc`'s git reads anyway names none.

**Out of scope:** the fixed keys and the cleared environment, which
[SEC-git-cleared-without-lazy-fetch](SEC-git-cleared-without-lazy-fetch.md) decides. A merge
driver, which `nuc` is and does not call. Commands git runs outside a diff, a status, or a hash of
the working tree, such as a credential helper or a transport: that ADR reaches them by its list and
by `--no-lazy-fetch`. Reading a submodule with its own drivers neutralized, which a later decision
may take up (`docs/backlog/binding-adrs-reads-submodules.md`). Configuration in the caller's
environment that names no command but changes what git reads, such as `core.autocrlf` or an `eol`
or `-diff` attribute: `nuc`'s git does not see it either, and a later decision may take it up.

## Latitude

None.

## Alternatives considered

- **Refuse every repository with an active filter** — one rule, the simplest to check, and the
  first shape of `plain-rules`. It made every repository with LFS undecidable, although the sensors
  rarely read a path under LFS. Rejected in round 2.
- **Report the raw bytes, and name the difference in G-9** — never exit 2, but where the filter
  decides, `nuc` may pass where the bash fails, against Q-1 and the parity G-9 promises.
- **Run the repository's drivers** — the bash's answer, but repository content executed, against
  Q-7. `docs/base-blobs` does this today, trusting the configuration as
  SEC-git-cleared-without-lazy-fetch trusts the user's.
- **Hash with `--no-filters`** — no driver runs, but the blob differs from the bash's wherever line
  endings are converted, and the check would decide on other input than the bash's.
- **Read only committed objects** — no filter ever runs, but the bash sensors read a change that
  ends in the working tree, and the reports would differ (G-9).
- **Count a stat-dirty path under a filter only where its raw bytes differ** — fewer runs not
  decided after a `touch`. But a file committed before its filter was set is unchanged raw and
  changed cleaned, and `nuc hook pre-push` would take the green record where the bash hook runs
  the checks.
- **No system or global configuration** — rejected in
  [SEC-git-cleared-without-lazy-fetch](SEC-git-cleared-without-lazy-fetch.md): it drops
  `safe.directory`, and the repository's own configuration, which names the drivers, would remain.
- **A key per driver on the fixed list** — what the current clause asks; the names are the
  repository's, so the list can never be complete.
- **Leave a driver of the caller's environment as a known gap** — no code, but a run that reads
  through such a driver may pass where the bash fails, and `nuc` can tell it.
- **Read only `GIT_CONFIG_PARAMETERS` and `GIT_CONFIG_COUNT`** — the two variables `git -c` uses,
  but a global configuration under another `XDG_CONFIG_HOME` hides an LFS driver as well.
- **Refuse wherever one of those variables is set** — simple, but `XDG_CONFIG_HOME` is set on many
  desktops, and most runs would end not decided.
- **Narrow `nuc check`'s refusal of a hook to active drivers** — the sensors' rule, but reading the
  configuration per driver for a refusal that never passes buys nothing.
- **Read a checked-out submodule with its own drivers neutralized** — fewer runs not decided, but
  nested submodules, their `ignore` settings, and a second filter pass for each; kept as a backlog
  entry.
- **Supersede SEC-git-cleared-without-lazy-fetch with one ADR for both** — one record of what `nuc`
  passes git, but two decisions with different forces: the environment and the fixed keys concern
  every call, the drivers only those that read content.

## Sources / Prior art

- [gitattributes(5)](https://git-scm.com/docs/gitattributes): `filter`, its `clean`, `smudge`,
  `process`, and `required`, `diff` and textconv, and when git runs each.
- [git-config(1)](https://git-scm.com/docs/git-config): `filter.<driver>.*`,
  `diff.<driver>.textconv`, `diff.<driver>.command`, `diff.external`, `diff.submodule`,
  `diff.ignoreSubmodules`, `submodule.<name>.ignore`, `core.attributesFile`, `include.path`, and
  the files git reads under `XDG_CONFIG_HOME`.
- [git-diff(1)](https://git-scm.com/docs/git-diff): `--no-textconv`, `--no-ext-diff`,
  `--ignore-submodules`.
- [git-hash-object(1)](https://git-scm.com/docs/git-hash-object): `--no-filters`, and the filters a
  hash of a working-tree file applies.
- [git(1), *Environment variables*](https://git-scm.com/docs/git#_environment_variables):
  `GIT_CONFIG_PARAMETERS`, `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_<n>`, `GIT_CONFIG_VALUE_<n>`,
  `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_SYSTEM`, `GIT_CONFIG_NOSYSTEM`, `GIT_ATTR_SOURCE`,
  `GIT_EXTERNAL_DIFF`.
- The reviews of tickets 05 and 06 of `sensors-under-nuc-sense`, named under *Context*. Each built
  a driver that writes a canary file and saw it written. The review of this record ran a clean
  command through `git hash-object` with git 2.56.1.

## Consequences

- Positive: a repository's filter, textconv driver, external diff, or submodule runs no command
  when `nuc` reads it (Q-7). Where the neutralized read can differ from the bash's, `nuc` never
  passes in its place (Q-1, G-9). A repository with LFS runs the sensors and the checks, as long as
  no path under its filter decides the answer.
- Negative / trade-offs: a run is not decided where the bash would report. That holds for:
  - a file of rules under a filter;
  - a `NUC-` file of the record or `AGENTS.md` under a filter in `docs/base-blobs`;
  - a working-tree change with a checked-out submodule, or a stat-dirty path under a filter, in
    `binding-adrs`;
  - a path whose hunks decide the report under a textconv driver, a diff command, or
    `diff.external` of the repository, in `tests-kept`, `plain-rules`, and `binding-adrs`;
  - a change that moves a submodule under a `diff.submodule` of `diff` or `log`, in `tests-kept`
    and in `binding-adrs` where the submodule is bound;
  - `docs/ARCHITECTURE.md` under a textconv driver, in `revision-due`;
  - a caller who sets a driver with `git -c` or in a configuration only the caller's git reads.

  `nuc hook pre-push` runs the checks on a tree the bash hook might take as clean, where a path
  under a filter is stat-dirty or a submodule is checked out. A `touch` of such a file has the
  same effect until git refreshes the index. Each call that may hash the working tree asks the
  configuration for its filter drivers first, and the paths under them.
- Follow-ups: `nuc hook pre-push`'s test of a clean tree; the refusal of a command the caller's
  environment configures; the refusal of a hashed file under a filter in `docs/base-blobs`; the
  refusal of a path whose diff a driver would change; the count of a stat-dirty path under a filter
  in `binding-adrs`; one guarded call for every diff of `nuc`, so that no call forgets a flag; the
  place in the specification where `nuc sense` may be not decided where the bash reports.

## Enforcement

Tests of `tests/contract.rs` cite this ADR with `Verifies:
SEC-repository-commands-neutralized-or-refused`. Each builds a repository whose configuration names
a driver. It checks what the sensor reports or that it is not decided, as this ADR says.

- The textconv driver, the external diffs, and `diff.submodule`, each with a canary file the
  driver would write:
  - `the_anchor_runs_no_textconv_driver_of_the_repository`;
  - `tests_kept_runs_no_driver_of_the_repository`;
  - `plain_rules_runs_no_driver_of_the_repository`;
  - `binding_adrs_runs_no_driver_of_the_repository`;
  - `review_binding_runs_no_driver_of_the_repository`, a canary alone: git runs none of these for
    the calls review-binding makes, so it fails on a call that would start one, not on a flag taken
    away.
- The filters:
  - `plain_rules_in_the_working_tree_runs_no_filter`;
  - `plain_rules_neutralizes_a_filter_outside_the_rules`;
  - `plain_rules_reads_the_filter_attribute_as_git_does`;
  - `plain_rules_refuses_a_rename_a_filter_could_pair_otherwise`, which checks the refusal with a
    filter that writes no canary;
  - `plain_rules_runs_on_an_adr_added_beside_a_filtered_file`;
  - `plain_rules_refuses_a_rule_file_under_a_driver_named_set`;
  - `plain_rules_leaves_a_driver_without_a_command_to_git`;
  - `binding_adrs_in_the_working_tree_runs_no_filter`, for the canary and the refusal of a
    checked-out submodule. Its first assertion, a report on a stat-dirty path under a filter whose
    raw bytes equal the index, turns to not decided with the change that counts such a path.

`tests/check.rs` cites it the same way in
`a_hook_whose_attributes_name_a_filter_is_not_hashed_and_not_decided`.

The code does not yet hold six clauses. Their tests join in the changes that implement them:

- a test in `tests/contract.rs` that names a driver through the caller's environment, by
  `GIT_CONFIG_PARAMETERS`, `GIT_EXTERNAL_DIFF`, and `XDG_CONFIG_HOME`, and checks that
  `nuc sense plain-rules` is not decided, with tests beside it for `nuc check` and `nuc hook`;
- a test in `tests/hooks.rs` that checks that `nuc hook pre-push` runs no filter and asks no
  checked-out submodule, and runs the checks on a tree with a stat-dirty or racily clean path under
  a filter, a checked-out submodule, or a staged change of a submodule;
- a test in `tests/check.rs` that checks that `docs/base-blobs` runs no filter and is not decided;
- a test in `tests/contract.rs` that checks that `binding-adrs` is not decided on a stat-dirty
  path under a filter whose raw bytes equal the index, and on a racily clean one, whose
  modification time falls in the second of the index's;
- the stand-in test of `tests/git_cli.rs` that checks the flags on every diff call, the `log -G`
  of revision-due among them, which lacks `--no-ext-diff` today;
- tests in `tests/contract.rs` that check that each sensor whose bash reads a patch is not decided
  where a path whose hunks decide its report has an active textconv driver or diff command, where
  `diff.external` is set, or where `diff.submodule` reads `diff` and the change moves a submodule.
  A test beside them checks that `revision-due` is not decided where `docs/ARCHITECTURE.md` has an
  active textconv driver, which its bash's `log -G` searches through. The five tests above for the
  textconv driver, the external diffs, and `diff.submodule` then put their drivers on a path that
  decides nothing and keep the canary, or expect the refusal where every path decides.
