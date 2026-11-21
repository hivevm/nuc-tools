# API-sensor-cores-ask-the-shell: `nuc sense <name>` ports the template's sensors that need only git, prints their report byte for byte as the bash does, exits as every `nuc` command does, and a sensor's core may ask the shell only where gathering its inputs up front would read more than its bash and no value it returns first could tell the shell what to read

- **Status:** 🟡 proposed
- **Date:** 2026-11-21
- **Deciders:** Markus Brigl
- **Applies to:** which sensors of the template `nuc` ports, the `nuc sense` command, its source in the shell and in the core, the names of the sensors it knows, its exit codes against the bash's, the comparison of the sensors in the parity harness, and the traits and callbacks through which a sensor's core asks the shell
- **Supersedes:** [API-git-only-sensors-under-sense](API-git-only-sensors-under-sense.md)

## Context

The problem names the sensors among the bash `nuc` is to replace (*Problem*), and the strategy
ports the scripts one by one, each proved by the parity harness (*Strategy*, Q-2). Q-2 already
counts every ported script. G-1 to G-8 name no sensor, and
[TEST-parity-harness](TEST-parity-harness.md) compares findings mapped to rule IDs. A sensor
prints a report, not findings. The template ships twelve sensors:

- **Only files and git:** `sensor-action-refs.sh`, `sensor-backlog.sh`, `sensor-binding-adrs.sh`,
  `sensor-plain-rules.sh`, `sensor-plan-frontier.sh`, `sensor-review-binding.sh`,
  `sensor-revision-due.sh`, and `sensor-tests-kept.sh`.
- **A process of the project:** `sensor-red-commits.sh` runs `scripts/check-all.sh` of each commit
  it reads.
- **The network:** `sensor-release-notes.sh`, `sensor-repo-settings.sh`, and
  `sensor-work-measures.sh` call `gh` and `jq`.

Two sensors also run as checks. `scripts/check-all.sh` runs `sensor-plain-rules.sh --require` and
`sensor-action-refs.sh --require sha`, and fails on their exit 1. A sensor's report is prose a
session reads, and its first line is what `check-all.sh` shows. Some sensors exit 1 where they
cannot read their input, as `sensor-revision-due.sh` does without a readable *Last design
revision* line. The contract of the command line gives every cause that keeps `nuc` from deciding
the exit 2, and keeps the standard output for findings and measures; the report of a sensor needs
a clause of its own, which
[API-command-line-with-sense](API-command-line-with-sense.md) adds. Every bash sensor takes
`--root <dir>`, where every `nuc` command takes `--repo <path>`. Some sensors exist only from a
later release: `sensor-backlog.sh` and `sensor-plain-rules.sh` from v0.8.0.

[ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md) decides the rules, the hooks,
and the shell, and leaves the module layout inside the core beyond them open. `nuc measure` is the
prior art of a command whose core returns neither an outcome nor an answer: `src/measure.rs`
counts in the core, and the shell reads and prints.

The ports showed two sensors whose bash reads an input on some answers and not on others.
`sensor-review-binding.sh` asks git about the commit a hand-off's `Reviewed at:` line names only
where the line names one: whether the range holds it, then how many commits follow it, then their
diffstat. Before v0.11.0 it also binds the hand-off of each merge that lands a branch on the
first-parent line, read from the merge's message. `sensor-binding-adrs.sh` diffs a path only where
an ADR binds it by its checks. Their cores in `src/sense/review_binding.rs` and
`src/sense/binding_adrs.rs` ask the shell for those inputs: review-binding through the trait
`review_binding::Repo`, thirteen questions that `shell::sense::ReviewRepo` answers each with one git
call or the hand-off, and binding-adrs through a callback, `diffed`, which ran one git diff a call.
Each core reads no file and runs no process, which `tests/functional_core.rs` holds. Each broke
"takes the values the shell gathered" of the superseded ADR, since the core picked the next git
call. Which paths binding-adrs diffs is known once the ADRs are read, whatever a diff says, so a
value of its core can name them: with this ADR it takes the diffs as a value, those of the paths
`binding_adrs::checked_paths` names. review-binding asks a chain, each question decided by the
answer before. The hand-off of ticket 07 of `sensors-under-nuc-sense` and both its reviews left the
shape open. The first review found no simpler shape that reads only where the bash reads. The unit
tests of `src/sense/review_binding/tests.rs` record the questions the core asks a fake `Repo` and
decide their order.

