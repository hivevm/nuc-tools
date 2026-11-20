# Ticket: binding-adrs counts every stat-dirty path under a filter

## What to build

In a change that ends in the working tree, `nuc sense binding-adrs` refuses a tracked path under an
active filter only where its raw bytes differ from the index (`refuse_filtered_dirty` in
`src/shell/sense.rs`). A path whose stat changed but whose raw bytes equal the index passes
unrefused. The bash's git hashes it through the filter and can list it as changed, as for a file
committed before its filter was set. `nuc` then binds fewer ADRs than the bash, and no refusal
follows. `docs/ARCHITECTURE.md` names the case as a known difference.

SEC-repository-commands-neutralized-or-refused decides that every tracked path under an active
filter whose stat differs from the index counts as changed, whatever its raw bytes. binding-adrs has
no side that never passes for it, so the run is not decided, `:filter`. So is a path whose stat
matches the index but is no older than it, a racily clean entry, since git then compares the
content. Name the stat-dirty paths with `git diff-files --name-only`, under the same neutralizing
keys as the diff, since git hashes a racily clean entry. `diff-files` does not name a racily clean
path whose raw bytes equal the index, so compare the modification time of each tracked path under an
active filter with the index file's in whole seconds, as git compares them, in `src/shell/`. Then
correct the sentence of `docs/ARCHITECTURE.md` that names the difference.

The parity scenario `bind_filtered` of `tests/parity/sensor-scenarios.sh` builds this case, and
the harness compares the standard output byte for byte whatever the exits, so a refusal cannot
pass there. Change the scenario so that `a.png` is older than the index by whole seconds and its
stat unchanged: neither stat-dirty nor racily clean, read alike by both. Its rows
`binding-adrs bind_filtered HEAD` and `main` of `tests/parity/sensors.tsv` then pass unchanged.
The refusal is pinned by the contract test, not by parity.

The first assertion of `binding_adrs_in_the_working_tree_runs_no_filter` in `tests/contract.rs`
changes: it expects exit 0 and the report on `c.sh` while `a.png`, a stat-dirty path under an
active filter whose raw bytes equal the index, lies in the tree, and now expects exit 2 with
`:filter`. The reason is the ADR's clause. The
fixture moves the report it checked to a path outside the filter, or backdates `a.png` by whole
seconds before the index, so the canary and the submodule refusal stay covered. Its doc comment,
that of `refuse_filtered_dirty`, that of `binding_adrs` in `src/shell/sense.rs`, and the comment of
the scenario `bind_filtered` in `tests/parity/sensor-scenarios.sh`, which name the case as not
refused or read alike, change with it.

## Acceptance criteria

- [ ] Given a file under an active filter, committed before the filter was set and then touched,
  when `nuc sense binding-adrs` runs on the working tree, then it exits 2 with `:filter` and names
  the path, and the filter's canary is not created.
- [ ] Given a file under an active filter whose stat is unchanged and older than the index, when
  the sensor runs, then it reports as today.
- [ ] Given a file under an active filter whose raw bytes equal the index and whose modification
  time falls in the same second as the index's, below it by a fraction, when the sensor runs, then
  it exits 2 with `:filter`, and no filter runs.
- [ ] Given the parity harness, when it runs, then every row passes, `bind_filtered` with its
  changed scenario.

## Read first

- Touches: `src/shell/sense.rs`, `tests/contract.rs`, `docs/ARCHITECTURE.md`,
  `tests/parity/sensor-scenarios.sh`
- SEC-repository-commands-neutralized-or-refused; API-git-only-sensors-under-sense

## Blocked by

- None (can start immediately)

## Mode

with the human: it changes the code that spawns git, a protected part of DEV-one-agent-one-ticket.