[ARCH-rules-need-only-git](ARCH-rules-need-only-git.md) keeps every rule that needs a process
other than git in the bash, for Q-4 and Q-7. The constraints make git the only runtime dependency,
and the non-goals allow the network only in a sensor that names it. Without a decision, a session
would put the sensors under `nuc check`, where their reports do not fit the format of a finding,
or would port `sensor-red-commits.sh` with a run of the project's scripts.

## Decision

We will port the template's eight sensors that need only git as `nuc sense <name>`, with the
bash's report and the exit codes of
[API-command-line-with-sense](API-command-line-with-sense.md). The *Core* clause names where a
sensor's core asks the shell instead of taking values.

- **Names:** `<name>` is the part of the script's name after `sensor-` and before `.sh`, such as
  `revision-due`. `nuc sense` knows `action-refs`, `backlog`, `binding-adrs`, `plain-rules`,
  `plan-frontier`, `review-binding`, `revision-due`, and `tests-kept`.
- **Not ported:** `red-commits`, `release-notes`, `repo-settings`, and `work-measures`. `nuc sense`
  must refuse each of them as an unknown sensor, with exit 2. The template keeps them in its
  bash.
- **Arguments:** `nuc sense` must take the arguments the bash sensor takes, `--require` among
  them, with the same meaning. It must take `--repo <path>` in place of `--root <dir>`, as every
  `nuc` command does, and must refuse `--root`.
- **Standard output:** `nuc sense` must write the report on the standard output, byte for byte as
  the bash sensor writes it there.
- **Core:** each sensor must be a function of the library outside `src/shell/`. It takes the
  values the shell gathered, or asks the shell where the next item allows, and returns its report
  or a refusal. The shell prints the report or
  the refusal and exits. A sensor reads no file and runs no process, as a rule does.
  - **Asking the shell:** a sensor's core may ask the shell for its inputs instead of taking them as
    values, where two things hold. First, gathering every input its bash may read, up front, would
    read more than the bash. It would make a git call the bash does not make, or refuse where the
    bash reads nothing. Second, no value the core returns first can tell the shell what to read.
    `binding_adrs::reads_adrs` is such a value: it tells the shell whether to read the ADRs. Where
    each read depends on the answer to the one before, no such value exists.
  - Such a core must ask through a trait or a callback that it declares and only the shell
    implements. The same trait may carry the sensor's questions that the bash asks on every run,
    so that one seam answers them all.
  - A trait the sensors' core declares must not be implemented in the core outside its test code.
    The test code is a `tests.rs` or a `#[cfg(test)]` module, where a fake answers it.
  - The shell must answer each question through `shell::git` or the `Tree` of `shell::tree`, under
    the guards of [SEC-git-cleared-without-lazy-fetch](SEC-git-cleared-without-lazy-fetch.md),
    [SEC-repository-commands-neutralized-or-refused](SEC-repository-commands-neutralized-or-refused.md),
    and `Tree::resolve`, as it gathers a value.
  - A sensor for which the two things do not hold must take its inputs as values.
  - review-binding asks through the trait `review_binding::Repo`. binding-adrs takes as a value
    the diffs of the paths `binding_adrs::checked_paths` names, which the shell reads first.
- **Release:** `nuc sense` must report as the sensor of the release the tree carries, as the rules
  do ([API-version-coupling-hook-defaults](API-version-coupling-hook-defaults.md)). Where the
  README names no release, it must report as the newest release `nuc` knows.
  - A tree whose release has no such sensor is not decided. `nuc sense` must exit 2 there, with
    the key `[sense/<name>:release]`.
  - `tests/parity/releases.tsv` must name the releases at which each sensor changed, in a row
    `sense/<name>`, as it does for each rule. A change of the sensor's script or of a library it
    sources counts.
- **Configuration:** `nuc sense` must read `nuc.toml` as `nuc check` does and refuse where
  `nuc check` refuses, so that G-5 holds for it. It must not read the `rules` key for a choice of
  its own.
- **Exit codes and keys:** [API-command-line-with-sense](API-command-line-with-sense.md) decides
  them for `nuc sense`, and this ADR repeats none of them.
- **Parity:** the parity harness must run each ported sensor and `nuc sense` on every tree and
  variant of the corpus whose release has the sensor.
  - A table beside the harness must name the arguments and the git state of each run.
  - The harness must compare the standard output byte for byte and the exit status, and fail on
    any difference.
  - A second table must list each bash exit that differs from `nuc`'s by this ADR, with the
    release and the cause. The harness must fail on a difference no row lists.
  - The corpus must pin a tree of each release at which a ported sensor changed.

**Out of scope:**

- the wording of a refusal on the standard error;
- a sensor the template adds later, which joins the ported or the not ported by the rule above;
- when the template switches its sensors to `nuc sense`;
- an output format other than the bash's.

## Latitude

Whether a core that asks declares a trait or takes a callback is the implementer's: a callback for
one question, a trait for several.

## Alternatives considered

- **The sensors as rules of `nuc check`** — one command, but a report is not a finding with a rule
  ID and a line, and `nuc.toml` would have to turn on what a session runs by hand.
- **Only the two `--require` modes as rules, the reports under `nuc sense`** — the checks in the
  finding format, but one sensor then has two outputs to keep at parity, and two places decide
  the same text.
- **Also the sensors that call `gh`** — the whole set, but a second process beside git breaks the
  constraint on runtime dependencies, and the network breaks Q-11 for a command a gate runs.
- **`red-commits` with a run of `check-all.sh`** — the bash's behaviour, but `nuc` would execute
  repository content, against Q-7, as [ARCH-rules-need-only-git](ARCH-rules-need-only-git.md)
  rejects for the rules.
- **Exit codes as the bash's** — strict parity, but `nuc sense` would answer 1 for "cannot
  decide" where every other command answers 2, and a gate could no longer tell a failed
  `--require` from an unreadable tree.
- **`--root` as the bash names it** — a call copied from the bash runs unchanged, but `nuc` would
  name the repository two ways, and `--repo` is what every command of `nuc` takes.
- **A superseding ADR of ARCH-rules-outcome-hooks-answer for the shape of the core** — a third
  kind of decision, a report, beside the outcome and the answer of
  [ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md). That ADR leaves the
  layout of the core beyond the rules and the hooks open, and `nuc measure` already returns a
  third kind under it. Its structural test holds the sensors to no effect outside the shell as it
  stands.
- **Gather, then decide** — the shell drives the bash's order of questions and hands the core only
  the answers, and every core stays a function of values. But the logic of which question comes
  next, the sensor's own, moves into the shell, and the unit tests of the order of the questions
  are lost. Only the parity harness would still hold it.
- **Gather every input the bash may read, up front** — the cores stay functions of values without
  logic in the shell. But `nuc` reads more than the bash. It makes git calls the bash does not
  make, and refuses where the bash reads nothing, as for a blob a partial clone lacks or a path
  under a diff driver (G-9, Q-2).
- **A value the core returns first, which tells the shell what to read, for review-binding too** —
  binding-adrs does so with `reads_adrs`, `reads_index`, and `checked_paths`, and its core takes
  values. It fits where one answer decides the reads. review-binding asks a chain, each question
  decided by the answer before: the shell would call the core once per answer, the trait in
  another form.
- **binding-adrs keeps its callback `diffed`** — no change of code, but its reads are known before
  any diff, so a value names them, and the condition of this ADR would not hold for it.
- **Let every sensor ask the shell** — one rule for all, but the form of values becomes the
  exception, and a review can no longer tell whether a trait was needed.
- **Review alone, no test** — no test to keep, but a reviewer would have to search the core for
  every sensor that asks, where the test names each by its implementation in `src/shell/`.
- **The bash sensor's report on a tree of a release without it** — the newest sensor would report
  on an old tree, which no bash of that release does, and parity has nothing to compare.
- **Parity of the counts only, not the wording** — looser to keep, but `check-all.sh` shows the
  first line, and a session reads the report as the bash wrote it.
- **Parity on the template's self-test fixtures only** — no corpus run, but the corpus is where
  the rules showed differences the fixtures missed.
- **One pilot sensor, the pattern decided per sensor** — less up front, but each later port would
  reopen the command, the exit codes, and the parity.

## Sources / Prior art

- The template's sensors under `scripts/` and their self-tests `scripts/test-sensor-*.sh`, at the
  release this repository carries.
- `scripts/check-all.sh`, which tells a sensor from a check by `--require`.
- The specification: *Problem*, *Strategy*, *Constraints*, *Non-Goals*, G-5, G-9, Q-2, Q-4, Q-7, and
  Q-11.
- The hand-off and both reviews of ticket 07 of `sensors-under-nuc-sense`, and ticket 01 of
  `docs/plan/revision-2026-11-20/`.
- The catalogue of `propose-adr`: *Functional core, imperative shell* and *Ports and adapters*.
  Gary Bernhardt, *Functional Core, Imperative Shell* and *Boundaries*, on the core of values,
  which this ADR keeps as the rule and leaves for a question only by an earlier answer.
- [ARCH-rules-need-only-git](ARCH-rules-need-only-git.md),
  [API-command-line-with-measure](API-command-line-with-measure.md),
  [ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md), `src/measure.rs`, and
  [TEST-parity-harness](TEST-parity-harness.md), whose patterns this ADR carries over to the
  sensors.

## Consequences

- Positive: a project runs eight sensors without bash (Q-4), and the two sensors that act as
  checks can move to `nuc` with the rest of the gates. Parity is decided byte for byte, so the
  report a session reads does not drift (Q-2). A sensor that asks reads only what its bash reads,
  so it makes no git call the bash does not make and refuses no input the bash leaves unread
  (G-9, Q-2). Its decisions keep unit tests of the order of its questions.
- Negative / trade-offs: `nuc` copies the bash's prose, and every change of a sensor's wording in
  a template release is a change of `nuc`. The parity harness needs scenarios with arguments and
  git state, since most sensors read a range of commits, and a pinned tree alone has none. The
  exit table is one more table to keep. Four sensors stay in the bash. A core that asks the shell
  is no function of values, so its unit tests need a fake of the trait or the callback. That fake
  answers as the test says, not as git does, and only the parity harness compares the answers
  with git's.
- Follow-ups: the structural test of the traits, beside this proposal and before its acceptance;
  in the change that accepts it, the `Verifies:` markers that cite the superseded ADR move to this
  one, in `tests/contract.rs`, `tests/releases.rs`, `tests/functional_core.rs`, and
  `scripts/test-check-parity.sh`, with the citations in comments; the Sensors of
  `docs/ARCHITECTURE.md` and the header of `src/sense.rs`, which call the cores pure functions; a
  later decision on the sensors that call `gh`, should the constraint on runtime dependencies
  move.

## Enforcement

`tests/contract.rs` must cite this ADR with `Verifies: API-sensor-cores-ask-the-shell`. It must
check the following:

- `nuc sense` with each of the four not ported names exits 2, with the key `[sense/unknown:name]`.
- With a `nuc.toml` above the running version, `nuc sense` exits 2 as too old before it judges the
  name.
- `nuc sense` with `--root` exits 2.
- On a tree whose release has no such sensor, `nuc sense` exits 2, with the key
  `[sense/<name>:release]`.
- A sensor that cannot read its input exits 2, and a failed `--require` exits 1.

`tests/releases.rs` must cite it the same way and hold each row `sense/<name>` to the template's
trees, as it holds the rows of the rules. `tests/functional_core.rs` must cite it and hold the
sensors' modules to no effect outside the shell. It must also check that each trait declared in
`src/sense.rs` or under `src/sense/` is implemented only in `src/shell/` or in the core's test code,
a `tests.rs` or a `#[cfg(test)]` module. A reviewer then finds every core that asks through a trait
by its implementation in `src/shell/`. The existing lists of `tests/functional_core.rs` decide that
the core reads no file and runs no process, so a question can be answered only in the shell.

**Not mechanically decidable:** five parts.

- Whether the two things hold for a sensor whose core asks.
- Whether the shell's answers keep the guards.
- Whether a sensor that does not ask takes all its inputs as values.
- A callback, and a trait another module of the core declares outside `src/sense/`, which the
  test does not search.
- A trait the test tells only by its name. One implemented under a renamed import escapes it. One
  of the same name declared elsewhere is held as the core's.

The reviewer of the change decides each against the bash sensor, and the maintainer who merges it
confirms.

The self-test of the parity harness, `scripts/test-check-parity.sh`, must cite it the same way. It
must check that each of the following fails:

- a pair of sensor outputs that differ in one byte;
- a pair that differs in an exit status no row of the exit table lists.
